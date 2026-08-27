export const OVERPASS_ENDPOINTS = [
  'https://overpass-api.de/api/interpreter',
  'https://overpass.kumi.systems/api/interpreter',
  'https://maps.mail.ru/osm/tools/overpass/api/interpreter'
];

/**
 * Execute Overpass query with automatic retry across fallback endpoints
 */
async function executeOverpassQuery(query: string): Promise<string> {
  let lastError: Error | null = null;

  for (const endpoint of OVERPASS_ENDPOINTS) {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 40000);

      const response = await fetch(endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: `data=${encodeURIComponent(query)}`,
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      if (response.ok) {
        return await response.text();
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
 * Fetch routable OSM road and cycle network for a given bounding box
 */
export async function fetchOsmNetworkForBbox(
  minLng: number,
  minLat: number,
  maxLng: number,
  maxLat: number
): Promise<string> {
  const query = `[out:json][timeout:35];
(
  way["highway"]["highway"!~"proposed|construction|corridor|elevator|steps"](${minLat},${minLng},${maxLat},${maxLng});
  way["cycleway"](${minLat},${minLng},${maxLat},${maxLng});
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
  const query = `[out:json][timeout:35];
(
  way["highway"]["highway"!~"proposed|construction|corridor|elevator|steps"](around:${radiusMeters},${lat},${lng});
  way["cycleway"](around:${radiusMeters},${lat},${lng});
);
out body;
>;
out skel qt;`;

  return await executeOverpassQuery(query);
}
