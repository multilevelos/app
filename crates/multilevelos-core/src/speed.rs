use std::collections::HashMap;

/// Parses a maxspeed tag string into miles per hour (mph), applying fallback defaults for untagged roads
pub fn parse_maxspeed(
    raw_maxspeed: Option<&str>,
    highway: &str,
    defaults: &HashMap<String, f64>,
) -> Option<f64> {
    // Non-traffic active travel ways do not have a motor vehicle speed limit
    if is_active_travel_path(highway) {
        return None;
    }

    if let Some(raw) = raw_maxspeed {
        let trimmed = raw.trim().to_lowercase();

        if trimmed == "national" {
            if highway == "motorway" || highway == "motorway_link" {
                return Some(70.0);
            } else {
                return Some(60.0);
            }
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
                // Convert kph to mph: 1 km/h ≈ 0.621371 mph
                return Some(kph * 0.621371);
            }
        }

        // Plain numeric string
        if let Ok(v) = trimmed.parse::<f64>() {
            // Assume mph if in typical UK range or <= 70, otherwise if > 70 it's likely km/h
            if v > 70.0 {
                return Some(v * 0.621371);
            }
            return Some(v);
        }
    }

    // Use default speed for highway type
    defaults.get(highway).copied().or_else(|| {
        Some(match highway {
            "motorway" | "motorway_link" => 70.0,
            "trunk" | "trunk_link" => 60.0,
            "primary" | "primary_link" => 40.0,
            "secondary" | "secondary_link" => 40.0,
            "tertiary" | "tertiary_link" => 40.0,
            "residential" | "unclassified" => 30.0,
            "living_street" => 10.0,
            "service" => 10.0,
            _ => 30.0,
        })
    })
}

/// Classify numeric speed in mph into standard Cycling by Design (CbD) speed categories
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
        "cycleway" | "footway" | "path" | "pedestrian" | "steps" | "bridleway"
    )
}
