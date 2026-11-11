use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TravelMode {
    #[default]
    Multimodal,
    Walking,
    Wheeling,
    Cycling,
}

impl TravelMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TravelMode::Multimodal => "multimodal",
            TravelMode::Walking => "walking",
            TravelMode::Wheeling => "wheeling",
            TravelMode::Cycling => "cycling",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DesignStandard {
    #[default]
    Ltn120,
    Rural2026,
    Cbd,
}

impl DesignStandard {
    pub fn as_str(&self) -> &'static str {
        match self {
            DesignStandard::Ltn120 => "ltn120",
            DesignStandard::Rural2026 => "rural2026",
            DesignStandard::Cbd => "cbd",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LosConfig {
    #[serde(default)]
    pub active_mode: TravelMode,

    #[serde(default)]
    pub standard: DesignStandard,

    #[serde(default = "default_true")]
    pub include_junctions: bool,

    #[serde(default = "default_true")]
    pub include_mixed_traffic: bool,

    #[serde(default = "default_min_offroad_distance")]
    pub min_offroad_distance: f64,

    #[serde(default = "default_min_wide_width")]
    pub min_wide_width: f64,

    #[serde(default = "default_traffic_multiplier")]
    pub traffic_multiplier: f64,

    #[serde(default = "default_aadt_map")]
    pub aadt_estimates: HashMap<String, u32>,

    #[serde(default = "default_speed_map")]
    pub speed_defaults: HashMap<String, f64>,
}

fn default_true() -> bool {
    true
}

fn default_min_offroad_distance() -> f64 {
    10.0
}

fn default_min_wide_width() -> f64 {
    2.0
}

fn default_traffic_multiplier() -> f64 {
    1.0
}

fn default_aadt_map() -> HashMap<String, u32> {
    crate::traffic::default_aadt_estimates()
}

fn default_speed_map() -> HashMap<String, f64> {
    let mut map = HashMap::new();
    map.insert("motorway".to_string(), 70.0);
    map.insert("motorway_link".to_string(), 70.0);
    map.insert("trunk".to_string(), 60.0);
    map.insert("trunk_link".to_string(), 60.0);
    map.insert("primary".to_string(), 40.0);
    map.insert("primary_link".to_string(), 40.0);
    map.insert("secondary".to_string(), 40.0);
    map.insert("secondary_link".to_string(), 40.0);
    map.insert("tertiary".to_string(), 40.0);
    map.insert("tertiary_link".to_string(), 40.0);
    map.insert("residential".to_string(), 30.0);
    map.insert("unclassified".to_string(), 30.0);
    map.insert("living_street".to_string(), 10.0);
    map.insert("service".to_string(), 10.0);
    map
}

impl Default for LosConfig {
    fn default() -> Self {
        Self {
            active_mode: TravelMode::default(),
            standard: DesignStandard::default(),
            include_junctions: true,
            include_mixed_traffic: default_true(),
            min_offroad_distance: default_min_offroad_distance(),
            min_wide_width: default_min_wide_width(),
            traffic_multiplier: default_traffic_multiplier(),
            aadt_estimates: default_aadt_map(),
            speed_defaults: default_speed_map(),
        }
    }
}
