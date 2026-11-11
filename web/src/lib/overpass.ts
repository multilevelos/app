export const OVERPASS_ENDPOINTS = [
  'https://overpass.openstreetmap.fr/api/interpreter',
  'https://lz4.overpass-api.de/api/interpreter',
  'https://z.overpass-api.de/api/interpreter',
  'https://overpass-api.de/api/interpreter'
];

/**
 * Execute Overpass query with automatic retry across reliable mirror endpoints
 */
async function executeOverpassQuery(query: string): Promise<string> {
  let lastError: Error | null = null;

  for (const endpoint of OVERPASS_ENDPOINTS) {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 15000);

      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8',
          'Accept': 'application/json'
        },
        body: `data=${encodeURIComponent(query)}`,
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      if (response.ok) {
        const text = await response.text();
        // Verify it is not an error payload
        if (text.startsWith('{') && text.includes('"elements"')) {
          return text;
        } else if (text.includes('runtime error') || text.includes('rate_limited')) {
          lastError = new Error(`Overpass runtime limit on ${endpoint}`);
          continue;
        }
        return text;
      } else {
        lastError = new Error(`HTTP ${response.status} from ${endpoint}`);
      }
    } catch (err: any) {
      lastError = err;
    }
  }

  throw lastError || new Error('Failed to fetch OSM data from all Overpass endpoints');
}

/**
 * Sanitizes and bounds a bbox to a manageable size (max ~6km span) to prevent Overpass timeouts
 */
export function sanitizeBbox(
  minLng: number,
  minLat: number,
  maxLng: number,
  maxLat: number,
  maxSpanDeg: number = 0.06
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
 * Fetch routable OSM street, walking, wheeling & cycling network + tagged crossings/signals for a bounding box
 */
export async function fetchOsmNetworkForBbox(
  minLng: number,
  minLat: number,
  maxLng: number,
  maxLat: number
): Promise<string> {
  const [w, s, e, n] = sanitizeBbox(minLng, minLat, maxLng, maxLat);

  const query = `[out:json][timeout:25];
(
  way["highway"]["highway"!~"proposed|construction|corridor|elevator|raceway"](${s},${w},${n},${e});
  way["cycleway"](${s},${w},${n},${e});
  node["highway"~"crossing|traffic_signals|mini_roundabout"](${s},${w},${n},${e});
);
out body;
>;
out skel qt;`;

  return await executeOverpassQuery(query);
}

/**
 * Fetch OSM network around a point within a radius (in meters)
 */
export async function fetchOsmNetworkAroundPoint(
  lng: number,
  lat: number,
  radiusMeters: number = 2500
): Promise<string> {
  const cleanRadius = Math.min(radiusMeters, 4000);
  const cleanLat = lat.toFixed(5);
  const cleanLng = lng.toFixed(5);

  const query = `[out:json][timeout:25];
(
  way["highway"]["highway"!~"proposed|construction|corridor|elevator|raceway"](around:${cleanRadius},${cleanLat},${cleanLng});
  way["cycleway"](around:${cleanRadius},${cleanLat},${cleanLng});
  node["highway"~"crossing|traffic_signals|mini_roundabout"](around:${cleanRadius},${cleanLat},${cleanLng});
);
out body;
>;
out skel qt;`;

  return await executeOverpassQuery(query);
}
