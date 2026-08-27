use crate::{
    config::LosConfig,
    infrastructure::{classify_cycle_infrastructure, InfrastructureType},
    los::lookup_level_of_service,
    osm::{extract_resolved_ways, parse_overpass_json, ResolvedOsmWay},
    spatial::{
        build_driving_roads_index, compute_distance_to_nearest_driving_road, polyline_length_m,
    },
    speed::{classify_speed_category, is_active_travel_path, parse_maxspeed},
    traffic::{classify_aadt_category, estimate_traffic_aadt},
};
use geo_types::LineString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn empty_linestring() -> LineString<f64> {
    LineString(vec![])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMetric {
    pub length_km: f64,
    pub percentage: f64,
    pub count: usize,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStats {
    pub total_length_km: f64,
    pub total_segments: usize,
    pub los_breakdown: HashMap<String, CategoryMetric>,
    pub infra_breakdown: HashMap<String, CategoryMetric>,
    pub high_med_pct: f64,
    pub unsuitable_pct: f64,
    pub average_speed_mph: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedSegment {
    pub osm_id: i64,
    pub name: String,
    pub highway: String,
    pub infrastructure: String,
    pub speed_limit_mph: Option<f64>,
    pub speed_category: String,
    pub estimated_aadt: Option<u32>,
    pub aadt_category: String,
    pub distance_to_road_m: Option<f64>,
    pub length_m: f64,
    pub los_score: i8,
    pub los_label: String,
    pub los_color: String,
    #[serde(skip, default = "empty_linestring")]
    pub geometry: LineString<f64>,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LosAnalysisResult {
    pub segments_count: usize,
    pub segments: Vec<ClassifiedSegment>,
    pub summary: SummaryStats,
}

pub struct LosEngine {
    pub raw_ways: Vec<ResolvedOsmWay>,
}

impl LosEngine {
    /// Create engine from Overpass API JSON string
    pub fn from_overpass_json(osm_json: &str) -> anyhow::Result<Self> {
        let resp = parse_overpass_json(osm_json)?;
        let raw_ways = extract_resolved_ways(&resp.elements);
        Ok(Self { raw_ways })
    }

    /// Run Level of Service analysis with custom configuration
    pub fn run_analysis(&self, config: &LosConfig) -> anyhow::Result<LosAnalysisResult> {
        // 1. Identify driving network for distance calculations
        let driving_ways: Vec<ResolvedOsmWay> = self
            .raw_ways
            .iter()
            .filter(|w| {
                let hw = w.tags.get("highway").map(|s| s.as_str()).unwrap_or("");
                is_driving_highway(hw)
            })
            .cloned()
            .collect();

        // Build spatial index of driving network
        let driving_index = build_driving_roads_index(&driving_ways);

        let mut classified_segments = Vec::new();

        for way in &self.raw_ways {
            let highway = match way.tags.get("highway") {
                Some(h) => h.as_str(),
                None => continue,
            };

            // Filter out non-cycling / non-routable ways
            if is_excluded_for_cycling(highway, &way.tags) {
                continue;
            }

            let length_m = polyline_length_m(&way.coords);
            if length_m < 1.0 {
                continue;
            }

            // Calculate distance to nearest driving road (for paths/cycleways)
            let distance_to_road_m = if is_active_travel_path(highway) {
                Some(compute_distance_to_nearest_driving_road(
                    &way.coords,
                    &driving_index,
                    0.002, // ~200m search box in degrees
                ))
            } else {
                None
            };

            // Classify infrastructure
            let infra_type = classify_cycle_infrastructure(
                highway,
                &way.tags,
                distance_to_road_m.unwrap_or(0.0),
                config.min_offroad_distance,
                config.min_wide_width,
            );

            // If user disabled mixed traffic streets, filter them out
            if !config.include_mixed_traffic && infra_type == InfrastructureType::MixedTrafficStreet {
                continue;
            }

            // Clean speeds and classify
            let raw_speed = way.tags.get("maxspeed").map(|s| s.as_str());
            let speed_limit_mph = parse_maxspeed(raw_speed, highway, &config.speed_defaults);
            let speed_cat = speed_limit_mph
                .map(classify_speed_category)
                .unwrap_or("<20 mph");

            // Estimate traffic AADT and classify
            let estimated_aadt = estimate_traffic_aadt(highway, &config.aadt_estimates, config.traffic_multiplier);
            let aadt_cat = classify_aadt_category(estimated_aadt);

            // Lookup Level of Service
            let los = lookup_level_of_service(speed_cat, aadt_cat, infra_type);

            let name = way
                .tags
                .get("name")
                .cloned()
                .or_else(|| way.tags.get("ref").cloned())
                .unwrap_or_else(|| "Unnamed Link".to_string());

            let line_string = LineString::from(
                way.coords
                    .iter()
                    .map(|(lng, lat)| geo_types::Coord { x: *lng, y: *lat })
                    .collect::<Vec<_>>(),
            );

            classified_segments.push(ClassifiedSegment {
                osm_id: way.id,
                name,
                highway: highway.to_string(),
                infrastructure: infra_type.as_str().to_string(),
                speed_limit_mph,
                speed_category: speed_cat.to_string(),
                estimated_aadt,
                aadt_category: aadt_cat.to_string(),
                distance_to_road_m,
                length_m: (length_m * 10.0).round() / 10.0,
                los_score: los.raw_value(),
                los_label: los.label().to_string(),
                los_color: los.color().to_string(),
                geometry: line_string,
                tags: way.tags.clone(),
            });
        }

        // Calculate summary statistics
        let summary = compute_summary_statistics(&classified_segments);

        Ok(LosAnalysisResult {
            segments_count: classified_segments.len(),
            segments: classified_segments,
            summary,
        })
    }
}

fn is_driving_highway(highway: &str) -> bool {
    matches!(
        highway,
        "motorway"
            | "motorway_link"
            | "trunk"
            | "trunk_link"
            | "primary"
            | "primary_link"
            | "secondary"
            | "secondary_link"
            | "tertiary"
            | "tertiary_link"
            | "unclassified"
            | "residential"
            | "living_street"
            | "service"
    )
}

fn is_excluded_for_cycling(highway: &str, tags: &HashMap<String, String>) -> bool {
    // Exclude motorways unless explicit bicycle permission
    if highway == "motorway" || highway == "motorway_link" {
        let bicycle = tags.get("bicycle").map(|s| s.as_str()).unwrap_or("");
        if bicycle != "yes" && bicycle != "designated" {
            return true;
        }
    }

    // Excluded highway types
    if matches!(
        highway,
        "construction"
            | "proposed"
            | "abandoned"
            | "corridor"
            | "elevator"
            | "escalator"
            | "platform"
            | "raceway"
            | "rest_area"
            | "services"
            | "steps"
    ) {
        return true;
    }

    // Bicycle tag checks
    if let Some(b) = tags.get("bicycle") {
        if matches!(b.as_str(), "no" | "dismount" | "discouraged") {
            return true;
        }
    }

    // Rough path exclusions (surface checks)
    if matches!(highway, "path" | "footway") {
        let bicycle = tags.get("bicycle").map(|s| s.as_str()).unwrap_or("");
        let surface = tags.get("surface").map(|s| s.as_str()).unwrap_or("");
        let smoothness = tags.get("smoothness").map(|s| s.as_str()).unwrap_or("");

        if bicycle != "designated" && bicycle != "yes" {
            if matches!(
                surface,
                "ground" | "grass" | "sand" | "dirt" | "wood"
            ) || matches!(
                smoothness,
                "very_bad" | "horrible" | "very_horrible" | "impassable"
            ) {
                return true;
            }
        }
    }

    false
}

fn compute_summary_statistics(segments: &[ClassifiedSegment]) -> SummaryStats {
    let mut total_length_m = 0.0;
    let mut los_lengths: HashMap<String, (f64, usize, String)> = HashMap::new();
    let mut infra_lengths: HashMap<String, (f64, usize, String)> = HashMap::new();

    let mut speed_sum_weighted = 0.0;
    let mut speed_len_weighted = 0.0;

    for s in segments {
        total_length_m += s.length_m;

        let entry_los = los_lengths
            .entry(s.los_label.clone())
            .or_insert((0.0, 0, s.los_color.clone()));
        entry_los.0 += s.length_m;
        entry_los.1 += 1;

        let infra_color = match s.infrastructure.as_str() {
            "Segregated Track (wide)" => "#054d05",
            "Off Road Path" => "#3a9120",
            "Segregated Track (narrow)" => "#87d668",
            "Shared Footway" => "#ffbf00",
            "Painted Cycle Lane" => "#f97316",
            _ => "#64748b",
        };

        let entry_infra = infra_lengths
            .entry(s.infrastructure.clone())
            .or_insert((0.0, 0, infra_color.to_string()));
        entry_infra.0 += s.length_m;
        entry_infra.1 += 1;

        if let Some(spd) = s.speed_limit_mph {
            speed_sum_weighted += spd * s.length_m;
            speed_len_weighted += s.length_m;
        }
    }

    let total_length_km = total_length_m / 1000.0;

    let mut los_breakdown = HashMap::new();
    for (label, (len_m, count, color)) in los_lengths {
        let len_km = len_m / 1000.0;
        let percentage = if total_length_m > 0.0 {
            (len_m / total_length_m) * 100.0
        } else {
            0.0
        };
        los_breakdown.insert(
            label,
            CategoryMetric {
                length_km: (len_km * 100.0).round() / 100.0,
                percentage: (percentage * 10.0).round() / 10.0,
                count,
                color,
            },
        );
    }

    let mut infra_breakdown = HashMap::new();
    for (label, (len_m, count, color)) in infra_lengths {
        let len_km = len_m / 1000.0;
        let percentage = if total_length_m > 0.0 {
            (len_m / total_length_m) * 100.0
        } else {
            0.0
        };
        infra_breakdown.insert(
            label,
            CategoryMetric {
                length_km: (len_km * 100.0).round() / 100.0,
                percentage: (percentage * 10.0).round() / 10.0,
                count,
                color,
            },
        );
    }

    let high_len_m = segments
        .iter()
        .filter(|s| s.los_score == 3 || s.los_score == 2)
        .map(|s| s.length_m)
        .sum::<f64>();

    let unsuitable_len_m = segments
        .iter()
        .filter(|s| s.los_score <= 0)
        .map(|s| s.length_m)
        .sum::<f64>();

    let high_med_pct = if total_length_m > 0.0 {
        ((high_len_m / total_length_m) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    let unsuitable_pct = if total_length_m > 0.0 {
        ((unsuitable_len_m / total_length_m) * 1000.0).round() / 10.0
    } else {
        0.0
    };

    let average_speed_mph = if speed_len_weighted > 0.0 {
        ((speed_sum_weighted / speed_len_weighted) * 10.0).round() / 10.0
    } else {
        20.0
    };

    SummaryStats {
        total_length_km: (total_length_km * 100.0).round() / 100.0,
        total_segments: segments.len(),
        los_breakdown,
        infra_breakdown,
        high_med_pct,
        unsuitable_pct,
        average_speed_mph,
    }
}
