use std::collections::HashMap;

/// Infer whether an OSM way is in a built-up urban environment (`atf-lab/osmactive` `urban.csv` rules)
pub fn is_urban_context(tags: &HashMap<String, String>) -> bool {
    if let Some(lit) = tags.get("lit") {
        if matches!(lit.as_str(), "yes" | "24/7" | "automatic") {
            return true;
        }
    }
    for key in &["sidewalk", "sidewalk:both", "sidewalk:left", "sidewalk:right"] {
        if let Some(val) = tags.get(*key) {
            let v = val.trim().to_lowercase();
            if !v.is_empty() && v != "no" && v != "none" {
                return true;
            }
        }
    }
    if let Some(hw) = tags.get("highway") {
        if matches!(hw.as_str(), "residential" | "living_street" | "pedestrian") {
            return true;
        }
    }
    false
}

/// Parses a maxspeed tag string into miles per hour (mph), applying urban/rural-aware defaults
pub fn parse_maxspeed_with_context(
    raw_maxspeed: Option<&str>,
    highway: &str,
    is_urban: bool,
    defaults: &HashMap<String, f64>,
) -> Option<f64> {
    if is_active_travel_path(highway) {
        return None;
    }

    if let Some(raw) = raw_maxspeed {
        let trimmed = raw.trim().to_lowercase();

        if trimmed == "national" || trimmed == "gb:nsl_single" {
            return Some(if highway == "motorway" || highway == "motorway_link" {
                70.0
            } else {
                60.0
            });
        }
        if trimmed == "gb:nsl_dual" || trimmed == "gb:motorway" {
            return Some(70.0);
        }
        if trimmed == "gb:nsl_restricted" || trimmed == "walk" {
            return Some(if trimmed == "walk" { 10.0 } else { 30.0 });
        }

        if trimmed.ends_with("mph") {
            let num_part = trimmed.trim_end_matches("mph").trim();
            if let Ok(v) = num_part.parse::<f64>() {
                return Some(v);
            }
        }

        if trimmed.ends_with("km/h") || trimmed.ends_with("kph") {
            let num_part = trimmed
                .trim_end_matches("km/h")
                .trim_end_matches("kph")
                .trim();
            if let Ok(kph) = num_part.parse::<f64>() {
                return Some((kph * 0.621371 * 10.0).round() / 10.0);
            }
        }

        if let Ok(v) = trimmed.parse::<f64>() {
            if v > 70.0 {
                return Some((v * 0.621371 * 10.0).round() / 10.0);
            }
            return Some(v);
        }
    }

    // Rural untagged unclassified/road links default to 60 mph per `atf-lab/osmactive/data/speeds.csv`
    if !is_urban && matches!(highway, "unclassified" | "road") {
        return Some(60.0);
    }

    defaults.get(highway).copied().or_else(|| {
        Some(match highway {
            "motorway" | "motorway_link" => 70.0,
            "trunk" | "trunk_link" => 60.0,
            "primary" | "primary_link" => 40.0,
            "secondary" | "secondary_link" => 40.0,
            "tertiary" | "tertiary_link" => 40.0,
            "residential" | "unclassified" => 30.0,
            "living_street" | "service" => 10.0,
            _ => 30.0,
        })
    })
}

/// Backwards-compatible maxspeed parser (assumes urban defaults when context tags are unavailable)
pub fn parse_maxspeed(
    raw_maxspeed: Option<&str>,
    highway: &str,
    defaults: &HashMap<String, f64>,
) -> Option<f64> {
    parse_maxspeed_with_context(raw_maxspeed, highway, true, defaults)
}

/// Classify numeric speed in mph into standard speed bands
pub fn classify_speed_category(speed_mph: f64) -> &'static str {
    if speed_mph < 20.0 {
        "<20 mph"
    } else if speed_mph < 30.0 {
        "20 mph"
    } else if speed_mph < 40.0 {
        "30 mph"
    } else if speed_mph < 50.0 {
        "40 mph"
    } else if speed_mph < 60.0 {
        "50 mph"
    } else {
        "60+ mph"
    }
}

/// Check if highway is a dedicated off-carriageway active travel path
pub fn is_active_travel_path(highway: &str) -> bool {
    matches!(
        highway,
        "cycleway" | "footway" | "path" | "pedestrian" | "steps" | "bridleway" | "track"
    )
}
