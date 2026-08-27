use geojson::{Feature, FeatureCollection, Geometry, JsonObject, Value};
use multilevelos_core::{ClassifiedSegment, LosConfig, LosEngine};
use std::sync::Once;
use wasm_bindgen::prelude::*;

static INIT_LOG: Once = Once::new();

#[wasm_bindgen(start)]
pub fn init() {
    INIT_LOG.call_once(|| {
        console_error_panic_hook::set_once();
        let _ = console_log::init_with_level(log::Level::Info);
    });
}

#[wasm_bindgen]
pub struct WasmLosEngine {
    engine: LosEngine,
}

#[wasm_bindgen]
impl WasmLosEngine {
    /// Creates a new LoS engine by parsing Overpass OSM JSON
    #[wasm_bindgen(constructor)]
    pub fn new(osm_json: &str) -> Result<WasmLosEngine, JsValue> {
        let engine = LosEngine::from_overpass_json(osm_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse OSM data: {e}")))?;
        Ok(WasmLosEngine { engine })
    }

    /// Computes Level of Service and returns GeoJSON and summary metrics
    #[wasm_bindgen(js_name = calculateLos)]
    pub fn calculate_los(&self, config_json: &str) -> Result<String, JsValue> {
        let config: LosConfig = if config_json.trim().is_empty() || config_json == "{}" {
            LosConfig::default()
        } else {
            serde_json::from_str(config_json)
                .map_err(|e| JsValue::from_str(&format!("Invalid configuration JSON: {e}")))?
        };

        let result = self
            .engine
            .run_analysis(&config)
            .map_err(|e| JsValue::from_str(&format!("LoS analysis failed: {e}")))?;

        let geojson = format_segments_geojson(&result.segments);

        let output = serde_json::json!({
            "segments_count": result.segments_count,
            "summary": result.summary,
            "geojson": geojson,
        });

        serde_json::to_string(&output)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
    }

    /// Returns default JSON configuration string
    #[wasm_bindgen(js_name = getDefaultConfig)]
    pub fn get_default_config() -> String {
        serde_json::to_string_pretty(&LosConfig::default()).unwrap_or_else(|_| "{}".to_string())
    }
}

fn format_segments_geojson(segments: &[ClassifiedSegment]) -> FeatureCollection {
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

    FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    }
}
