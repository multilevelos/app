use anyhow::Result;
use multilevelos_core::{LosConfig, LosEngine};
use std::env;
use std::fs;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: multilevelos-cli <path-to-osm-json> [output-geojson]");
        println!("  Example: multilevelos-cli edinburgh.json edinburgh_los.geojson");
        return Ok(());
    }

    let input_path = &args[1];
    println!("Reading OSM data from: {}", input_path);
    let osm_json = fs::read_to_string(input_path)?;

    println!("Initializing Level of Service Engine...");
    let engine = LosEngine::from_overpass_json(&osm_json)?;

    let config = LosConfig::default();
    println!("Running Level of Service analysis...");
    let result = engine.run_analysis(&config)?;

    println!("============================================");
    println!("Multi-Modal Level of Service Summary:");
    println!("  Total Segments: {}", result.segments_count);
    println!("  Total Network Length: {:.2} km", result.summary.total_length_km);
    println!("  Suitable (Grades A-B): {:.1}%", result.summary.high_med_pct);
    println!("  Hostile / Barrier (Grade E): {:.1}%", result.summary.unsuitable_pct);
    println!(
        "  Cross-Modal A-B: MMLoS {:.1}% | Walk {:.1}% | Wheel {:.1}% | Cycle {:.1}%",
        result.summary.mode_suitability.multimodal_ab_pct,
        result.summary.mode_suitability.walking_ab_pct,
        result.summary.mode_suitability.wheeling_ab_pct,
        result.summary.mode_suitability.cycling_ab_pct,
    );
    println!(
        "  Junctions Analyzed: {} ({:.1}% Grades A-B, {} signalized)",
        result.junctions_count,
        result.summary.junctions.suitable_pct,
        result.summary.junctions.signalized_count,
    );
    println!("  Average Speed Limit: {:.1} mph", result.summary.average_speed_mph);
    println!("============================================");

    if args.len() >= 3 {
        let output_path = &args[2];
        let geojson = format_geojson(&result.segments);
        fs::write(output_path, geojson)?;
        println!("Exported GeoJSON to: {}", output_path);
    }

    Ok(())
}

fn format_geojson(segments: &[multilevelos_core::ClassifiedSegment]) -> String {
    use geojson::{Feature, FeatureCollection, Geometry, JsonObject, Value};

    let features: Vec<Feature> = segments
        .iter()
        .map(|s| {
            let coords: Vec<Vec<f64>> = s
                .geometry
                .0
                .iter()
                .map(|c| vec![c.x, c.y])
                .collect();

            let geom = Geometry::new(Value::LineString(coords));
            let mut props = JsonObject::new();
            props.insert("osm_id".to_string(), serde_json::json!(s.osm_id));
            props.insert("name".to_string(), serde_json::json!(s.name));
            props.insert("highway".to_string(), serde_json::json!(s.highway));
            props.insert("infrastructure".to_string(), serde_json::json!(s.infrastructure));
            props.insert("speed_limit_mph".to_string(), serde_json::json!(s.speed_limit_mph));
            props.insert("speed_category".to_string(), serde_json::json!(s.speed_category));
            props.insert("estimated_aadt".to_string(), serde_json::json!(s.estimated_aadt));
            props.insert("aadt_category".to_string(), serde_json::json!(s.aadt_category));
            props.insert("distance_to_road_m".to_string(), serde_json::json!(s.distance_to_road_m));
            props.insert("length_m".to_string(), serde_json::json!(s.length_m));
            props.insert("los_score".to_string(), serde_json::json!(s.los_score));
            props.insert("los_label".to_string(), serde_json::json!(s.los_label));
            props.insert("los_color".to_string(), serde_json::json!(s.los_color));

            Feature {
                bbox: None,
                geometry: Some(geom),
                id: None,
                properties: Some(props),
                foreign_members: None,
            }
        })
        .collect();

    let fc = FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };

    serde_json::to_string_pretty(&fc).unwrap_or_default()
}
