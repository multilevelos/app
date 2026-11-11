use std::collections::HashMap;

/// Infrastructure segregation & street facility types across LTN 1/20, CbD, Walking & Wheeling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InfrastructureType {
    SegregatedTrackWide,
    OffRoadPath,
    SegregatedTrackNarrow,
    LightSegregation,
    BusLane,
    SharedFootway,
    PedestrianFootway,
    PaintedCycleLane,
    MixedTrafficStreet,
    Steps,
}

impl InfrastructureType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InfrastructureType::SegregatedTrackWide => "Segregated Track (wide)",
            InfrastructureType::OffRoadPath => "Off Road Path",
            InfrastructureType::SegregatedTrackNarrow => "Segregated Track (narrow)",
            InfrastructureType::LightSegregation => "Light Segregation",
            InfrastructureType::BusLane => "Bus / Shared Priority Lane",
            InfrastructureType::SharedFootway => "Shared Footway",
            InfrastructureType::PedestrianFootway => "Pedestrian Footway / Zone",
            InfrastructureType::PaintedCycleLane => "Painted Cycle Lane",
            InfrastructureType::MixedTrafficStreet => "Mixed Traffic Street",
            InfrastructureType::Steps => "Steps (Stepped Access)",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            InfrastructureType::SegregatedTrackWide => "#059669",
            InfrastructureType::OffRoadPath => "#16a34a",
            InfrastructureType::SegregatedTrackNarrow => "#65a30d",
            InfrastructureType::LightSegregation => "#0d9488",
            InfrastructureType::BusLane => "#0284c7",
            InfrastructureType::PedestrianFootway => "#2563eb",
            InfrastructureType::SharedFootway => "#d97706",
            InfrastructureType::PaintedCycleLane => "#ea580c",
            InfrastructureType::MixedTrafficStreet => "#64748b",
            InfrastructureType::Steps => "#9333ea",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SurfaceQuality {
    Smooth,
    Moderate,
    Rough,
    Unpaved,
}

impl SurfaceQuality {
    pub fn as_str(&self) -> &'static str {
        match self {
            SurfaceQuality::Smooth => "Smooth paved",
            SurfaceQuality::Moderate => "Paved / flags",
            SurfaceQuality::Rough => "Rough / setts",
            SurfaceQuality::Unpaved => "Unpaved / loose",
        }
    }
}

/// Extract physical separation class per `atf-lab/osmactive/data/separation.csv`
pub fn extract_separation(tags: &HashMap<String, String>) -> &'static str {
    for (k, v) in tags {
        if k.contains("separation") || k.starts_with("cycleway") && k.ends_with("protection") {
            let val = v.trim().to_lowercase();
            if val.contains("guard_rail")
                || val.contains("fence")
                || val.contains("jersey_barrier")
                || val.contains("barrier")
                || val.contains("wall")
            {
                return "barrier";
            }
            if val.contains("verge") || val.contains("tree") || val.contains("hedge") {
                return "verge";
            }
            if val.contains("kerb") || val.contains("curb") {
                return "kerb";
            }
            if val.contains("parking") {
                return "parking";
            }
            if val.contains("flex_post")
                || val.contains("bollard")
                || val.contains("wand")
                || val.contains("armadillo")
                || val.contains("planter")
                || val.contains("bump")
                || val.contains("studs")
                || val.contains("delineator")
            {
                return "light";
            }
            if val.contains("line") || val.contains("busway") {
                return "paint";
            }
        }
    }
    "none"
}

/// Extract surface quality for Walking, Wheeling, and Cycling
pub fn extract_surface_quality(tags: &HashMap<String, String>) -> SurfaceQuality {
    if let Some(sm) = tags.get("smoothness") {
        match sm.as_str() {
            "excellent" | "good" => return SurfaceQuality::Smooth,
            "intermediate" => return SurfaceQuality::Moderate,
            "bad" => return SurfaceQuality::Rough,
            "very_bad" | "horrible" | "very_horrible" | "impassable" => {
                return SurfaceQuality::Unpaved
            }
            _ => {}
        }
    }

    if let Some(surf) = tags.get("surface") {
        let s = surf.trim().to_lowercase();
        if matches!(
            s.as_str(),
            "asphalt" | "concrete" | "paved" | "tartan" | "concrete:plates"
        ) {
            return SurfaceQuality::Smooth;
        }
        if matches!(
            s.as_str(),
            "paving_stones" | "compacted" | "fine_gravel" | "brick" | "wood" | "metal"
        ) {
            return SurfaceQuality::Moderate;
        }
        if matches!(s.as_str(), "sett" | "cobblestone" | "unhewn_cobblestone") {
            return SurfaceQuality::Rough;
        }
        if matches!(
            s.as_str(),
            "gravel" | "pebblestone" | "ground" | "dirt" | "earth" | "grass" | "mud" | "sand" | "unpaved"
        ) {
            return SurfaceQuality::Unpaved;
        }
    }

    // Default urban highway assumption
    if let Some(hw) = tags.get("highway") {
        if matches!(hw.as_str(), "path" | "bridleway" | "track") {
            return SurfaceQuality::Moderate;
        }
    }

    SurfaceQuality::Smooth
}

/// Parse longitudinal gradient (`incline` tag) as absolute percentage
pub fn extract_incline_pct(tags: &HashMap<String, String>) -> Option<f64> {
    let raw = tags.get("incline")?.trim().to_lowercase();
    if raw.is_empty() {
        return None;
    }
    if raw == "up" || raw == "down" {
        return Some(4.0);
    }
    if raw == "steep" {
        return Some(10.0);
    }
    if let Some(deg_str) = raw.strip_suffix('°') {
        if let Ok(deg) = deg_str.trim().parse::<f64>() {
            return Some((deg.abs().to_radians().tan() * 100.0 * 10.0).round() / 10.0);
        }
    }
    let clean = raw.trim_end_matches('%').trim();
    if let Ok(pct) = clean.parse::<f64>() {
        return Some((pct.abs() * 10.0).round() / 10.0);
    }
    None
}

/// Extract and clean width in meters from OSM tags
pub fn extract_width_m(tags: &HashMap<String, String>) -> Option<f64> {
    for key in &[
        "cycleway:width",
        "footway:width",
        "sidewalk:width",
        "width",
        "cycleway:est_width",
        "est_width",
    ] {
        if let Some(val) = tags.get(*key) {
            let clean = val.trim().trim_end_matches('m').trim();
            if let Ok(w) = clean.parse::<f64>() {
                if w > 0.0 {
                    return Some(w);
                }
            }
        }
    }
    None
}

/// Determine whether a road carriageway explicitly has or lacks a pedestrian sidewalk
pub fn sidewalk_status(tags: &HashMap<String, String>) -> Option<bool> {
    for key in &["sidewalk", "sidewalk:both", "sidewalk:left", "sidewalk:right"] {
        if let Some(val) = tags.get(*key) {
            let v = val.trim().to_lowercase();
            if v == "no" || v == "none" {
                return Some(false);
            }
            if matches!(v.as_str(), "both" | "left" | "right" | "yes" | "separate") {
                return Some(true);
            }
        }
    }
    None
}

/// Classifies OSM way infrastructure type across LTN 1/20, CbD, Walking & Wheeling
pub fn classify_cycle_infrastructure(
    highway: &str,
    tags: &HashMap<String, String>,
    distance_to_road_m: f64,
    min_offroad_distance: f64,
    min_wide_width: f64,
) -> InfrastructureType {
    if highway == "steps" {
        return InfrastructureType::Steps;
    }

    let width_m = extract_width_m(tags);
    let is_wide = width_m.map(|w| w >= min_wide_width).unwrap_or(false);
    let separation = extract_separation(tags);

    let name = tags.get("name").map(|s| s.as_str()).unwrap_or("");
    let segregated = tags.get("segregated").map(|s| s.as_str()).unwrap_or("");
    let foot = tags.get("foot").map(|s| s.as_str()).unwrap_or("");
    let bicycle = tags.get("bicycle").map(|s| s.as_str()).unwrap_or("");
    let footway_tag = tags.get("footway").map(|s| s.as_str()).unwrap_or("");

    // Pedestrian-only footways / zones (no cycling designation)
    if matches!(highway, "footway" | "pedestrian")
        && !matches!(bicycle, "yes" | "designated" | "permissive")
        && segregated != "yes"
    {
        if distance_to_road_m > min_offroad_distance && highway != "footway" {
            return InfrastructureType::OffRoadPath;
        }
        return InfrastructureType::PedestrianFootway;
    }

    let mut cycleway_tags_combined = String::new();
    for (k, v) in tags {
        if k.starts_with("cycleway") || k.starts_with("busway") || k == "lanes:bus" {
            cycleway_tags_combined.push_str(v);
            cycleway_tags_combined.push(' ');
        }
    }
    let cycleway_tags = cycleway_tags_combined.to_lowercase();

    if separation == "light" {
        return InfrastructureType::LightSegregation;
    }

    let mut initial_type = if highway == "cycleway" {
        "Level track"
    } else if matches!(highway, "footway" | "path" | "pedestrian" | "bridleway") {
        if segregated == "yes" {
            "Level track"
        } else {
            "Footway"
        }
    } else {
        "Mixed Traffic Street"
    };

    let is_named_trail = name.contains("Path")
        || name.contains("Towpath")
        || name.contains("Railway")
        || name.contains("Trail")
        || name.contains("Greenway")
        || name.contains("Walkway")
        || name.contains("Avenue");

    if is_named_trail && initial_type == "Level track" {
        initial_type = "Off Road Path";
    }

    if distance_to_road_m > min_offroad_distance
        && (initial_type == "Level track" || initial_type == "Footway")
    {
        initial_type = "Off Road Path";
    }

    if initial_type == "Mixed Traffic Street" {
        if cycleway_tags.contains("track")
            || cycleway_tags.contains("separate")
            || tags
                .get("cycleway:left:segregated")
                .map(|s| s == "yes")
                .unwrap_or(false)
            || tags
                .get("cycleway:right:segregated")
                .map(|s| s == "yes")
                .unwrap_or(false)
        {
            if separation == "light" {
                return InfrastructureType::LightSegregation;
            }
            initial_type = "Level track";
        } else if cycleway_tags.contains("share_busway") || highway == "busway" {
            return InfrastructureType::BusLane;
        } else if cycleway_tags.contains("lane")
            || cycleway_tags.contains("share_")
            || cycleway_tags.contains("opposite")
        {
            initial_type = "Painted Cycle Lane";
        }
    }

    if highway == "cycleway" && footway_tag.eq_ignore_ascii_case("sidewalk") {
        return if distance_to_road_m > min_offroad_distance {
            InfrastructureType::OffRoadPath
        } else {
            InfrastructureType::SharedFootway
        };
    }

    if highway == "cycleway" && foot == "designated" && segregated != "yes" {
        return if distance_to_road_m > min_offroad_distance {
            InfrastructureType::OffRoadPath
        } else {
            InfrastructureType::SharedFootway
        };
    }

    if matches!(highway, "cycleway" | "footway" | "pedestrian")
        && bicycle == "designated"
        && segregated != "yes"
        && !footway_tag.eq_ignore_ascii_case("sidewalk")
    {
        return InfrastructureType::OffRoadPath;
    }

    if highway == "cycleway" && foot == "yes" && bicycle != "designated" && segregated != "yes" {
        return if distance_to_road_m > min_offroad_distance {
            InfrastructureType::OffRoadPath
        } else {
            InfrastructureType::SharedFootway
        };
    }

    match initial_type {
        "Off Road Path" => InfrastructureType::OffRoadPath,
        "Level track" => {
            if is_wide {
                InfrastructureType::SegregatedTrackWide
            } else {
                InfrastructureType::SegregatedTrackNarrow
            }
        }
        "Painted Cycle Lane" => InfrastructureType::PaintedCycleLane,
        "Footway" => {
            if distance_to_road_m > min_offroad_distance {
                InfrastructureType::OffRoadPath
            } else {
                InfrastructureType::SharedFootway
            }
        }
        _ => InfrastructureType::MixedTrafficStreet,
    }
}
