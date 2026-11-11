const OSM_MAIN_API = 'https://api.openstreetmap.org/api/0.6/map.json';

export const OVERPASS_ENDPOINTS = [
  'https://overpass-api.de/api/interpreter',
  'https://lz4.overpass-api.de/api/interpreter',
  'https://overpass.private.coffee/api/interpreter'
];

interface RawOsmElement {
  type: 'node' | 'way' | 'relation';
  id: number;
  lat?: number;
  lon?: number;
  nodes?: number[];
  tags?: Record<string, string>;
  geometry?: Array<{ lat: number; lon: number }>;
}

/**
 * Prunes buildings/POIs/landuse from raw OSM elements, retaining only highway/cycleway ways,
 * their vertex nodes, and tagged crossing/signal/kerb nodes. Cuts payload size by ~85%.
 */
function pruneToHighwayElements(elements: RawOsmElement[]): {
  nodes: Map<number, RawOsmElement>;
  ways: Map<number, RawOsmElement>;
} {
  const ways = new Map<number, RawOsmElement>();
  const neededNodeIds = new Set<number>();

  for (const el of elements) {
    if (el.type === 'way' && (el.tags?.highway || el.tags?.cycleway)) {
      ways.set(el.id, el);
      if (Array.isArray(el.nodes)) {
        for (const nid of el.nodes) {
          neededNodeIds.add(nid);
        }
      }
    }
  }

  const nodes = new Map<number, RawOsmElement>();
  for (const el of elements) {
    if (el.type === 'node') {
      const hasRelevantTag =
        el.tags &&
        (el.tags.highway || el.tags.crossing || el.tags.tactile_paving || el.tags.kerb);
      if (neededNodeIds.has(el.id) || hasRelevantTag) {
        nodes.set(el.id, el);
      }
    }
  }

  return { nodes, ways };
}

/**
 * Fetches live OSM data directly from OpenStreetMap's high-availability cgimap v0.6 JSON endpoint.
 * Automatically subdivides into a 2x2 tile grid if a dense urban centre exceeds the 50,000-node per-tile cap.
 */
async function fetchOsmV06Recursive(
  w: number,
  s: number,
  e: number,
  n: number,
  depth: number = 0
): Promise<{ nodes: Map<number, RawOsmElement>; ways: Map<number, RawOsmElement> }> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), 16000);

  try {
    const url = `${OSM_MAIN_API}?bbox=${w.toFixed(5)},${s.toFixed(5)},${e.toFixed(5)},${n.toFixed(5)}`;
    const response = await fetch(url, {
      method: 'GET',
      headers: { Accept: 'application/json' },
      signal: controller.signal
    });
    clearTimeout(timeoutId);

    if (response.ok) {
      const data = await response.json();
      return pruneToHighwayElements(Array.isArray(data?.elements) ? data.elements : []);
    }

    // HTTP 400 indicates bbox exceeded OSM v0.6's 50,000-node limit -> subdivide into 4 quadrants
    if (response.status === 400 && depth < 2) {
      const midLng = (w + e) / 2;
      const midLat = (s + n) / 2;
      const quadrants = await Promise.all([
        fetchOsmV06Recursive(w, s, midLng, midLat, depth + 1),
        fetchOsmV06Recursive(midLng, s, e, midLat, depth + 1),
        fetchOsmV06Recursive(w, midLat, midLng, n, depth + 1),
        fetchOsmV06Recursive(midLng, midLat, e, n, depth + 1)
      ]);

      const mergedNodes = new Map<number, RawOsmElement>();
      const mergedWays = new Map<number, RawOsmElement>();
      for (const q of quadrants) {
        for (const [id, node] of q.nodes) mergedNodes.set(id, node);
        for (const [id, way] of q.ways) mergedWays.set(id, way);
      }
      return { nodes: mergedNodes, ways: mergedWays };
    }

    throw new Error(`OSM API HTTP ${response.status}`);
  } finally {
    clearTimeout(timeoutId);
  }
}

/**
 * Fallback single-pass `out geom qt;` Overpass query if `api.openstreetmap.org` is unreachable
 */
async function executeOverpassFallback(
  w: number,
  s: number,
  e: number,
  n: number
): Promise<string> {
  const query = `[out:json][timeout:18];(way["highway"](${s},${w},${n},${e});node["highway"~"crossing|traffic_signals|mini_roundabout"](${s},${w},${n},${e}););out geom qt;`;
  let lastError: Error | null = null;

  for (const endpoint of OVERPASS_ENDPOINTS) {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 15000);

      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8'
        },
        body: `data=${encodeURIComponent(query)}`,
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      if (response.ok) {
        const text = await response.text();
        if (text.startsWith('{') && text.includes('"elements"')) {
          return text;
        }
      } else {
        lastError = new Error(`HTTP ${response.status} from ${endpoint}`);
      }
    } catch (err: any) {
      lastError = err;
    }
  }

  throw lastError || new Error('Failed to fetch live OSM network');
}

/**
 * Sanitizes and bounds a bbox to a fast urban study window (~3.5km span)
 */
export function sanitizeBbox(
  minLng: number,
  minLat: number,
  maxLng: number,
  maxLat: number,
  maxSpanDeg: number = 0.032
): [number, number, number, number] {
  let s = Math.min(minLat, maxLat);
  let n = Math.max(minLat, maxLat);
  let w = Math.min(minLng, maxLng);
  let e = Math.max(minLng, maxLng);

  const spanLat = n - s;
  const spanLng = e - w;

  if (spanLat > maxSpanDeg || spanLng > maxSpanDeg) {
    const centerLat = (s + n) / 2;
    const centerLng = (w + e) / 2;
    const halfSpan = maxSpanDeg / 2;
    s = centerLat - halfSpan;
    n = centerLat + halfSpan;
    w = centerLng - halfSpan;
    e = centerLng + halfSpan;
  }

  return [
    parseFloat(w.toFixed(5)),
    parseFloat(s.toFixed(5)),
    parseFloat(e.toFixed(5)),
    parseFloat(n.toFixed(5))
  ];
}

/**
 * Fetch routable OSM street, walking, wheeling & cycling network + crossings/signals for a bounding box.
 * Uses OpenStreetMap's fast CDN-backed v0.6 JSON API first (with automatic quadrant tiling), falling back to Overpass.
 */
export async function fetchOsmNetworkForBbox(
  minLng: number,
  minLat: number,
  maxLng: number,
  maxLat: number
): Promise<string> {
  const [w, s, e, n] = sanitizeBbox(minLng, minLat, maxLng, maxLat);

  try {
    const { nodes, ways } = await fetchOsmV06Recursive(w, s, e, n, 0);
    const elements: RawOsmElement[] = [...nodes.values(), ...ways.values()];
    return JSON.stringify({ version: 0.6, elements });
  } catch (primaryErr) {
    console.warn('OSM v0.6 API fallback to Overpass:', primaryErr);
    return await executeOverpassFallback(w, s, e, n);
  }
}

/**
 * Fetch OSM network around a point within a radius (in meters)
 */
export async function fetchOsmNetworkAroundPoint(
  lng: number,
  lat: number,
  radiusMeters: number = 1800
): Promise<string> {
  const cleanRadius = Math.min(radiusMeters, 3000);
  const dLat = cleanRadius / 111_132;
  const dLon = cleanRadius / (111_320 * Math.max(0.2, Math.cos((lat * Math.PI) / 180)));
  return await fetchOsmNetworkForBbox(lng - dLon, lat - dLat, lng + dLon, lat + dLat);
}
