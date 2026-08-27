<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import maplibregl from 'maplibre-gl';
  import type { ColorMode } from '../../types';
  import type { FeatureCollection } from 'geojson';

  let {
    geojson = null,
    colorMode = 'los',
    onFetchViewport = () => {},
    onSelectSegment = (segment: any) => {}
  }: {
    geojson: FeatureCollection | null;
    colorMode: ColorMode;
    onFetchViewport?: () => void;
    onSelectSegment?: (segment: any) => void;
  } = $props();

  let mapContainer: HTMLDivElement;
  let map: maplibregl.Map | null = null;
  let hoveredFeatureId: string | number | null = null;
  let popup: maplibregl.Popup | null = null;

  const BASEMAP_STYLES = {
    dark: 'https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json',
    positron: 'https://basemaps.cartocdn.com/gl/positron-gl-style/style.json',
    voyager: 'https://basemaps.cartocdn.com/gl/voyager-gl-style/style.json'
  };

  let currentBasemap = $state<'dark' | 'positron' | 'voyager'>('dark');

  onMount(() => {
    map = new maplibregl.Map({
      container: mapContainer,
      style: BASEMAP_STYLES[currentBasemap],
      center: [-3.195, 55.952], // Edinburgh default
      zoom: 14.5,
      pitch: 0,
      bearing: 0,
      attributionControl: false
    });

    map.addControl(new maplibregl.NavigationControl({ showCompass: true }), 'top-right');
    map.addControl(new maplibregl.ScaleControl({ unit: 'metric' }), 'bottom-right');
    map.addControl(
      new maplibregl.AttributionControl({
        compact: true,
        customAttribution: '© OpenStreetMap contributors | MultiLevelOS'
      }),
      'bottom-right'
    );

    map.on('load', () => {
      initLayers();
      if (geojson) {
        updateSource(geojson);
      }
    });

    map.on('mousemove', 'los-network-lines', (e) => {
      if (!map) return;
      map.getCanvas().style.cursor = 'pointer';
    });

    map.on('mouseleave', 'los-network-lines', () => {
      if (!map) return;
      map.getCanvas().style.cursor = '';
    });

    map.on('click', 'los-network-lines', (e) => {
      if (!map || !e.features || e.features.length === 0) return;
      const feature = e.features[0];
      const props = feature.properties || {};

      onSelectSegment(props);

      const coordinates = e.lngLat;

      const html = `
        <div style="font-family: inherit; font-size: 13px;">
          <div style="font-weight: 700; font-size: 14px; margin-bottom: 6px; color: #f8fafc;">
            ${props.name || 'Unnamed Street'}
          </div>
          <div style="display: inline-block; padding: 2px 8px; border-radius: 4px; font-weight: 600; font-size: 11px; margin-bottom: 8px; background-color: ${props.los_color || '#16a34a'}; color: #ffffff;">
            LoS: ${props.los_label || 'High'} (Score: ${props.los_score})
          </div>
          <div style="display: grid; grid-template-columns: auto auto; gap: 4px 12px; color: #cbd5e1; font-size: 12px;">
            <span style="color: #94a3b8;">Highway:</span>
            <span style="font-weight: 500; font-family: monospace;">${props.highway}</span>

            <span style="color: #94a3b8;">Infrastructure:</span>
            <span style="font-weight: 500;">${props.infrastructure}</span>

            <span style="color: #94a3b8;">Speed Limit:</span>
            <span style="font-weight: 500;">${props.speed_limit_mph != null ? props.speed_limit_mph + ' mph (' + props.speed_category + ')' : 'Traffic Free'}</span>

            <span style="color: #94a3b8;">Est. AADT:</span>
            <span style="font-weight: 500;">${props.estimated_aadt != null ? props.estimated_aadt.toLocaleString() + ' (' + props.aadt_category + ')' : 'None (Active)'}</span>

            ${props.distance_to_road_m != null ? `
              <span style="color: #94a3b8;">Dist to Road:</span>
              <span style="font-weight: 500;">${props.distance_to_road_m} m</span>
            ` : ''}

            <span style="color: #94a3b8;">Length:</span>
            <span style="font-weight: 500;">${props.length_m} m</span>
          </div>
          <div style="margin-top: 10px; padding-top: 6px; border-top: 1px solid #334155; font-size: 11px;">
            <a href="https://www.openstreetmap.org/way/${props.osm_id}" target="_blank" rel="noreferrer" style="color: #38bdf8; text-decoration: none; display: flex; align-items: center; gap: 4px;">
              View on OpenStreetMap (ID: ${props.osm_id}) ↗
            </a>
          </div>
        </div>
      `;

      if (popup) popup.remove();
      popup = new maplibregl.Popup({ offset: 12 })
        .setLngLat(coordinates)
        .setHTML(html)
        .addTo(map);
    });
  });

  function initLayers() {
    if (!map) return;

    if (!map.getSource('los-network')) {
      map.addSource('los-network', {
        type: 'geojson',
        data: geojson || { type: 'FeatureCollection', features: [] }
      });
    }

    if (!map.getLayer('los-network-casing')) {
      map.addLayer({
        id: 'los-network-casing',
        type: 'line',
        source: 'los-network',
        layout: {
          'line-cap': 'round',
          'line-join': 'round'
        },
        paint: {
          'line-color': '#020617',
          'line-width': [
            'interpolate',
            ['linear'],
            ['zoom'],
            12, 3.5,
            15, 6,
            18, 9
          ],
          'line-opacity': 0.8
        }
      });
    }

    if (!map.getLayer('los-network-lines')) {
      map.addLayer({
        id: 'los-network-lines',
        type: 'line',
        source: 'los-network',
        layout: {
          'line-cap': 'round',
          'line-join': 'round'
        },
        paint: getLinePaintExpression(colorMode)
      });
    }
  }

  function getLinePaintExpression(mode: ColorMode): any {
    let colorExpr: any;

    if (mode === 'los') {
      colorExpr = ['coalesce', ['get', 'los_color'], '#16a34a'];
    } else if (mode === 'infrastructure') {
      colorExpr = [
        'match',
        ['get', 'infrastructure'],
        'Segregated Track (wide)', '#054d05',
        'Off Road Path', '#3a9120',
        'Segregated Track (narrow)', '#87d668',
        'Shared Footway', '#ffbf00',
        'Painted Cycle Lane', '#f97316',
        /* default */ '#64748b'
      ];
    } else if (mode === 'speed') {
      colorExpr = [
        'match',
        ['get', 'speed_category'],
        '<20 mph', '#22c55e',
        '20 mph', '#84cc16',
        '30 mph', '#eab308',
        '40 mph', '#f97316',
        '50 mph', '#ef4444',
        '60+ mph', '#7f1d1d',
        /* default */ '#22c55e'
      ];
    } else {
      // Traffic AADT
      colorExpr = [
        'match',
        ['get', 'aadt_category'],
        '0 to 999', '#22c55e',
        '1000 to 1999', '#eab308',
        '2000 to 3999', '#f97316',
        '4000+', '#ef4444',
        /* default */ '#22c55e'
      ];
    }

    return {
      'line-color': colorExpr,
      'line-width': [
        'interpolate',
        ['linear'],
        ['zoom'],
        12, 2.0,
        15, 3.8,
        18, 6.5
      ],
      'line-opacity': 0.95
    };
  }

  function updateSource(data: FeatureCollection | null) {
    if (!map) return;
    const src = map.getSource('los-network') as maplibregl.GeoJSONSource | undefined;
    if (src) {
      src.setData(data || { type: 'FeatureCollection', features: [] });
    }
  }

  $effect(() => {
    if (map && map.isStyleLoaded()) {
      updateSource(geojson);
    }
  });

  $effect(() => {
    if (map && map.isStyleLoaded() && map.getLayer('los-network-lines')) {
      const expr = getLinePaintExpression(colorMode);
      map.setPaintProperty('los-network-lines', 'line-color', expr['line-color']);
    }
  });

  export function flyToBounds(bbox: [number, number, number, number]) {
    if (!map) return;
    map.fitBounds(
      [
        [bbox[0], bbox[1]],
        [bbox[2], bbox[3]]
      ],
      { padding: 40, maxZoom: 16, duration: 1500 }
    );
  }

  export function flyToPoint(lng: number, lat: number, zoom: number = 14.5) {
    if (!map) return;
    map.flyTo({ center: [lng, lat], zoom, duration: 1500 });
  }

  export function getMapBounds(): [number, number, number, number] | null {
    if (!map) return null;
    const b = map.getBounds();
    return [b.getWest(), b.getSouth(), b.getEast(), b.getNorth()];
  }

  function setBasemap(styleKey: 'dark' | 'positron' | 'voyager') {
    if (!map) return;
    currentBasemap = styleKey;
    map.setStyle(BASEMAP_STYLES[styleKey]);
    map.once('style.load', () => {
      initLayers();
      updateSource(geojson);
    });
  }

  onDestroy(() => {
    if (map) map.remove();
  });
</script>

<div class="map-wrapper">
  <div bind:this={mapContainer} class="map-element"></div>

  <!-- Map Top Controls Overlay -->
  <div class="map-top-bar">
    <div class="basemap-selector">
      <button
        class="btn-style {currentBasemap === 'dark' ? 'active' : ''}"
        onclick={() => setBasemap('dark')}
      >
        Dark
      </button>
      <button
        class="btn-style {currentBasemap === 'positron' ? 'active' : ''}"
        onclick={() => setBasemap('positron')}
      >
        Light
      </button>
      <button
        class="btn-style {currentBasemap === 'voyager' ? 'active' : ''}"
        onclick={() => setBasemap('voyager')}
      >
        Streets
      </button>
    </div>

    <button class="btn-fetch-view" onclick={onFetchViewport}>
      <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
        <path d="M3 3v5h5"/>
        <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"/>
        <path d="M16 16h5v5"/>
      </svg>
      Fetch OSM for Visible View
    </button>
  </div>
</div>

<style>
  .map-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .map-element {
    width: 100%;
    height: 100%;
  }

  .map-top-bar {
    position: absolute;
    top: 14px;
    left: 14px;
    display: flex;
    align-items: center;
    gap: 10px;
    z-index: 10;
    pointer-events: auto;
  }

  .basemap-selector {
    display: flex;
    background: rgba(15, 23, 42, 0.85);
    backdrop-filter: blur(10px);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 3px;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
  }

  .btn-style {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    padding: 5px 10px;
    border-radius: 5px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-style:hover {
    color: var(--text-primary);
  }

  .btn-style.active {
    background: var(--bg-card);
    color: #38bdf8;
    font-weight: 600;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  .btn-fetch-view {
    display: flex;
    align-items: center;
    gap: 6px;
    background: rgba(15, 23, 42, 0.9);
    backdrop-filter: blur(10px);
    border: 1px solid #0284c7;
    color: #e0f2fe;
    padding: 6px 12px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 4px 8px rgba(2, 132, 199, 0.25);
    transition: all 0.2s ease;
  }

  .btn-fetch-view:hover {
    background: #0284c7;
    color: #ffffff;
    transform: translateY(-1px);
    box-shadow: 0 6px 12px rgba(2, 132, 199, 0.35);
  }
</style>
