use crate::{
    config::{LosConfig, TravelMode},
    infrastructure::{
        classify_cycle_infrastructure, extract_incline_pct, extract_separation,
        extract_surface_quality, extract_width_m, sidewalk_status, InfrastructureType,
    },
    junctions::{analyze_junctions, ClassifiedJunction, JunctionSummaryStats},
    los::{
        lookup_level_of_service, score_cycling_grade, score_multimodal_grade, score_walking_grade,
        score_wheeling_grade, MmlosGrade,
    },
    osm::{extract_resolved_ways, extract_tagged_nodes, parse_overpass_json, ResolvedOsmNode, ResolvedOsmWay},
    spatial::{
        associate_nearest_driving_road, build_driving_roads_index_with_config, polyline_length_m,
    },
    speed::{classify_speed_category, is_active_travel_path, is_urban_context, parse_maxspeed_with_context},
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModeSuitability {
    pub multimodal_ab_pct: f64,
    pub walking_ab_pct: f64,
    pub wheeling_ab_pct: f64,
    pub cycling_ab_pct: f64,
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
    pub mode_suitability: ModeSuitability,
    pub junctions: JunctionSummaryStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedSegment {
    pub osm_id: i64,
    pub name: String,
    pub highway: String,
    pub infrastructure: String,
    pub separation: String,
    pub surface: String,
    pub width_m: Option<f64>,
    pub incline_pct: Option<f64>,
    pub is_lit: bool,
    pub is_urban: bool,
    pub speed_limit_mph: Option<f64>,
    pub speed_category: String,
    pub estimated_aadt: Option<u32>,
    pub aadt_category: String,
    pub distance_to_road_m: Option<f64>,
    pub length_m: f64,
    pub grade_multimodal: String,
    pub grade_walking: String,
    pub grade_wheeling: String,
    pub grade_cycling: String,
    pub active_grade: String,
    pub cbd_label: String,
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
    pub junctions_count: usize,
    pub junctions: Vec<ClassifiedJunction>,
    pub summary: SummaryStats,
}

pub struct LosEngine {
    pub raw_ways: Vec<ResolvedOsmWay>,
    pub tagged_nodes: Vec<ResolvedOsmNode>,
}

impl LosEngine {
    /// Create engine from Overpass API JSON string
    pub fn from_overpass_json(osm_json: &str) -> anyhow::Result<Self> {
        let resp = parse_overpass_json(osm_json)?;
        let raw_ways = extract_resolved_ways(&resp.elements);
        let tagged_nodes = extract_tagged_nodes(&resp.elements);
        Ok(Self {
            raw_ways,
            tagged_nodes,
        })
    }

    /// Run Multi-Modal Level of Service (MMLoS) analysis on segments and junctions
    pub fn run_analysis(&self, config: &LosConfig) -> anyhow::Result<LosAnalysisResult> {
        let driving_ways: Vec<ResolvedOsmWay> = self
            .raw_ways
            .iter()
            .filter(|w| {
                let hw = w.tags.get("highway").map(|s| s.as_str()).unwrap_or("");
                is_driving_highway(hw)
            })
            .cloned()
            .collect();

        // Build R-tree of driving carriageways with resolved speeds & AADT for 5m sidepath transfer
        let driving_index = build_driving_roads_index_with_config(&driving_ways, config);

        let mut classified_segments = Vec::new();

        for way in &self.raw_ways {
            let Some(highway) = way.tags.get("highway").map(|s| s.as_str()) else {
                continue;
            };

            if is_non_network_way(highway) {
                continue;
            }

            let cycling_allowed = !is_excluded_for_cycling(highway, &way.tags);
            let walking_allowed = !is_excluded_for_walking(highway, &way.tags);

            // Exclude ways inaccessible to the selected mode
            match config.active_mode {
                TravelMode::Cycling if !cycling_allowed => continue,
                TravelMode::Walking | TravelMode::Wheeling if !walking_allowed => continue,
                TravelMode::Multimodal if !cycling_allowed && !walking_allowed => continue,
                _ => {}
            }

            let length_m = polyline_length_m(&way.coords);
            if length_m < 1.0 {
                continue;
            }

            // 5m-sampled median distance & adjacent carriageway speed/AADT inheritance
            let road_assoc = if is_active_travel_path(highway) {
                Some(associate_nearest_driving_road(
                    &way.coords,
                    &driving_index,
                    0.002,
                ))
            } else {
                None
            };

            let distance_to_road_m = road_assoc.as_ref().map(|r| r.distance_m);

            let infra_type = classify_cycle_infrastructure(
                highway,
                &way.tags,
                distance_to_road_m.unwrap_or(0.0),
                config.min_offroad_distance,
                config.min_wide_width,
            );

            if !config.include_mixed_traffic
                && infra_type == InfrastructureType::MixedTrafficStreet
            {
                continue;
            }

            let is_urban = is_urban_context(&way.tags);
            let is_lit = way
                .tags
                .get("lit")
                .map(|v| matches!(v.as_str(), "yes" | "24/7" | "automatic"))
                .unwrap_or(is_urban && matches!(highway, "residential" | "primary" | "secondary" | "tertiary" | "pedestrian"));

            // Resolve speed & AADT: roadside active paths within `min_offroad_distance` inherit adjacent carriageway exposure!
            let (speed_limit_mph, estimated_aadt) = if let Some(ref assoc) = road_assoc {
                if assoc.distance_m <= config.min_offroad_distance {
                    (assoc.adjacent_speed_mph, assoc.adjacent_aadt)
                } else {
                    (None, None)
                }
            } else {
                let spd = parse_maxspeed_with_context(
                    way.tags.get("maxspeed").map(|s| s.as_str()),
                    highway,
                    is_urban,
                    &config.speed_defaults,
                );
                let aadt = estimate_traffic_aadt(
                    highway,
                    &config.aadt_estimates,
                    config.traffic_multiplier,
                );
                (spd, aadt)
            };

            let speed_cat = speed_limit_mph
                .map(classify_speed_category)
                .unwrap_or("<20 mph");
            let aadt_cat = classify_aadt_category(estimated_aadt);

            let separation = extract_separation(&way.tags);
            let surface = extract_surface_quality(&way.tags);
            let width_m = extract_width_m(&way.tags);
            let incline_pct = extract_incline_pct(&way.tags);
            let sidewalk = sidewalk_status(&way.tags);
            let kerb_raised = way
                .tags
                .get("kerb")
                .map(|k| k == "raised" || k == "yes")
                .unwrap_or(false);

            let effective_speed = speed_limit_mph.unwrap_or(10.0);

            // Compute Scottish CbD reference label
            let cbd_score = lookup_level_of_service(speed_cat, aadt_cat, infra_type);

            // Compute calibrated A–E grades across all modes
            let g_cycling = score_cycling_grade(
                config.standard,
                effective_speed,
                estimated_aadt,
                infra_type,
                surface,
                cycling_allowed,
            );
            let g_walking = score_walking_grade(
                highway,
                infra_type,
                effective_speed,
                estimated_aadt,
                sidewalk,
                width_m,
                surface,
                is_lit,
                walking_allowed,
            );
            let g_wheeling = score_wheeling_grade(
                highway,
                infra_type,
                g_walking,
                width_m,
                surface,
                incline_pct,
                kerb_raised,
            );
            let g_multimodal = score_multimodal_grade(g_walking, g_wheeling, g_cycling, infra_type);

            let chosen_grade: MmlosGrade = match config.active_mode {
                TravelMode::Multimodal => g_multimodal,
                TravelMode::Walking => g_walking,
                TravelMode::Wheeling => g_wheeling,
                TravelMode::Cycling => g_cycling,
            };

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
                separation: separation.to_string(),
                surface: surface.as_str().to_string(),
                width_m,
                incline_pct,
                is_lit,
                is_urban,
                speed_limit_mph,
                speed_category: speed_cat.to_string(),
                estimated_aadt,
                aadt_category: aadt_cat.to_string(),
                distance_to_road_m,
                length_m: (length_m * 10.0).round() / 10.0,
                grade_multimodal: g_multimodal.letter().to_string(),
                grade_walking: g_walking.letter().to_string(),
                grade_wheeling: g_wheeling.letter().to_string(),
                grade_cycling: g_cycling.letter().to_string(),
                active_grade: chosen_grade.letter().to_string(),
                cbd_label: cbd_score.label().to_string(),
                los_score: chosen_grade.numeric_score(),
                los_label: chosen_grade.label().to_string(),
                los_color: chosen_grade.color().to_string(),
                geometry: line_string,
                tags: way.tags.clone(),
            });
        }

        let (junctions, junction_summary) = if config.include_junctions {
            analyze_junctions(&self.raw_ways, &self.tagged_nodes, config)
        } else {
            (Vec::new(), JunctionSummaryStats::default())
        };

        let summary = compute_summary_statistics(&classified_segments, junction_summary);

        Ok(LosAnalysisResult {
            segments_count: classified_segments.len(),
            segments: classified_segments,
            junctions_count: junctions.len(),
            junctions,
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

fn is_non_network_way(highway: &str) -> bool {
    matches!(
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
    )
}

fn is_excluded_for_walking(highway: &str, tags: &HashMap<String, String>) -> bool {
    if matches!(highway, "motorway" | "motorway_link") {
        return true;
    }
    if let Some(foot) = tags.get("foot") {
        if matches!(foot.as_str(), "no" | "private" | "prohibited") {
            return true;
        }
    }
    if tags.get("motorroad").map(|v| v == "yes").unwrap_or(false) {
        return true;
    }
    false
}

fn is_excluded_for_cycling(highway: &str, tags: &HashMap<String, String>) -> bool {
    if highway == "motorway" || highway == "motorway_link" || highway == "steps" {
        let bicycle = tags.get("bicycle").map(|s| s.as_str()).unwrap_or("");
        if bicycle != "yes" && bicycle != "designated" {
            return true;
        }
    }

    if let Some(b) = tags.get("bicycle") {
        if matches!(b.as_str(), "no" | "dismount" | "discouraged" | "private") {
            return true;
        }
    }

    if matches!(highway, "footway" | "pedestrian") {
        let bicycle = tags.get("bicycle").map(|s| s.as_str()).unwrap_or("");
        let seg = tags.get("segregated").map(|s| s.as_str()).unwrap_or("");
        if !matches!(bicycle, "yes" | "designated" | "permissive") && seg != "yes" {
            return true;
        }
    }

    false
}

fn compute_summary_statistics(
    segments: &[ClassifiedSegment],
    junctions: JunctionSummaryStats,
) -> SummaryStats {
    let mut total_length_m = 0.0;
    let mut los_lengths: HashMap<String, (f64, usize, String)> = HashMap::new();
    let mut infra_lengths: HashMap<String, (f64, usize, String)> = HashMap::new();

    let mut speed_sum_weighted = 0.0;
    let mut speed_len_weighted = 0.0;

    let mut mm_ab_m = 0.0;
    let mut walk_ab_m = 0.0;
    let mut wheel_ab_m = 0.0;
    let mut cycle_ab_m = 0.0;

    for s in segments {
        total_length_m += s.length_m;

        let entry_los = los_lengths
            .entry(s.los_label.clone())
            .or_insert((0.0, 0, s.los_color.clone()));
        entry_los.0 += s.length_m;
        entry_los.1 += 1;

        let infra_color = match s.infrastructure.as_str() {
            "Segregated Track (wide)" => "#059669",
            "Off Road Path" => "#16a34a",
            "Segregated Track (narrow)" => "#65a30d",
            "Light Segregation" => "#0d9488",
            "Bus / Shared Priority Lane" => "#0284c7",
            "Pedestrian Footway / Zone" => "#2563eb",
            "Shared Footway" => "#d97706",
            "Painted Cycle Lane" => "#ea580c",
            "Steps (Stepped Access)" => "#9333ea",
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

        if matches!(s.grade_multimodal.as_str(), "A" | "B") {
            mm_ab_m += s.length_m;
        }
        if matches!(s.grade_walking.as_str(), "A" | "B") {
            walk_ab_m += s.length_m;
        }
        if matches!(s.grade_wheeling.as_str(), "A" | "B") {
            wheel_ab_m += s.length_m;
        }
        if matches!(s.grade_cycling.as_str(), "A" | "B") {
            cycle_ab_m += s.length_m;
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

    // Suitable = Grade A or B (los_score >= 3)
    let high_len_m = segments
        .iter()
        .filter(|s| s.los_score >= 3)
        .map(|s| s.length_m)
        .sum::<f64>();

    // Hostile / Unsuitable = Grade E (los_score == 0)
    let unsuitable_len_m = segments
        .iter()
        .filter(|s| s.los_score <= 0)
        .map(|s| s.length_m)
        .sum::<f64>();

    let pct_of = |m: f64| -> f64 {
        if total_length_m > 0.0 {
            ((m / total_length_m) * 1000.0).round() / 10.0
        } else {
            0.0
        }
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
        high_med_pct: pct_of(high_len_m),
        unsuitable_pct: pct_of(unsuitable_len_m),
        average_speed_mph,
        mode_suitability: ModeSuitability {
            multimodal_ab_pct: pct_of(mm_ab_m),
            walking_ab_pct: pct_of(walk_ab_m),
            wheeling_ab_pct: pct_of(wheel_ab_m),
            cycling_ab_pct: pct_of(cycle_ab_m),
        },
        junctions,
    }
}
