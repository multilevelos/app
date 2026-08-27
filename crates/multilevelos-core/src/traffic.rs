use std::collections::HashMap;

/// Returns default road class-based Annual Average Daily Traffic (AADT) estimates
pub fn default_aadt_estimates() -> HashMap<String, u32> {
    let mut map = HashMap::new();
    map.insert("motorway".to_string(), 20000);
    map.insert("motorway_link".to_string(), 20000);
    map.insert("trunk".to_string(), 8000);
    map.insert("trunk_link".to_string(), 8000);
    map.insert("primary".to_string(), 6000);
    map.insert("primary_link".to_string(), 6000);
    map.insert("secondary".to_string(), 5000);
    map.insert("secondary_link".to_string(), 5000);
    map.insert("tertiary".to_string(), 3000);
    map.insert("tertiary_link".to_string(), 3000);
    map.insert("residential".to_string(), 500);
    map.insert("service".to_string(), 500);
    map.insert("unclassified".to_string(), 500);
    map.insert("living_street".to_string(), 500);
    map
}

/// Estimate AADT traffic volume based on highway class and custom configuration
pub fn estimate_traffic_aadt(
    highway: &str,
    custom_estimates: &HashMap<String, u32>,
    multiplier: f64,
) -> Option<u32> {
    if matches!(
        highway,
        "cycleway" | "footway" | "path" | "pedestrian" | "steps" | "bridleway" | "track"
    ) {
        return None;
    }

    let base_aadt = custom_estimates.get(highway).copied().or_else(|| {
        match highway {
            "motorway" | "motorway_link" => Some(20000),
            "trunk" | "trunk_link" => Some(8000),
            "primary" | "primary_link" => Some(6000),
            "secondary" | "secondary_link" => Some(5000),
            "tertiary" | "tertiary_link" => Some(3000),
            "residential" | "service" | "unclassified" | "living_street" => Some(500),
            _ => Some(500),
        }
    });

    base_aadt.map(|v| ((v as f64) * multiplier).round().max(0.0) as u32)
}

/// Classifies numeric AADT into CbD AADT character ranges
pub fn classify_aadt_category(aadt: Option<u32>) -> &'static str {
    match aadt {
        None => "0 to 999",
        Some(v) if v < 1000 => "0 to 999",
        Some(v) if v < 2000 => "1000 to 1999",
        Some(v) if v < 4000 => "2000 to 3999",
        Some(_) => "4000+",
    }
}
