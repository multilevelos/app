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

/// Raw parsed Way with resolved coordinate sequence
#[derive(Debug, Clone)]
pub struct ResolvedOsmWay {
    pub id: i64,
    pub tags: HashMap<String, String>,
    pub coords: Vec<(f64, f64)>, // (lng, lat)
}

/// Parses an Overpass API JSON string into an OverpassResponse
pub fn parse_overpass_json(json_str: &str) -> anyhow::Result<OverpassResponse> {
    let resp: OverpassResponse = serde_json::from_str(json_str)?;
    Ok(resp)
}

/// Resolves raw Overpass elements into a list of ResolvedOsmWay with coordinates
pub fn extract_resolved_ways(elements: &[OsmElement]) -> Vec<ResolvedOsmWay> {
    let mut node_coords: HashMap<i64, (f64, f64)> = HashMap::new();

    // First pass: collect all node coordinates
    for el in elements {
        if let OsmElement::Node { id, lat, lon, .. } = el {
            node_coords.insert(*id, (*lon, *lat));
        }
    }

    let mut ways = Vec::new();

    // Second pass: extract ways
    for el in elements {
        if let OsmElement::Way {
            id,
            nodes,
            tags,
            geometry,
        } = el
        {
            let coords = if let Some(geom) = geometry {
                // If inline geometry is provided (e.g. out geom;)
                geom.iter().map(|c| (c.lon, c.lat)).collect::<Vec<_>>()
            } else {
                // Otherwise look up node coords
                nodes
                    .iter()
                    .filter_map(|nid| node_coords.get(nid).copied())
                    .collect::<Vec<_>>()
            };

            // Only include ways with at least 2 points
            if coords.len() >= 2 {
                ways.push(ResolvedOsmWay {
                    id: *id,
                    tags: tags.clone(),
                    coords,
                });
            }
        }
    }

    ways
}
