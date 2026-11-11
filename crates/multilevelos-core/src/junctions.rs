use crate::config::LosConfig;
use crate::los::MmlosGrade;
use crate::osm::{ResolvedOsmNode, ResolvedOsmWay};
use crate::spatial::haversine_distance;
use crate::speed::{is_active_travel_path, is_urban_context, parse_maxspeed_with_context};
use crate::traffic::estimate_traffic_aadt;
use geo::ConvexHull;
use geo_types::{Coord, LineString, MultiPoint, Polygon};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

fn empty_polygon() -> Polygon<f64> {
    Polygon::new(LineString(vec![]), vec![])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedJunction {
    pub junction_id: usize,
    pub name: String,
    pub arm_count: usize,
    pub control_type: String,
    pub is_roundabout: bool,
    pub has_tactile_paving: bool,
    pub max_speed_mph: f64,
    pub max_aadt: u32,
    pub radius_m: f64,
    pub center: (f64, f64),
    pub grade_multimodal: String,
    pub grade_walking: String,
    pub grade_wheeling: String,
    pub grade_cycling: String,
    pub active_grade: String,
    pub los_color: String,
    #[serde(skip, default = "empty_polygon")]
    pub polygon: Polygon<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JunctionSummaryStats {
    pub total_junctions: usize,
    pub signalized_count: usize,
    pub marked_crossing_count: usize,
    pub suitable_pct: f64,
    pub hostile_pct: f64,
    pub grade_breakdown: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
struct ArmInfo {
    highway: String,
    name: Option<String>,
    is_motor: bool,
    is_active: bool,
    is_roundabout: bool,
    lanes: u8,
    speed_mph: f64,
    aadt: u32,
}

#[derive(Debug, Clone)]
struct JunctionSeed {
    coord: (f64, f64),
    radius_m: f64,
    arms: Vec<ArmInfo>,
    is_roundabout: bool,
    has_signals: bool,
    has_marked_crossing: bool,
    has_uncontrolled_crossing: bool,
    has_tactile: bool,
    has_lowered_kerb: bool,
}

fn is_motor_road(hw: &str) -> bool {
    matches!(
        hw,
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

/// Graduated road-class + lane disc radius (`atf-lab/junctionareas` `csis` preset)
fn class_radius_m(hw: &str, lanes: u8) -> f64 {
    let base: f64 = match hw.strip_suffix("_link").unwrap_or(hw) {
        "motorway" | "trunk" => 20.0,
        "primary" => 17.0,
        "secondary" => 14.5,
        "tertiary" => 12.5,
        "residential" | "unclassified" | "living_street" => 10.0,
        _ => 8.5,
    };
    let extra = (lanes.saturating_sub(2) as f64) * 1.5;
    (base + extra).min(24.0)
}

fn parse_lanes(tags: &HashMap<String, String>) -> u8 {
    if let Some(l) = tags.get("lanes").and_then(|s| s.trim().parse::<u8>().ok()) {
        return l.clamp(1, 8);
    }
    let fwd = tags
        .get("lanes:forward")
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(0);
    let bwd = tags
        .get("lanes:backward")
        .and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(0);
    if fwd + bwd > 0 {
        return (fwd + bwd).clamp(1, 8);
    }
    2
}

/// Build polygonized Junction & Crossing areas and score A–E Multimodal Junction LoS
pub fn analyze_junctions(
    ways: &[ResolvedOsmWay],
    tagged_nodes: &[ResolvedOsmNode],
    config: &LosConfig,
) -> (Vec<ClassifiedJunction>, JunctionSummaryStats) {
    let mut node_coords: HashMap<i64, (f64, f64)> = HashMap::new();
    let mut node_arms: HashMap<i64, Vec<ArmInfo>> = HashMap::new();
    let mut node_way_ids: HashMap<i64, HashSet<i64>> = HashMap::new();

    for way in ways {
        let Some(hw) = way.tags.get("highway").map(|s| s.as_str()) else {
            continue;
        };
        let is_motor = is_motor_road(hw);
        let is_active = is_active_travel_path(hw);
        if !is_motor && !is_active {
            continue;
        }

        let urban = is_urban_context(&way.tags);
        let speed_mph = parse_maxspeed_with_context(
            way.tags.get("maxspeed").map(|s| s.as_str()),
            hw,
            urban,
            &config.speed_defaults,
        )
        .unwrap_or(10.0);
        let aadt = estimate_traffic_aadt(hw, &config.aadt_estimates, config.traffic_multiplier)
            .unwrap_or(0);
        let lanes = parse_lanes(&way.tags);
        let is_roundabout = way
            .tags
            .get("junction")
            .map(|j| j == "roundabout" || j == "circular")
            .unwrap_or(false);
        let name = way.tags.get("name").cloned().or_else(|| way.tags.get("ref").cloned());

        let n = way.coords.len().min(way.node_ids.len());
        for idx in 0..n {
            let nid = way.node_ids[idx];
            node_coords.insert(nid, way.coords[idx]);
            node_way_ids.entry(nid).or_default().insert(way.id);

            // Endpoints contribute 1 arm; interior vertices shared with another way contribute 2 arms
            let incident_arms = if idx == 0 || idx == n - 1 { 1 } else { 2 };
            let entry = node_arms.entry(nid).or_default();
            for _ in 0..incident_arms {
                entry.push(ArmInfo {
                    highway: hw.to_string(),
                    name: name.clone(),
                    is_motor,
                    is_active,
                    is_roundabout,
                    lanes,
                    speed_mph,
                    aadt,
                });
            }
        }
    }

    let mut tagged_lookup: HashMap<i64, &ResolvedOsmNode> = HashMap::new();
    for tn in tagged_nodes {
        tagged_lookup.insert(tn.id, tn);
    }

    let mut seeds: Vec<JunctionSeed> = Vec::new();

    for (nid, way_set) in &node_way_ids {
        let Some(&coord) = node_coords.get(nid) else {
            continue;
        };
        let Some(arms) = node_arms.get(nid) else {
            continue;
        };

        let tn = tagged_lookup.get(nid).copied();
        let hw_tag = tn
            .and_then(|n| n.tags.get("highway"))
            .map(|s| s.as_str())
            .unwrap_or("");
        let crossing_tag = tn
            .and_then(|n| n.tags.get("crossing"))
            .map(|s| s.as_str())
            .unwrap_or("");

        let is_tagged_crossing_or_signal = matches!(
            hw_tag,
            "traffic_signals" | "crossing" | "mini_roundabout"
        ) || !crossing_tag.is_empty();

        let has_motor = arms.iter().any(|a| a.is_motor);
        let has_active = arms.iter().any(|a| a.is_active);
        let motor_arm_count = arms.iter().filter(|a| a.is_motor).count();

        // Qualifying junction: >=3 motor arms across >=2 ways, OR active path crossing a motor road, OR explicit signal/crossing on a motor road
        let qualifies = (way_set.len() >= 2 && motor_arm_count >= 3)
            || (way_set.len() >= 2 && has_motor && has_active)
            || (has_motor && is_tagged_crossing_or_signal);

        if !qualifies {
            continue;
        }

        let radius_m = arms
            .iter()
            .map(|a| class_radius_m(&a.highway, a.lanes))
            .fold(9.0_f64, f64::max);

        let has_signals = hw_tag == "traffic_signals"
            || crossing_tag == "traffic_signals"
            || tn
                .and_then(|n| n.tags.get("crossing:signals"))
                .map(|v| v == "yes")
                .unwrap_or(false);

        let has_marked_crossing = matches!(
            crossing_tag,
            "zebra" | "marked" | "pelican" | "puffin" | "toucan" | "pegasus" | "parallel"
        ) || tn
            .and_then(|n| n.tags.get("crossing_ref"))
            .map(|v| !v.is_empty())
            .unwrap_or(false);

        let has_uncontrolled_crossing =
            hw_tag == "crossing" && !has_signals && !has_marked_crossing;

        let has_tactile = tn
            .and_then(|n| n.tags.get("tactile_paving"))
            .map(|v| v == "yes")
            .unwrap_or(false);

        let has_lowered_kerb = tn
            .and_then(|n| n.tags.get("kerb"))
            .map(|v| matches!(v.as_str(), "lowered" | "flush" | "dropped"))
            .unwrap_or(false);

        let is_roundabout =
            hw_tag == "mini_roundabout" || arms.iter().any(|a| a.is_roundabout);

        seeds.push(JunctionSeed {
            coord,
            radius_m,
            arms: arms.clone(),
            is_roundabout,
            has_signals,
            has_marked_crossing,
            has_uncontrolled_crossing,
            has_tactile,
            has_lowered_kerb,
        });
    }

    // Also attach standalone tagged crossing/signal nodes within disc radius of seeds
    for tn in tagged_nodes {
        let hw_tag = tn.tags.get("highway").map(|s| s.as_str()).unwrap_or("");
        let crossing_tag = tn.tags.get("crossing").map(|s| s.as_str()).unwrap_or("");
        if !matches!(hw_tag, "traffic_signals" | "crossing" | "mini_roundabout")
            && crossing_tag.is_empty()
        {
            continue;
        }
        for seed in &mut seeds {
            if haversine_distance(seed.coord, (tn.lon, tn.lat)) <= seed.radius_m + 12.0 {
                if hw_tag == "traffic_signals" || crossing_tag == "traffic_signals" {
                    seed.has_signals = true;
                }
                if matches!(
                    crossing_tag,
                    "zebra" | "marked" | "pelican" | "puffin" | "toucan" | "parallel"
                ) {
                    seed.has_marked_crossing = true;
                }
                if tn.tags.get("tactile_paving").map(|v| v == "yes").unwrap_or(false) {
                    seed.has_tactile = true;
                }
                if tn
                    .tags
                    .get("kerb")
                    .map(|v| matches!(v.as_str(), "lowered" | "flush" | "dropped"))
                    .unwrap_or(false)
                {
                    seed.has_lowered_kerb = true;
                }
            }
        }
    }

    // Greedy bounded touching-disc cluster merge (`atf-lab/junctionareas` bounded merge)
    let mut visited = vec![false; seeds.len()];
    let mut classified = Vec::new();

    for i in 0..seeds.len() {
        if visited[i] {
            continue;
        }
        visited[i] = true;
        let mut cluster_indices = vec![i];
        let anchor = seeds[i].coord;

        for j in (i + 1)..seeds.len() {
            if visited[j] {
                continue;
            }
            let d_anchor = haversine_distance(anchor, seeds[j].coord);
            // Cap cluster diameter at 38m so neighbouring intersections stay distinct
            if d_anchor > 38.0 {
                continue;
            }
            let touches = cluster_indices.iter().any(|&k| {
                let d = haversine_distance(seeds[k].coord, seeds[j].coord);
                d <= (seeds[k].radius_m + seeds[j].radius_m) * 0.85
            });
            if touches {
                visited[j] = true;
                cluster_indices.push(j);
            }
        }

        let junction = build_classified_junction(classified.len() + 1, &cluster_indices, &seeds, config);
        classified.push(junction);
    }

    let summary = summarize_junctions(&classified);
    (classified, summary)
}

fn build_classified_junction(
    junction_id: usize,
    indices: &[usize],
    seeds: &[JunctionSeed],
    config: &LosConfig,
) -> ClassifiedJunction {
    let mut sum_lon = 0.0;
    let mut sum_lat = 0.0;
    let mut max_radius: f64 = 9.0;
    let mut max_speed: f64 = 10.0;
    let mut max_aadt: u32 = 0;
    let mut max_lanes: u8 = 1;
    let mut is_roundabout = false;
    let mut has_signals = false;
    let mut has_marked = false;
    let mut has_uncontrolled = false;
    let mut has_tactile = false;
    let mut has_lowered_kerb = false;

    let mut street_names: Vec<String> = Vec::new();
    let mut total_arms = 0usize;

    for &idx in indices {
        let s = &seeds[idx];
        sum_lon += s.coord.0;
        sum_lat += s.coord.1;
        max_radius = max_radius.max(s.radius_m);
        is_roundabout |= s.is_roundabout;
        has_signals |= s.has_signals;
        has_marked |= s.has_marked_crossing;
        has_uncontrolled |= s.has_uncontrolled_crossing;
        has_tactile |= s.has_tactile;
        has_lowered_kerb |= s.has_lowered_kerb;
        total_arms += s.arms.len();

        for arm in &s.arms {
            if arm.is_motor {
                max_speed = max_speed.max(arm.speed_mph);
                max_aadt = max_aadt.max(arm.aadt);
                max_lanes = max_lanes.max(arm.lanes);
            }
            if let Some(ref nm) = arm.name {
                if !nm.is_empty() && !street_names.contains(nm) && street_names.len() < 2 {
                    street_names.push(nm.clone());
                }
            }
        }
    }

    // Deduplicate internal link arms inside multi-node clusters (`arms - 2*(n-1)`)
    let effective_arms = total_arms
        .saturating_sub(indices.len().saturating_sub(1) * 2)
        .max(3);

    let center = (
        sum_lon / (indices.len() as f64),
        sum_lat / (indices.len() as f64),
    );

    let name = if street_names.len() >= 2 {
        format!("{} / {}", street_names[0], street_names[1])
    } else if street_names.len() == 1 {
        format!("{} Junction", street_names[0])
    } else if is_roundabout {
        format!("Roundabout #{}", junction_id)
    } else {
        format!("Junction #{}", junction_id)
    };

    let control_type = if has_signals {
        "Signal-controlled"
    } else if has_marked {
        "Marked priority crossing"
    } else if is_roundabout {
        "Roundabout"
    } else if has_uncontrolled {
        "Uncontrolled crossing"
    } else {
        "Unmarked priority junction"
    };

    // Score Walking, Wheeling & Cycling Junction LoS (A–E)
    let (grade_walking, grade_wheeling, grade_cycling) =
        if config.standard == crate::config::DesignStandard::Rural2026 {
            crate::rural::score_rural_junction_grades(
                max_speed,
                max_aadt,
                max_lanes,
                is_roundabout,
                has_signals,
                has_marked,
                has_tactile || has_lowered_kerb,
            )
        } else {
            let gw = score_junction_walking(
                max_speed,
                max_aadt,
                effective_arms,
                is_roundabout,
                has_signals,
                has_marked,
            );
            let gwh = score_junction_wheeling(
                gw,
                has_signals || has_marked,
                has_tactile || has_lowered_kerb,
                is_roundabout,
            );
            let gc = score_junction_cycling(
                max_speed,
                max_aadt,
                effective_arms,
                is_roundabout,
                has_signals,
                has_marked,
            );
            (gw, gwh, gc)
        };

    let min_g = grade_walking.min(grade_wheeling).min(grade_cycling).numeric_score();
    let avg_g = ((grade_walking.numeric_score()
        + grade_wheeling.numeric_score()
        + grade_cycling.numeric_score()) as f64
        / 3.0)
        .round() as i8;
    let grade_multimodal = MmlosGrade::from_numeric(avg_g.min(min_g + 1));

    let chosen_grade = match config.active_mode {
        crate::config::TravelMode::Multimodal => grade_multimodal,
        crate::config::TravelMode::Walking => grade_walking,
        crate::config::TravelMode::Wheeling => grade_wheeling,
        crate::config::TravelMode::Cycling => grade_cycling,
    };

    let polygon = build_cluster_polygon(center, indices, seeds);

    ClassifiedJunction {
        junction_id,
        name,
        arm_count: effective_arms,
        control_type: control_type.to_string(),
        is_roundabout,
        has_tactile_paving: has_tactile,
        max_speed_mph: (max_speed * 10.0).round() / 10.0,
        max_aadt,
        radius_m: (max_radius * 10.0).round() / 10.0,
        center,
        grade_multimodal: grade_multimodal.letter().to_string(),
        grade_walking: grade_walking.letter().to_string(),
        grade_wheeling: grade_wheeling.letter().to_string(),
        grade_cycling: grade_cycling.letter().to_string(),
        active_grade: chosen_grade.letter().to_string(),
        los_color: chosen_grade.color().to_string(),
        polygon,
    }
}

fn score_junction_walking(
    max_speed: f64,
    max_aadt: u32,
    arms: usize,
    is_roundabout: bool,
    has_signals: bool,
    has_marked: bool,
) -> MmlosGrade {
    if has_signals {
        if max_speed <= 30.0 {
            return MmlosGrade::A;
        } else if max_speed <= 40.0 {
            return MmlosGrade::B;
        } else {
            return MmlosGrade::C;
        }
    }
    if has_marked {
        if max_speed <= 20.0 {
            return MmlosGrade::A;
        } else if max_speed <= 30.0 {
            return MmlosGrade::B;
        } else {
            return MmlosGrade::D;
        }
    }
    if is_roundabout {
        return if max_aadt < 2000 && max_speed <= 20.0 {
            MmlosGrade::B
        } else if max_aadt < 4000 && max_speed <= 30.0 {
            MmlosGrade::D
        } else {
            MmlosGrade::E
        };
    }

    // Uncontrolled junction
    if max_speed <= 20.0 && max_aadt < 1000 {
        MmlosGrade::A
    } else if max_speed <= 20.0 && max_aadt < 2500 {
        MmlosGrade::B
    } else if max_speed <= 30.0 && max_aadt < 1500 && arms <= 3 {
        MmlosGrade::B
    } else if max_speed <= 30.0 && max_aadt < 3500 {
        MmlosGrade::C
    } else if max_speed <= 30.0 {
        MmlosGrade::D
    } else {
        MmlosGrade::E
    }
}

fn score_junction_wheeling(
    walking: MmlosGrade,
    has_formal_crossing: bool,
    has_kerb_or_tactile: bool,
    is_roundabout: bool,
) -> MmlosGrade {
    let mut g = walking;
    if is_roundabout && !has_formal_crossing {
        g = g.min(MmlosGrade::D);
    }
    if !has_formal_crossing && !has_kerb_or_tactile && g == MmlosGrade::A {
        // Unverified dropped kerbs at unmarked side streets cap Wheeling at B
        g = MmlosGrade::B;
    }
    g
}

fn score_junction_cycling(
    max_speed: f64,
    max_aadt: u32,
    arms: usize,
    is_roundabout: bool,
    has_signals: bool,
    has_marked: bool,
) -> MmlosGrade {
    // LTN 1/20 Ch 10: unsegregated multi-lane/busy roundabouts are high-risk collision points for cyclists
    if is_roundabout {
        return if max_speed <= 20.0 && max_aadt < 2000 {
            MmlosGrade::B
        } else if has_signals && max_speed <= 30.0 {
            MmlosGrade::C
        } else {
            MmlosGrade::E
        };
    }

    if has_signals {
        return if max_speed <= 20.0 {
            MmlosGrade::A
        } else if max_speed <= 30.0 && max_aadt < 6000 {
            MmlosGrade::B
        } else if max_speed <= 40.0 {
            MmlosGrade::C
        } else {
            MmlosGrade::D
        };
    }

    if has_marked && max_speed <= 30.0 {
        return MmlosGrade::B;
    }

    if max_speed <= 20.0 && max_aadt < 2000 {
        MmlosGrade::A
    } else if max_speed <= 30.0 && max_aadt < 1500 && arms <= 3 {
        MmlosGrade::B
    } else if max_speed <= 30.0 && max_aadt < 3000 {
        MmlosGrade::C
    } else if max_speed <= 30.0 && max_aadt < 5000 {
        MmlosGrade::D
    } else {
        MmlosGrade::E
    }
}

/// Generate smooth buffered convex-hull polygon around merged junction discs in local metric plane
fn build_cluster_polygon(
    center: (f64, f64),
    indices: &[usize],
    seeds: &[JunctionSeed],
) -> Polygon<f64> {
    let lat_rad = center.1.to_radians();
    let m_per_deg_lat =
        111_132.954 - 559.822 * (2.0 * lat_rad).cos() + 1.175 * (4.0 * lat_rad).cos();
    let m_per_deg_lon = 111_412.84 * lat_rad.cos() - 93.5 * (3.0 * lat_rad).cos();

    // Sample 12-gon perimeter points around each disc in local meters, then take convex hull
    let mut metric_pts: Vec<Coord<f64>> = Vec::with_capacity(indices.len() * 12);
    for &idx in indices {
        let s = &seeds[idx];
        let cx = (s.coord.0 - center.0) * m_per_deg_lon;
        let cy = (s.coord.1 - center.1) * m_per_deg_lat;
        let r = s.radius_m;

        for k in 0..12 {
            let theta = (k as f64) * std::f64::consts::TAU / 12.0;
            metric_pts.push(Coord {
                x: cx + r * theta.cos(),
                y: cy + r * theta.sin(),
            });
        }
    }

    let mp = MultiPoint::from(metric_pts);
    let hull = mp.convex_hull();

    let deg_ring: Vec<Coord<f64>> = hull
        .exterior()
        .0
        .iter()
        .map(|c| Coord {
            x: center.0 + c.x / m_per_deg_lon,
            y: center.1 + c.y / m_per_deg_lat,
        })
        .collect();

    Polygon::new(LineString(deg_ring), vec![])
}

fn summarize_junctions(junctions: &[ClassifiedJunction]) -> JunctionSummaryStats {
    let total = junctions.len();
    if total == 0 {
        return JunctionSummaryStats::default();
    }

    let mut signalized = 0usize;
    let mut marked = 0usize;
    let mut suitable = 0usize;
    let mut hostile = 0usize;
    let mut grade_breakdown: HashMap<String, usize> = HashMap::new();

    for j in junctions {
        if j.control_type == "Signal-controlled" {
            signalized += 1;
        } else if j.control_type == "Marked priority crossing" {
            marked += 1;
        }
        if j.active_grade == "A" || j.active_grade == "B" {
            suitable += 1;
        }
        if j.active_grade == "E" {
            hostile += 1;
        }
        *grade_breakdown.entry(j.active_grade.clone()).or_insert(0) += 1;
    }

    JunctionSummaryStats {
        total_junctions: total,
        signalized_count: signalized,
        marked_crossing_count: marked,
        suitable_pct: ((suitable as f64 / total as f64) * 1000.0).round() / 10.0,
        hostile_pct: ((hostile as f64 / total as f64) * 1000.0).round() / 10.0,
        grade_breakdown,
    }
}
