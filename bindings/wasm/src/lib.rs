use geojson::{Feature, FeatureCollection, Geometry, JsonObject, Value};
use multilevelos_core::{ClassifiedJunction, ClassifiedSegment, LosConfig, LosEngine};
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
    /// Creates a new MMLoS engine by parsing Overpass OSM JSON
    #[wasm_bindgen(constructor)]
    pub fn new(osm_json: &str) -> Result<WasmLosEngine, JsValue> {
        let engine = LosEngine::from_overpass_json(osm_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse OSM data: {e}")))?;
        Ok(WasmLosEngine { engine })
    }

    /// Computes Multi-Modal Level of Service (segments + junctions) and returns GeoJSON and summary metrics
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
        let junctions_geojson = format_junctions_geojson(&result.junctions);

        let output = serde_json::json!({
            "segments_count": result.segments_count,
            "junctions_count": result.junctions_count,
            "summary": result.summary,
            "geojson": geojson,
            "junctions_geojson": junctions_geojson,
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
            let coords: Vec<Vec<f64>> = s.geometry.0.iter().map(|c| vec![c.x, c.y]).collect();
            let geom = Geometry::new(Value::LineString(coords));
            let mut props = JsonObject::new();
            props.insert("layer_type".to_string(), serde_json::json!("segment"));
            props.insert("osm_id".to_string(), serde_json::json!(s.osm_id));
            props.insert("name".to_string(), serde_json::json!(s.name));
            props.insert("highway".to_string(), serde_json::json!(s.highway));
            props.insert("infrastructure".to_string(), serde_json::json!(s.infrastructure));
            props.insert(
                "walking_category".to_string(),
                serde_json::json!(s.walking_category),
            );
            props.insert(
                "cycling_category".to_string(),
                serde_json::json!(s.cycling_category),
            );
            props.insert(
                "prow_designation".to_string(),
                serde_json::json!(s.prow_designation),
            );
            props.insert(
                "rural_surface".to_string(),
                serde_json::json!(s.rural_surface),
            );
            props.insert("separation".to_string(), serde_json::json!(s.separation));
            props.insert("surface".to_string(), serde_json::json!(s.surface));
            props.insert("width_m".to_string(), serde_json::json!(s.width_m));
            props.insert("incline_pct".to_string(), serde_json::json!(s.incline_pct));
            props.insert("is_lit".to_string(), serde_json::json!(s.is_lit));
            props.insert("is_urban".to_string(), serde_json::json!(s.is_urban));
            props.insert("speed_limit_mph".to_string(), serde_json::json!(s.speed_limit_mph));
            props.insert("speed_category".to_string(), serde_json::json!(s.speed_category));
            props.insert("estimated_aadt".to_string(), serde_json::json!(s.estimated_aadt));
            props.insert("aadt_category".to_string(), serde_json::json!(s.aadt_category));
            props.insert("distance_to_road_m".to_string(), serde_json::json!(s.distance_to_road_m));
            props.insert("length_m".to_string(), serde_json::json!(s.length_m));
            props.insert("grade_multimodal".to_string(), serde_json::json!(s.grade_multimodal));
            props.insert("grade_walking".to_string(), serde_json::json!(s.grade_walking));
            props.insert("grade_wheeling".to_string(), serde_json::json!(s.grade_wheeling));
            props.insert("grade_cycling".to_string(), serde_json::json!(s.grade_cycling));
            props.insert("active_grade".to_string(), serde_json::json!(s.active_grade));
            props.insert("cbd_label".to_string(), serde_json::json!(s.cbd_label));
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

fn format_junctions_geojson(junctions: &[ClassifiedJunction]) -> FeatureCollection {
    let features: Vec<Feature> = junctions
        .iter()
        .map(|j| {
            let exterior: Vec<Vec<f64>> = j
                .polygon
                .exterior()
                .0
                .iter()
                .map(|c| vec![c.x, c.y])
                .collect();

            let geom = Geometry::new(Value::Polygon(vec![exterior]));
            let mut props = JsonObject::new();
            props.insert("layer_type".to_string(), serde_json::json!("junction"));
            props.insert("junction_id".to_string(), serde_json::json!(j.junction_id));
            props.insert("name".to_string(), serde_json::json!(j.name));
            props.insert("arm_count".to_string(), serde_json::json!(j.arm_count));
            props.insert("control_type".to_string(), serde_json::json!(j.control_type));
            props.insert("is_roundabout".to_string(), serde_json::json!(j.is_roundabout));
            props.insert(
                "has_tactile_paving".to_string(),
                serde_json::json!(j.has_tactile_paving),
            );
            props.insert("max_speed_mph".to_string(), serde_json::json!(j.max_speed_mph));
            props.insert("max_aadt".to_string(), serde_json::json!(j.max_aadt));
            props.insert("radius_m".to_string(), serde_json::json!(j.radius_m));
            props.insert("center_lng".to_string(), serde_json::json!(j.center.0));
            props.insert("center_lat".to_string(), serde_json::json!(j.center.1));
            props.insert("grade_multimodal".to_string(), serde_json::json!(j.grade_multimodal));
            props.insert("grade_walking".to_string(), serde_json::json!(j.grade_walking));
            props.insert("grade_wheeling".to_string(), serde_json::json!(j.grade_wheeling));
            props.insert("grade_cycling".to_string(), serde_json::json!(j.grade_cycling));
            props.insert("active_grade".to_string(), serde_json::json!(j.active_grade));
            props.insert("los_color".to_string(), serde_json::json!(j.los_color));

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
