<script lang="ts" module>
  export function streetViewUrl(lon: number, lat: number): string {
    return `https://maps.google.com/maps?q=&layer=c&cbll=${lat.toFixed(6)},${lon.toFixed(6)}&cbp=11,0,0,0,0`;
  }

  export function openStreetMapUrl(lon: number, lat: number): string {
    return `https://www.openstreetmap.org/#map=19/${lat.toFixed(6)}/${lon.toFixed(6)}`;
  }

  export function osmWayUrl(osmId: number | string | undefined | null): string | null {
    if (osmId === undefined || osmId === null || osmId === '') return null;
    return `https://www.openstreetmap.org/way/${encodeURIComponent(String(osmId))}`;
  }

  export function mapButtonsHtml(lon: number, lat: number, osmId?: number | string | null): string {
    const sv = streetViewUrl(lon, lat);
    const osm = osmWayUrl(osmId) ?? openStreetMapUrl(lon, lat);
    return `
      <div class="map-popup-links">
        <a href="${sv}" target="_blank" rel="noopener noreferrer" class="map-ext-link">Street View</a>
        <a href="${osm}" target="_blank" rel="noopener noreferrer" class="map-ext-link">OSM Way ↗</a>
      </div>
    `;
  }

  export function featureCenter(geometry: any): [number, number] | null {
    const c = geometry?.coordinates;
    if (!Array.isArray(c) || c.length === 0) return null;
    if (typeof c[0] === 'number' && typeof c[1] === 'number') {
      return [c[0], c[1]];
    }
    if (Array.isArray(c[0]) && typeof c[0][0] === 'number' && typeof c[0][1] === 'number') {
      const mid = c[Math.floor(c.length / 2)];
      return [mid[0], mid[1]];
    }
    if (
      Array.isArray(c[0]) &&
      Array.isArray(c[0][0]) &&
      typeof c[0][0][0] === 'number' &&
      typeof c[0][0][1] === 'number'
    ) {
      const ring = c[0];
      let sumX = 0;
      let sumY = 0;
      for (const pt of ring) {
        sumX += pt[0];
        sumY += pt[1];
      }
      return [sumX / ring.length, sumY / ring.length];
    }
    return null;
  }

  export function featureBbox(geometry: any): [number, number, number, number] | null {
    const c = geometry?.coordinates;
    if (!Array.isArray(c) || c.length === 0) return null;
    let minX = 180;
    let minY = 90;
    let maxX = -180;
    let maxY = -90;

    function visit(node: any) {
      if (!Array.isArray(node) || node.length === 0) return;
      if (typeof node[0] === 'number' && typeof node[1] === 'number') {
        if (node[0] < minX) minX = node[0];
        if (node[1] < minY) minY = node[1];
        if (node[0] > maxX) maxX = node[0];
        if (node[1] > maxY) maxY = node[1];
      } else {
        for (const child of node) visit(child);
      }
    }

    visit(c);
    if (minX > maxX || minY > maxY) return null;
    // Pad degenerate point bboxes slightly so fitBounds zooms cleanly
    if (Math.abs(maxX - minX) < 0.0004) {
      minX -= 0.0005;
      maxX += 0.0005;
    }
    if (Math.abs(maxY - minY) < 0.0004) {
      minY -= 0.0005;
      maxY += 0.0005;
    }
    return [minX, minY, maxX, maxY];
  }
</script>

<script lang="ts">
  let {
    center = null,
    osmId = null
  }: {
    center?: [number, number] | null;
    osmId?: number | string | null;
  } = $props();

  const hasCenter = $derived(
    Array.isArray(center) &&
      center.length === 2 &&
      Number.isFinite(center[0]) &&
      Number.isFinite(center[1])
  );
  const svHref = $derived(hasCenter && center ? streetViewUrl(center[0], center[1]) : '#');
  const osmHref = $derived(
    osmWayUrl(osmId) ?? (hasCenter && center ? openStreetMapUrl(center[0], center[1]) : '#')
  );
</script>

{#if hasCenter || osmId}
  <div class="map-popup-links">
    {#if hasCenter}
      <a href={svHref} target="_blank" rel="noopener noreferrer" class="map-ext-link">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" style="display:none"/>
          <circle cx="12" cy="10" r="3"></circle>
          <path d="M12 21.7C17.3 17 20 13 20 10a8 8 0 1 0-16 0c0 3 2.7 6.9 8 11.7z"></path>
        </svg>
        Street View
      </a>
    {/if}
    <a href={osmHref} target="_blank" rel="noopener noreferrer" class="map-ext-link">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
        <polyline points="15 3 21 3 21 9"></polyline>
        <line x1="10" y1="14" x2="21" y2="3"></line>
      </svg>
      {osmId ? 'OSM Way' : 'OpenStreetMap'}
    </a>
  </div>
{/if}
