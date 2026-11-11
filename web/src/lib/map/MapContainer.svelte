<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import maplibregl from 'maplibre-gl';
  import {
    escapeHtml,
    formatShortLosLabel,
    getColorExpression,
    getFilterExpression,
    getLosTone
  } from '../palette';
  import { featureBbox, featureCenter, mapButtonsHtml } from '../components/MapButtons.svelte';
  import type { BasemapId, ColorMode, SelectedSegment } from '../../types';
  import type { FeatureCollection } from 'geojson';

  let {
    geojson = null,
    colorMode = 'los',
    hiddenValues = [],
    basemap = 'light',
    showPlaceNames = true,
    sidebarOpen = true,
    onSelectSegment = (_seg: SelectedSegment | null) => {}
  }: {
    geojson: FeatureCollection | null;
    colorMode: ColorMode;
    hiddenValues?: string[];
    basemap?: BasemapId;
    showPlaceNames?: boolean;
    sidebarOpen?: boolean;
    onSelectSegment?: (seg: SelectedSegment | null) => void;
  } = $props();

  let mapContainer: HTMLDivElement;
  let map: maplibregl.Map | null = null;
  let clickPopup: maplibregl.Popup | null = null;
  let hoverPopup: maplibregl.Popup | null = null;
  let hoveredFeatureId: string | number | null = null;
  let isTouchDevice = false;

  // Split label-free base cartography + toggleable top-level place-name reference layer
  function buildStyleSpec(id: BasemapId): maplibregl.StyleSpecification {
    if (id === 'dark') {
      return {
        version: 8,
        sources: {
          'basemap-base': {
            type: 'raster',
            tiles: ['https://basemaps.cartocdn.com/dark_nolabels/{z}/{x}/{y}@2x.png'],
            tileSize: 256,
            attribution: '&copy; OpenStreetMap contributors &copy; CARTO'
          },
          'basemap-labels': {
            type: 'raster',
            tiles: ['https://basemaps.cartocdn.com/dark_only_labels/{z}/{x}/{y}@2x.png'],
            tileSize: 256
          }
        },
        layers: [{ id: 'basemap-base-layer', type: 'raster', source: 'basemap-base' }]
      };
    }

    if (id === 'voyager') {
      return {
        version: 8,
        sources: {
          'basemap-base': {
            type: 'raster',
            tiles: ['https://basemaps.cartocdn.com/rastertiles/voyager_nolabels/{z}/{x}/{y}@2x.png'],
            tileSize: 256,
            attribution: '&copy; OpenStreetMap contributors &copy; CARTO'
          },
          'basemap-labels': {
            type: 'raster',
            tiles: ['https://basemaps.cartocdn.com/rastertiles/voyager_only_labels/{z}/{x}/{y}@2x.png'],
            tileSize: 256
          }
        },
        layers: [{ id: 'basemap-base-layer', type: 'raster', source: 'basemap-base' }]
      };
    }

    if (id === 'satellite') {
      return {
        version: 8,
        sources: {
          'basemap-base': {
            type: 'raster',
            tiles: [
              'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}'
            ],
            tileSize: 256,
            maxzoom: 19,
            attribution: 'Tiles &copy; Esri &mdash; Esri, Maxar, Earthstar Geographics'
          },
          'basemap-labels': {
            type: 'raster',
            tiles: ['https://basemaps.cartocdn.com/dark_only_labels/{z}/{x}/{y}@2x.png'],
            tileSize: 256
          }
        },
        layers: [{ id: 'basemap-base-layer', type: 'raster', source: 'basemap-base' }]
      };
    }

    if (id === 'opencyclemap') {
      return {
        version: 8,
        sources: {
          'basemap-base': {
            type: 'raster',
            tiles: [
              'https://tile.thunderforest.com/cycle/{z}/{x}/{y}@2x.png?apikey=e26a96d14f5e41d78dd4842ac12f458c'
            ],
            tileSize: 256,
            attribution: 'Maps &copy; Thunderforest, Data &copy; OpenStreetMap contributors'
          }
        },
        layers: [
          { id: 'basemap-bg', type: 'background', paint: { 'background-color': '#f8fafc' } },
          {
            id: 'basemap-base-layer',
            type: 'raster',
            source: 'basemap-base',
            paint: { 'raster-opacity': 0.72 }
          }
        ]
      };
    }

    // Default: 'light' Dignified Civic Light Canvas
    return {
      version: 8,
      sources: {
        'basemap-base': {
          type: 'raster',
          tiles: ['https://basemaps.cartocdn.com/light_nolabels/{z}/{x}/{y}@2x.png'],
          tileSize: 256,
          attribution: '&copy; OpenStreetMap contributors &copy; CARTO'
        },
        'basemap-labels': {
          type: 'raster',
          tiles: ['https://basemaps.cartocdn.com/light_only_labels/{z}/{x}/{y}@2x.png'],
          tileSize: 256
        }
      },
      layers: [{ id: 'basemap-base-layer', type: 'raster', source: 'basemap-base' }]
    };
  }

  function getCasingColor(id: BasemapId): string {
    return id === 'dark' || id === 'satellite' ? '#0f172a' : '#ffffff';
  }

  function viewportPadding() {
    const isDesktop = typeof window !== 'undefined' && window.innerWidth > 768;
    return {
      top: 48,
      bottom: 48,
      left: isDesktop && sidebarOpen ? 390 : 48,
      right: 48
    };
  }

  onMount(() => {
    isTouchDevice =
      typeof window !== 'undefined' && window.matchMedia('(hover: none)').matches;

    map = new maplibregl.Map({
      container: mapContainer,
      style: buildStyleSpec(basemap),
      center: [-3.195, 55.952],
      zoom: 14.2,
      pitch: 0,
      bearing: 0,
      hash: 'map',
      canvasContextAttributes: { preserveDrawingBuffer: true },
      attributionControl: false
    });
    appliedBasemap = basemap;

    map.addControl(new maplibregl.NavigationControl({ showCompass: true }), 'top-right');
    map.addControl(
      new maplibregl.GeolocateControl({
        positionOptions: { enableHighAccuracy: true },
        trackUserLocation: false
      }),
      'top-right'
    );
    map.addControl(new maplibregl.ScaleControl({ unit: 'metric' }), 'bottom-left');
    map.addControl(
      new maplibregl.AttributionControl({
        compact: true,
        customAttribution: 'MultiLevelOS'
      }),
      'bottom-right'
    );

    hoverPopup = new maplibregl.Popup({
      closeButton: false,
      closeOnClick: false,
      offset: 12,
      className: 'hover-tooltip'
    });

    map.on('load', () => {
      initLayers();
      if (geojson) updateSource(geojson);
    });

    map.on('mousemove', 'los-network-lines', (e) => {
      if (!map || !e.features || e.features.length === 0) return;
      map.getCanvas().style.cursor = 'pointer';

      const feat = e.features[0];
      if (hoveredFeatureId !== null && hoveredFeatureId !== feat.id) {
        map.setFeatureState({ source: 'los-network', id: hoveredFeatureId }, { hover: false });
      }
      if (feat.id !== undefined) {
        hoveredFeatureId = feat.id;
        map.setFeatureState({ source: 'los-network', id: feat.id }, { hover: true });
      }

      if (!isTouchDevice && hoverPopup) {
        const p = feat.properties || {};
        const tone = getLosTone(p.los_label, Number(p.los_score));
        const shortLos = formatShortLosLabel(p.los_label, Number(p.los_score));
        const spdText =
          p.speed_limit_mph != null ? `${escapeHtml(p.speed_limit_mph)} mph` : 'Traffic-free';
        const html = `
          <div class="map-popup-card">
            <div class="map-badge-group">
              <span class="map-badge ${tone}">${escapeHtml(shortLos)}</span>
              <span class="map-badge">${escapeHtml(p.infrastructure)}</span>
            </div>
            <div class="map-popup-title">${escapeHtml(p.name || 'Unnamed Street')}</div>
            <div class="map-popup-table">
              <div class="map-cell"><span class="k">Speed / Flow</span><span class="v font-mono">${spdText}${p.estimated_aadt != null ? ' · ' + Number(p.estimated_aadt).toLocaleString() + ' AADT' : ''}</span></div>
              <div class="map-cell"><span class="k">Length</span><span class="v font-mono">${escapeHtml(p.length_m)} m</span></div>
            </div>
          </div>
        `;
        hoverPopup.setLngLat(e.lngLat).setHTML(html).addTo(map);
      }
    });

    map.on('mouseleave', 'los-network-lines', () => {
      if (!map) return;
      map.getCanvas().style.cursor = '';
      if (hoveredFeatureId !== null) {
        map.setFeatureState({ source: 'los-network', id: hoveredFeatureId }, { hover: false });
        hoveredFeatureId = null;
      }
      hoverPopup?.remove();
    });

    map.on('click', 'los-network-lines', (e) => {
      if (!map || !e.features || e.features.length === 0) return;
      hoverPopup?.remove();

      const feat = e.features[0];
      const p = feat.properties || {};
      const center = featureCenter(feat.geometry) ?? [e.lngLat.lng, e.lngLat.lat];
      const bbox = featureBbox(feat.geometry);

      const seg: SelectedSegment = {
        osm_id: Number(p.osm_id),
        name: String(p.name || 'Unnamed Street'),
        highway: String(p.highway || 'road'),
        infrastructure: String(p.infrastructure || ''),
        speed_limit_mph: p.speed_limit_mph != null ? Number(p.speed_limit_mph) : null,
        speed_category: String(p.speed_category || ''),
        estimated_aadt: p.estimated_aadt != null ? Number(p.estimated_aadt) : null,
        aadt_category: String(p.aadt_category || ''),
        distance_to_road_m: p.distance_to_road_m != null ? Number(p.distance_to_road_m) : null,
        length_m: Number(p.length_m || 0),
        los_score: Number(p.los_score ?? 0),
        los_label: String(p.los_label || ''),
        los_color: String(p.los_color || '#16a34a'),
        center,
        bbox
      };

      onSelectSegment(seg);

      const tone = getLosTone(seg.los_label, seg.los_score);
      const shortLos = formatShortLosLabel(seg.los_label, seg.los_score);
      const html = `
        <div class="map-popup-card">
          <div class="map-badge-group">
            <span class="map-badge ${tone}">${escapeHtml(shortLos)}</span>
            <span class="map-badge civic">${escapeHtml(seg.highway)}</span>
          </div>
          <div class="map-popup-title">${escapeHtml(seg.name)}</div>
          <div class="map-popup-table">
            <div class="map-cell"><span class="k">Infrastructure</span><span class="v">${escapeHtml(seg.infrastructure)}</span></div>
            <div class="map-cell"><span class="k">Speed limit</span><span class="v font-mono">${seg.speed_limit_mph != null ? seg.speed_limit_mph + ' mph' : 'Traffic-free'}</span></div>
            <div class="map-cell"><span class="k">Est. AADT</span><span class="v font-mono">${seg.estimated_aadt != null ? seg.estimated_aadt.toLocaleString() : 'None'}</span></div>
            <div class="map-cell"><span class="k">Length</span><span class="v font-mono">${seg.length_m} m</span></div>
          </div>
          ${mapButtonsHtml(center[0], center[1], seg.osm_id)}
        </div>
      `;

      clickPopup?.remove();
      clickPopup = new maplibregl.Popup({ offset: 12, maxWidth: '290px' })
        .setLngLat(e.lngLat)
        .setHTML(html)
        .addTo(map);
    });
  });

  function initLayers() {
    if (!map || !map.isStyleLoaded()) return;

    if (!map.getSource('los-network')) {
      map.addSource('los-network', {
        type: 'geojson',
        generateId: true,
        data: geojson || { type: 'FeatureCollection', features: [] }
      });
    }

    const filterExpr = getFilterExpression(colorMode, hiddenValues);

    if (!map.getLayer('los-network-casing')) {
      map.addLayer({
        id: 'los-network-casing',
        type: 'line',
        source: 'los-network',
        ...(filterExpr ? { filter: filterExpr } : {}),
        layout: {
          'line-cap': 'round',
          'line-join': 'round'
        },
        paint: {
          'line-color': getCasingColor(basemap),
          'line-width': [
            'interpolate',
            ['linear'],
            ['zoom'],
            8,
            ['case', ['boolean', ['feature-state', 'hover'], false], 4.5, 2.2],
            13,
            ['case', ['boolean', ['feature-state', 'hover'], false], 7.5, 4.8],
            17,
            ['case', ['boolean', ['feature-state', 'hover'], false], 12.0, 8.6]
          ],
          'line-opacity': 0.88
        }
      });
    }

    if (!map.getLayer('los-network-lines')) {
      map.addLayer({
        id: 'los-network-lines',
        type: 'line',
        source: 'los-network',
        ...(filterExpr ? { filter: filterExpr } : {}),
        layout: {
          'line-cap': 'round',
          'line-join': 'round'
        },
        paint: {
          'line-color': getColorExpression(colorMode),
          'line-width': [
            'interpolate',
            ['linear'],
            ['zoom'],
            8,
            ['case', ['boolean', ['feature-state', 'hover'], false], 3.0, 1.4],
            13,
            ['case', ['boolean', ['feature-state', 'hover'], false], 5.4, 3.3],
            17,
            ['case', ['boolean', ['feature-state', 'hover'], false], 9.0, 6.2]
          ],
          'line-opacity': 0.95
        }
      });
    }

    // Place-names label layer sits ABOVE analyzed vector corridors
    if (map.getSource('basemap-labels') && !map.getLayer('basemap-labels-layer')) {
      map.addLayer({
        id: 'basemap-labels-layer',
        type: 'raster',
        source: 'basemap-labels',
        layout: {
          visibility: showPlaceNames ? 'visible' : 'none'
        },
        paint: {
          'raster-opacity': 0.92
        }
      });
    }
  }

  function updateSource(data: FeatureCollection | null) {
    if (!map) return;
    if (!map.isStyleLoaded()) {
      map.once('style.load', () => updateSource(data));
      return;
    }
    initLayers();
    const src = map.getSource('los-network') as maplibregl.GeoJSONSource | undefined;
    if (src) {
      src.setData(data || { type: 'FeatureCollection', features: [] });
    }
  }

  let appliedBasemap: BasemapId | null = null;

  $effect(() => {
    const nextBasemap = basemap;
    if (map && appliedBasemap !== null && nextBasemap !== appliedBasemap) {
      appliedBasemap = nextBasemap;
      hoveredFeatureId = null;
      map.setStyle(buildStyleSpec(nextBasemap));
      map.once('style.load', () => {
        initLayers();
        updateSource(geojson);
      });
    }
  });

  $effect(() => {
    const data = geojson;
    if (data && map) {
      updateSource(data);
    }
  });

  $effect(() => {
    const mode = colorMode;
    const hidden = hiddenValues;
    if (!map || !map.isStyleLoaded()) return;
    if (map.getLayer('los-network-lines')) {
      map.setPaintProperty('los-network-lines', 'line-color', getColorExpression(mode));
    }
    const filterExpr = getFilterExpression(mode, hidden);
    for (const layerId of ['los-network-casing', 'los-network-lines']) {
      if (map.getLayer(layerId)) {
        map.setFilter(layerId, filterExpr);
      }
    }
  });

  $effect(() => {
    const show = showPlaceNames;
    if (map && map.isStyleLoaded() && map.getLayer('basemap-labels-layer')) {
      map.setLayoutProperty('basemap-labels-layer', 'visibility', show ? 'visible' : 'none');
    }
  });

  export function flyToBounds(bbox: [number, number, number, number], skipIfHashPresent = false) {
    if (!map) return;
    if (skipIfHashPresent && window.location.hash.includes('map=')) return;
    map.fitBounds(
      [
        [bbox[0], bbox[1]],
        [bbox[2], bbox[3]]
      ],
      { padding: viewportPadding(), maxZoom: 16.5, duration: 1200 }
    );
  }

  export function getMapBounds(): [number, number, number, number] | null {
    if (!map) return null;
    const b = map.getBounds();
    return [b.getWest(), b.getSouth(), b.getEast(), b.getNorth()];
  }

  export function exportPng(filename: string = 'multilevelos-map.png') {
    if (!map) return;
    map.triggerRepaint();
    const dataUrl = map.getCanvas().toDataURL('image/png');
    const a = document.createElement('a');
    a.href = dataUrl;
    a.download = filename;
    a.click();
  }

  onDestroy(() => {
    clickPopup?.remove();
    hoverPopup?.remove();
    if (map) map.remove();
  });
</script>

<div class="map-wrapper" class:drawer-open={sidebarOpen}>
  <div bind:this={mapContainer} class="map-canvas"></div>
</div>

<style>
  .map-wrapper {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .map-canvas {
    width: 100%;
    height: 100%;
  }

  /* Offset bottom-left scale bar when desktop drawer is open */
  @media (min-width: 769px) {
    .map-wrapper.drawer-open :global(.maplibregl-ctrl-bottom-left) {
      transform: translateX(356px);
      transition: transform 0.24s cubic-bezier(0.16, 1, 0.3, 1);
    }
    .map-wrapper:not(.drawer-open) :global(.maplibregl-ctrl-bottom-left) {
      transform: translateX(0);
      transition: transform 0.24s cubic-bezier(0.16, 1, 0.3, 1);
    }
  }
</style>
