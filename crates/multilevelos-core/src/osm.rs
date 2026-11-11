use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Overpass API Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverpassResponse {
    #[serde(default)]
    pub elements: Vec<OsmElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OsmElement {
    #[serde(rename = "node")]
    Node {
        id: i64,
        lat: f64,
        lon: f64,
        #[serde(default)]
        tags: HashMap<String, String>,
    },
    #[serde(rename = "way")]
    Way {
        id: i64,
        #[serde(default)]
        nodes: Vec<i64>,
        #[serde(default)]
        tags: HashMap<String, String>,
        #[serde(default)]
        geometry: Option<Vec<OsmGeometryCoord>>,
    },
    #[serde(rename = "relation")]
    Relation {
        id: i64,
        #[serde(default)]
        tags: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsmGeometryCoord {
    pub lat: f64,
    pub lon: f64,
}

/// Tagged OSM point feature (crossings, signals, barriers, mini-roundabouts)
#[derive(Debug, Clone)]
pub struct ResolvedOsmNode {
    pub id: i64,
    pub lon: f64,
    pub lat: f64,
    pub tags: HashMap<String, String>,
}

/// Raw parsed Way with resolved coordinate sequence and topological node IDs
#[derive(Debug, Clone)]
pub struct ResolvedOsmWay {
    pub id: i64,
    pub node_ids: Vec<i64>,
    pub tags: HashMap<String, String>,
    pub coords: Vec<(f64, f64)>, // (lng, lat)
}

/// Parses an Overpass API JSON string into an OverpassResponse
pub fn parse_overpass_json(json_str: &str) -> anyhow::Result<OverpassResponse> {
    let resp: OverpassResponse = serde_json::from_str(json_str)?;
    Ok(resp)
}

fn synthetic_coord_id(lon: f64, lat: f64) -> i64 {
    let qx = (lon * 1_000_000.0).round() as i64;
    let qy = (lat * 1_000_000.0).round() as i64;
    // Negative synthetic ID to avoid colliding with positive OSM node IDs
    -((qx.wrapping_mul(1_000_003)) ^ qy).abs() - 1
}

/// Extracts all OSM nodes carrying non-empty tags (e.g. crossings, signals, kerbs, barriers)
pub fn extract_tagged_nodes(elements: &[OsmElement]) -> Vec<ResolvedOsmNode> {
    let mut tagged: HashMap<i64, ResolvedOsmNode> = HashMap::new();
    for el in elements {
        if let OsmElement::Node { id, lat, lon, tags } = el {
            if !tags.is_empty() {
                tagged.insert(
                    *id,
                    ResolvedOsmNode {
                        id: *id,
                        lon: *lon,
                        lat: *lat,
                        tags: tags.clone(),
                    },
                );
            }
        }
    }
    tagged.into_values().collect()
}

/// Resolves raw Overpass elements into a list of ResolvedOsmWay with coordinates and node IDs
pub fn extract_resolved_ways(elements: &[OsmElement]) -> Vec<ResolvedOsmWay> {
    let mut node_coords: HashMap<i64, (f64, f64)> = HashMap::new();

    for el in elements {
        if let OsmElement::Node { id, lat, lon, .. } = el {
            node_coords.insert(*id, (*lon, *lat));
        }
    }

    let mut ways = Vec::new();

    for el in elements {
        if let OsmElement::Way {
            id,
            nodes,
            tags,
            geometry,
        } = el
        {
            let mut resolved_coords = Vec::new();
            let mut resolved_ids = Vec::new();

            if let Some(geom) = geometry {
                for (idx, c) in geom.iter().enumerate() {
                    resolved_coords.push((c.lon, c.lat));
                    if idx < nodes.len() {
                        resolved_ids.push(nodes[idx]);
                    } else {
                        resolved_ids.push(synthetic_coord_id(c.lon, c.lat));
                    }
                }
            } else {
                for nid in nodes {
                    if let Some(&coord) = node_coords.get(nid) {
                        resolved_coords.push(coord);
                        resolved_ids.push(*nid);
                    }
                }
            }

            if resolved_coords.len() >= 2 {
                ways.push(ResolvedOsmWay {
                    id: *id,
                    node_ids: resolved_ids,
                    tags: tags.clone(),
                    coords: resolved_coords,
                });
            }
        }
    }

    ways
}
