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

/// A line segment representation in local metric coordinates for R-Tree spatial indexing
#[derive(Debug, Clone)]
pub struct IndexedSegment {
    pub p1: (f64, f64), // (lng, lat)
    pub p2: (f64, f64),
    pub road_type: String,
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

/// Build an RTree of driving road segments
pub fn build_driving_roads_index(road_ways: &[crate::osm::ResolvedOsmWay]) -> RTree<IndexedSegment> {
    let mut segments = Vec::new();
    for way in road_ways {
        let highway = way
            .tags
            .get("highway")
            .cloned()
            .unwrap_or_else(|| "unclassified".to_string());

        for i in 0..way.coords.len() - 1 {
            segments.push(IndexedSegment {
                p1: way.coords[i],
                p2: way.coords[i + 1],
                road_type: highway.clone(),
            });
        }
    }
    RTree::bulk_load(segments)
}

/// Calculate minimum distance in meters from a point to a line segment in geographic space
pub fn point_to_segment_distance_m(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    // Project to local metric plane relative to point p
    let lat_rad = p.1.to_radians();
    let m_per_deg_lat = 111_132.954 - 559.822 * (2.0 * lat_rad).cos() + 1.175 * (4.0 * lat_rad).cos();
    let m_per_deg_lon = 111_412.84 * lat_rad.cos() - 93.5 * (3.0 * lat_rad).cos();

    let px = 0.0;
    let py = 0.0;

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

    // Projection scalar t of point p onto segment AB
    let apx = px - ax;
    let apy = py - ay;
    let t = ((apx * abx + apy * aby) / ab_len_sq).clamp(0.0, 1.0);

    let closest_x = ax + t * abx;
    let closest_y = ay + t * aby;

    (closest_x * closest_x + closest_y * closest_y).sqrt()
}

/// Compute distance in meters from a candidate way's midpoint to the nearest driving road in the R-tree
pub fn compute_distance_to_nearest_driving_road(
    coords: &[(f64, f64)],
    index: &RTree<IndexedSegment>,
    search_radius_deg: f64,
) -> f64 {
    if index.size() == 0 {
        return 999.0;
    }

    let mid = polyline_midpoint(coords);

    // Search window in degrees (~200m)
    let search_box = AABB::from_corners(
        [mid.0 - search_radius_deg, mid.1 - search_radius_deg],
        [mid.0 + search_radius_deg, mid.1 + search_radius_deg],
    );

    let mut min_dist_m = 999.0;

    for seg in index.locate_in_envelope(&search_box) {
        let d = point_to_segment_distance_m(mid, seg.p1, seg.p2);
        if d < min_dist_m {
            min_dist_m = d;
        }
    }

    // If no segment found in envelope, search nearest neighbor
    if min_dist_m >= 990.0 {
        if let Some(seg) = index.nearest_neighbor(&[mid.0, mid.1]) {
            min_dist_m = point_to_segment_distance_m(mid, seg.p1, seg.p2);
        }
    }

    (min_dist_m * 10.0).round() / 10.0
}
