use crate::config::LosConfig;
use crate::speed::{is_urban_context, parse_maxspeed_with_context};
use crate::traffic::estimate_traffic_aadt;
use rstar::{PointDistance, RTree, RTreeObject, AABB};

const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Calculate Haversine distance in meters between two (lng, lat) coordinates
pub fn haversine_distance(p1: (f64, f64), p2: (f64, f64)) -> f64 {
    let (lon1, lat1) = (p1.0.to_radians(), p1.1.to_radians());
    let (lon2, lat2) = (p2.0.to_radians(), p2.1.to_radians());

    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_M * c
}

/// Calculate total length in meters of a coordinate sequence
pub fn polyline_length_m(coords: &[(f64, f64)]) -> f64 {
    if coords.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    for i in 0..coords.len() - 1 {
        total += haversine_distance(coords[i], coords[i + 1]);
    }
    total
}

/// Get the midpoint / point-on-surface of a polyline
pub fn polyline_midpoint(coords: &[(f64, f64)]) -> (f64, f64) {
    if coords.is_empty() {
        return (0.0, 0.0);
    }
    if coords.len() == 1 {
        return coords[0];
    }
    let total_len = polyline_length_m(coords);
    if total_len <= 0.0 {
        return coords[0];
    }

    let target = total_len / 2.0;
    let mut accumulated = 0.0;

    for i in 0..coords.len() - 1 {
        let seg_len = haversine_distance(coords[i], coords[i + 1]);
        if accumulated + seg_len >= target && seg_len > 0.0 {
            let frac = (target - accumulated) / seg_len;
            let lng = coords[i].0 + frac * (coords[i + 1].0 - coords[i].0);
            let lat = coords[i].1 + frac * (coords[i + 1].1 - coords[i].1);
            return (lng, lat);
        }
        accumulated += seg_len;
    }

    coords[coords.len() / 2]
}

/// Sample points along a polyline every `step_m` meters (`atf-lab/osmactive` 5m sampling pattern)
pub fn sample_polyline_points(coords: &[(f64, f64)], step_m: f64) -> Vec<(f64, f64)> {
    if coords.is_empty() {
        return Vec::new();
    }
    if coords.len() == 1 {
        return vec![coords[0]];
    }
    let total_len = polyline_length_m(coords);
    if total_len <= step_m {
        return vec![coords[0], polyline_midpoint(coords), *coords.last().unwrap()];
    }

    let num_steps = ((total_len / step_m).ceil() as usize).clamp(3, 64);
    let actual_step = total_len / (num_steps as f64);

    let mut samples = Vec::with_capacity(num_steps + 1);
    samples.push(coords[0]);

    let mut target = actual_step;
    let mut acc = 0.0;

    for i in 0..coords.len() - 1 {
        let seg_len = haversine_distance(coords[i], coords[i + 1]);
        if seg_len <= 1e-6 {
            continue;
        }
        while target <= acc + seg_len && samples.len() < num_steps {
            let frac = ((target - acc) / seg_len).clamp(0.0, 1.0);
            let lng = coords[i].0 + frac * (coords[i + 1].0 - coords[i].0);
            let lat = coords[i].1 + frac * (coords[i + 1].1 - coords[i].1);
            samples.push((lng, lat));
            target += actual_step;
        }
        acc += seg_len;
    }

    if let Some(&last) = coords.last() {
        samples.push(last);
    }
    samples
}

/// A line segment representation in local metric coordinates for R-Tree spatial indexing
#[derive(Debug, Clone)]
pub struct IndexedSegment {
    pub p1: (f64, f64), // (lng, lat)
    pub p2: (f64, f64),
    pub road_type: String,
    pub speed_mph: Option<f64>,
    pub aadt: Option<u32>,
}

impl RTreeObject for IndexedSegment {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let min_x = self.p1.0.min(self.p2.0);
        let max_x = self.p1.0.max(self.p2.0);
        let min_y = self.p1.1.min(self.p2.1);
        let max_y = self.p1.1.max(self.p2.1);
        AABB::from_corners([min_x, min_y], [max_x, max_y])
    }
}

impl PointDistance for IndexedSegment {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let p = (point[0], point[1]);
        let d = point_to_segment_distance_m(p, self.p1, self.p2);
        d * d
    }
}

/// Build an RTree of driving road segments with default configuration
pub fn build_driving_roads_index(
    road_ways: &[crate::osm::ResolvedOsmWay],
) -> RTree<IndexedSegment> {
    build_driving_roads_index_with_config(road_ways, &LosConfig::default())
}

/// Build an RTree of driving road segments carrying resolved speed and AADT attributes
pub fn build_driving_roads_index_with_config(
    road_ways: &[crate::osm::ResolvedOsmWay],
    config: &LosConfig,
) -> RTree<IndexedSegment> {
    let mut segments = Vec::new();
    for way in road_ways {
        let highway = way
            .tags
            .get("highway")
            .cloned()
            .unwrap_or_else(|| "unclassified".to_string());
        let urban = is_urban_context(&way.tags);
        let speed_mph = parse_maxspeed_with_context(
            way.tags.get("maxspeed").map(|s| s.as_str()),
            &highway,
            urban,
            &config.speed_defaults,
        );
        let aadt = estimate_traffic_aadt(
            &highway,
            &config.aadt_estimates,
            config.traffic_multiplier,
        );

        for i in 0..way.coords.len() - 1 {
            segments.push(IndexedSegment {
                p1: way.coords[i],
                p2: way.coords[i + 1],
                road_type: highway.clone(),
                speed_mph,
                aadt,
            });
        }
    }
    RTree::bulk_load(segments)
}

/// Calculate minimum distance in meters from a point to a line segment in geographic space
pub fn point_to_segment_distance_m(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let lat_rad = p.1.to_radians();
    let m_per_deg_lat =
        111_132.954 - 559.822 * (2.0 * lat_rad).cos() + 1.175 * (4.0 * lat_rad).cos();
    let m_per_deg_lon = 111_412.84 * lat_rad.cos() - 93.5 * (3.0 * lat_rad).cos();

    let ax = (a.0 - p.0) * m_per_deg_lon;
    let ay = (a.1 - p.1) * m_per_deg_lat;

    let bx = (b.0 - p.0) * m_per_deg_lon;
    let by = (b.1 - p.1) * m_per_deg_lat;

    let abx = bx - ax;
    let aby = by - ay;
    let ab_len_sq = abx * abx + aby * aby;

    if ab_len_sq <= 1e-6 {
        return (ax * ax + ay * ay).sqrt();
    }

    let t = ((-ax * abx + -ay * aby) / ab_len_sq).clamp(0.0, 1.0);
    let closest_x = ax + t * abx;
    let closest_y = ay + t * aby;

    (closest_x * closest_x + closest_y * closest_y).sqrt()
}

#[derive(Debug, Clone)]
pub struct RoadAssociation {
    pub distance_m: f64,
    pub adjacent_highway: Option<String>,
    pub adjacent_speed_mph: Option<f64>,
    pub adjacent_aadt: Option<u32>,
}

/// 5m-sampled median distance & adjacent carriageway attribute transfer (`atf-lab/osmactive` `ci_dist_m`)
pub fn associate_nearest_driving_road(
    coords: &[(f64, f64)],
    index: &RTree<IndexedSegment>,
    search_radius_deg: f64,
) -> RoadAssociation {
    if index.size() == 0 || coords.is_empty() {
        return RoadAssociation {
            distance_m: 999.0,
            adjacent_highway: None,
            adjacent_speed_mph: None,
            adjacent_aadt: None,
        };
    }

    let samples = sample_polyline_points(coords, 5.0);
    let mut hits: Vec<(f64, Option<String>, Option<f64>, Option<u32>)> =
        Vec::with_capacity(samples.len());

    for pt in samples {
        let search_box = AABB::from_corners(
            [pt.0 - search_radius_deg, pt.1 - search_radius_deg],
            [pt.0 + search_radius_deg, pt.1 + search_radius_deg],
        );

        let mut best_d = 999.0;
        let mut best_seg: Option<&IndexedSegment> = None;

        for seg in index.locate_in_envelope(&search_box) {
            let d = point_to_segment_distance_m(pt, seg.p1, seg.p2);
            if d < best_d {
                best_d = d;
                best_seg = Some(seg);
            }
        }

        if best_d >= 990.0 {
            if let Some(seg) = index.nearest_neighbor(&[pt.0, pt.1]) {
                best_d = point_to_segment_distance_m(pt, seg.p1, seg.p2);
                best_seg = Some(seg);
            }
        }

        hits.push((
            best_d,
            best_seg.map(|s| s.road_type.clone()),
            best_seg.and_then(|s| s.speed_mph),
            best_seg.and_then(|s| s.aadt),
        ));
    }

    hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mid_idx = hits.len() / 2;
    let median = &hits[mid_idx];

    RoadAssociation {
        distance_m: (median.0 * 10.0).round() / 10.0,
        adjacent_highway: median.1.clone(),
        adjacent_speed_mph: median.2,
        adjacent_aadt: median.3,
    }
}

/// Compute median distance in meters from a candidate way to the nearest driving road
pub fn compute_distance_to_nearest_driving_road(
    coords: &[(f64, f64)],
    index: &RTree<IndexedSegment>,
    search_radius_deg: f64,
) -> f64 {
    associate_nearest_driving_road(coords, index, search_radius_deg).distance_m
}
