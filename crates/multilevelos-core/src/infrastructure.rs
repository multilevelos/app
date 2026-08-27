use std::collections::HashMap;

/// Infrastructure segregation types according to Scottish Cycling by Design (CbD)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InfrastructureType {
    SegregatedTrackWide,
    OffRoadPath,
    SegregatedTrackNarrow,
    SharedFootway,
    PaintedCycleLane,
    MixedTrafficStreet,
}

impl InfrastructureType {
    pub fn as_str(&self) -> &'static str {
        match self {
            InfrastructureType::SegregatedTrackWide => "Segregated Track (wide)",
            InfrastructureType::OffRoadPath => "Off Road Path",
            InfrastructureType::SegregatedTrackNarrow => "Segregated Track (narrow)",
            InfrastructureType::SharedFootway => "Shared Footway",
            InfrastructureType::PaintedCycleLane => "Painted Cycle Lane",
            InfrastructureType::MixedTrafficStreet => "Mixed Traffic Street",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            InfrastructureType::SegregatedTrackWide => "#054d05", // Dark Green
            InfrastructureType::OffRoadPath => "#3a9120",         // Medium Green
            InfrastructureType::SegregatedTrackNarrow => "#87d668", // Light Green
            InfrastructureType::SharedFootway => "#ffbf00",       // Amber
            InfrastructureType::PaintedCycleLane => "#f97316",    // Orange
            InfrastructureType::MixedTrafficStreet => "#64748b",  // Slate Gray
        }
    }
}

/// Extract and clean width in meters from OSM tags
pub fn extract_width_m(tags: &HashMap<String, String>) -> Option<f64> {
    for key in &["cycleway:width", "width", "cycleway:est_width", "est_width"] {
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

/// Classifies OSM way infrastructure type according to Scottish Cycling by Design (CbD)
pub fn classify_cycle_infrastructure(
    highway: &str,
    tags: &HashMap<String, String>,
    distance_to_road_m: f64,
    min_offroad_distance: f64,
    min_wide_width: f64,
) -> InfrastructureType {
    let width_m = extract_width_m(tags);
    let is_wide = width_m.map(|w| w >= min_wide_width).unwrap_or(false);

    let name = tags.get("name").map(|s| s.as_str()).unwrap_or("");
    let segregated = tags.get("segregated").map(|s| s.as_str()).unwrap_or("");
    let foot = tags.get("foot").map(|s| s.as_str()).unwrap_or("");
    let bicycle = tags.get("bicycle").map(|s| s.as_str()).unwrap_or("");
    let footway_tag = tags.get("footway").map(|s| s.as_str()).unwrap_or("");

    // Look for cycleway sub-tags: cycleway, cycleway:left, cycleway:right, cycleway:both, cycleway:lane, etc.
    let mut cycleway_tags_combined = String::new();
    for (k, v) in tags {
        if k.starts_with("cycleway") {
            cycleway_tags_combined.push_str(v);
            cycleway_tags_combined.push(' ');
        }
    }
    let cycleway_tags = cycleway_tags_combined.to_lowercase();

    // 1. Initial categorization
    let mut initial_type = if highway == "cycleway" {
        "Level track"
    } else if matches!(highway, "footway" | "path" | "pedestrian") {
        if segregated == "yes" {
            "Level track"
        } else {
            "Footway"
        }
    } else {
        "Mixed Traffic Street"
    };

    // 2. Name-based heuristics for off-road trails/towpaths
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

    // 3. Distance to driving road heuristic
    if distance_to_road_m > min_offroad_distance && (initial_type == "Level track" || initial_type == "Footway") {
        initial_type = "Off Road Path";
    }

    // 4. Cycleway tags on driving carriageways
    if initial_type == "Mixed Traffic Street" {
        if cycleway_tags.contains("track")
            || cycleway_tags.contains("separate")
            || tags.get("cycleway:left:segregated").map(|s| s == "yes").unwrap_or(false)
            || tags.get("cycleway:right:segregated").map(|s| s == "yes").unwrap_or(false)
        {
            initial_type = "Light segregation";
        } else if cycleway_tags.contains("lane") || cycleway_tags.contains("share_") || cycleway_tags.contains("opposite") {
            initial_type = "Painted Cycle Lane";
        }
    }

    // 5. Final mapping to InfrastructureType enum
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
        "Level track" | "Light segregation" => {
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
