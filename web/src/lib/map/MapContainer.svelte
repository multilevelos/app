<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import maplibregl from 'maplibre-gl';
  import {
    escapeHtml,
    formatShortLosLabel,
    getColorExpression,
    getFilterExpression,
    gradeTone
  } from '../palette';
  import { featureBbox, featureCenter, mapButtonsHtml } from '../components/MapButtons.svelte';
  import type {
    BasemapId,
    ColorMode,
    SelectedFeature,
    SelectedJunction,
    SelectedSegment
  } from '../../types';
  import type { FeatureCollection } from 'geojson';

  let {
    geojson = null,
    junctionsGeojson = null,
    showJunctions = true,
    colorMode = 'los',
    hiddenValues = [],
    basemap = 'light',
    showPlaceNames = true,
    sidebarOpen = true,
    onSelectFeature = (_feat: SelectedFeature | null) => {}
  }: {
    geojson: FeatureCollection | null;
    junctionsGeojson?: FeatureCollection | null;
    showJunctions?: boolean;
    colorMode: ColorMode;
    hiddenValues?: string[];
    basemap?: BasemapId;
    showPlaceNames?: boolean;
    sidebarOpen?: boolean;
    onSelectFeature?: (feat: SelectedFeature | null) => void;
  } = $props();

  let mapContainer: HTMLDivElement;
  let map: maplibregl.Map | null = null;
  let clickPopup: maplibregl.Popup | null = null;
  let hoverPopup: maplibregl.Popup | null = null;
  let hoveredSegId: string | number | null = null;
  let hoveredJuncId: string | number | null = null;
  let isTouchDevice = false;

  // Vector GL styles (CARTO MVT) + open keyless CyclOSM & Esri Satellite styles (zero watermarks / API keys)
  function buildStyleSpec(id: BasemapId): string | maplibregl.StyleSpecification {
    if (id === 'dark') {
      return 'https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json';
    }

    if (id === 'voyager') {
      return 'https://basemaps.cartocdn.com/gl/voyager-gl-style/style.json';
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
            tiles: [
              'https://services.arcgisonline.com/ArcGIS/rest/services/Reference/World_Boundaries_and_Places/MapServer/tile/{z}/{y}/{x}'
            ],
            tileSize: 256,
            maxzoom: 19
          }
        },
        layers: [{ id: 'basemap-base-layer', type: 'raster', source: 'basemap-base' }]
      };
    }

    if (id === 'cyclosm') {
      return {
        version: 8,
        sources: {
          'basemap-base': {
            type: 'raster',
            tiles: [
              'https://a.tile-cyclosm.openstreetmap.fr/cyclosm/{z}/{x}/{y}.png',
              'https://b.tile-cyclosm.openstreetmap.fr/cyclosm/{z}/{x}/{y}.png',
              'https://c.tile-cyclosm.openstreetmap.fr/cyclosm/{z}/{x}/{y}.png'
            ],
            tileSize: 256,
            maxzoom: 20,
            attribution:
              '<a href="https://github.com/cyclosm/cyclosm-cartocss-style/releases" target="_blank">CyclOSM</a> | &copy; OpenStreetMap contributors'
          }
        },
        layers: [
          { id: 'basemap-bg', type: 'background', paint: { 'background-color': '#f8fafc' } },
          {
            id: 'basemap-base-layer',
            type: 'raster',
            source: 'basemap-base',
            paint: { 'raster-opacity': 0.78 }
          }
        ]
      };
    }

    // Default: 'light' Dignified Civic Light Vector Canvas (Positron GL)
    return 'https://basemaps.cartocdn.com/gl/positron-gl-style/style.json';
  }

  function applyPlaceNamesVisibility(show: boolean) {
    if (!map || !map.isStyleLoaded()) return;
    const vis = show ? 'visible' : 'none';
    if (map.getLayer('basemap-labels-layer')) {
      map.setLayoutProperty('basemap-labels-layer', 'visibility', vis);
    }
    const layers = map.getStyle()?.layers ?? [];
    for (const l of layers) {
      if (l.type === 'symbol' && !l.id.startsWith('los-')) {
        map.setLayoutProperty(l.id, 'visibility', vis);
      }
    }
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

  function clearHoverStates() {
    if (!map) return;
    if (hoveredSegId !== null) {
      map.setFeatureState({ source: 'los-network', id: hoveredSegId }, { hover: false });
      hoveredSegId = null;
    }
    if (hoveredJuncId !== null) {
      map.setFeatureState({ source: 'los-junctions', id: hoveredJuncId }, { hover: false });
      hoveredJuncId = null;
    }
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
      if (geojson) updateSegmentsSource(geojson);
      if (junctionsGeojson) updateJunctionsSource(junctionsGeojson);
    });

    // Unified pointer hover across Junction polygons & Street segments
    map.on('mousemove', (e) => {
      if (!map || !map.isStyleLoaded()) return;
      const queryLayers: string[] = [];
      if (showJunctions && map.getLayer('los-junctions-fill')) {
        queryLayers.push('los-junctions-fill');
      }
      if (map.getLayer('los-network-lines')) {
        queryLayers.push('los-network-lines');
      }
      if (queryLayers.length === 0) return;

      const hits = map.queryRenderedFeatures(e.point, { layers: queryLayers });
      if (!hits || hits.length === 0) {
        map.getCanvas().style.cursor = '';
        clearHoverStates();
        hoverPopup?.remove();
        return;
      }

      map.getCanvas().style.cursor = 'pointer';
      const feat = hits[0];
      const p = feat.properties || {};
      const isJunction = feat.layer.id === 'los-junctions-fill' || p.layer_type === 'junction';

      if (isJunction) {
        if (hoveredSegId !== null) {
          map.setFeatureState({ source: 'los-network', id: hoveredSegId }, { hover: false });
          hoveredSegId = null;
        }
        if (hoveredJuncId !== null && hoveredJuncId !== feat.id) {
          map.setFeatureState({ source: 'los-junctions', id: hoveredJuncId }, { hover: false });
        }
        if (feat.id !== undefined) {
          hoveredJuncId = feat.id;
          map.setFeatureState({ source: 'los-junctions', id: feat.id }, { hover: true });
        }

        if (!isTouchDevice && hoverPopup) {
          const tone = gradeTone(p.active_grade);
          const shortLos = formatShortLosLabel(undefined, p.active_grade);
          const html = `
            <div class="map-popup-card">
              <div class="map-badge-group">
                <span class="map-badge ${tone}">${escapeHtml(shortLos)}</span>
                <span class="map-badge civic">${escapeHtml(p.control_type)}</span>
              </div>
              <div class="map-popup-title">${escapeHtml(p.name || 'Junction')}</div>
              <div class="map-popup-table">
                <div class="map-cell"><span class="k">MMLoS / W / Wh / C</span><span class="v font-mono">${escapeHtml(p.grade_multimodal)} · ${escapeHtml(p.grade_walking)} · ${escapeHtml(p.grade_wheeling)} · ${escapeHtml(p.grade_cycling)}</span></div>
                <div class="map-cell"><span class="k">Approach max</span><span class="v font-mono">${escapeHtml(p.max_speed_mph)} mph · ${Number(p.max_aadt || 0).toLocaleString()} AADT</span></div>
              </div>
            </div>
          `;
          hoverPopup.setLngLat(e.lngLat).setHTML(html).addTo(map);
        }
      } else {
        if (hoveredJuncId !== null) {
          map.setFeatureState({ source: 'los-junctions', id: hoveredJuncId }, { hover: false });
          hoveredJuncId = null;
        }
        if (hoveredSegId !== null && hoveredSegId !== feat.id) {
          map.setFeatureState({ source: 'los-network', id: hoveredSegId }, { hover: false });
        }
        if (feat.id !== undefined) {
          hoveredSegId = feat.id;
          map.setFeatureState({ source: 'los-network', id: feat.id }, { hover: true });
        }

        if (!isTouchDevice && hoverPopup) {
          const tone = gradeTone(p.active_grade);
          const shortLos = formatShortLosLabel(p.los_label, p.active_grade);
          const spdText =
            p.speed_limit_mph != null ? `${escapeHtml(p.speed_limit_mph)} mph` : 'Traffic-free';
          const facilityBadge =
            colorMode === 'walking_category'
              ? p.walking_category || p.infrastructure
              : colorMode === 'cycling_category'
                ? p.cycling_category || p.infrastructure
                : p.infrastructure;
          const html = `
            <div class="map-popup-card">
              <div class="map-badge-group">
                <span class="map-badge ${tone}">${escapeHtml(shortLos)}</span>
                <span class="map-badge">${escapeHtml(facilityBadge)}</span>
              </div>
              <div class="map-popup-title">${escapeHtml(p.name || 'Unnamed Street')}</div>
              <div class="map-popup-table">
                <div class="map-cell"><span class="k">MMLoS / W / Wh / C</span><span class="v font-mono">${escapeHtml(p.grade_multimodal)} · ${escapeHtml(p.grade_walking)} · ${escapeHtml(p.grade_wheeling)} · ${escapeHtml(p.grade_cycling)}</span></div>
                <div class="map-cell"><span class="k">Speed / Flow</span><span class="v font-mono">${spdText}${p.estimated_aadt != null ? ' · ' + Number(p.estimated_aadt).toLocaleString() + ' AADT' : ''}</span></div>
              </div>
            </div>
          `;
          hoverPopup.setLngLat(e.lngLat).setHTML(html).addTo(map);
        }
      }
    });

    // Unified click handler across Junction polygons & Street segments
    map.on('click', (e) => {
      if (!map || !map.isStyleLoaded()) return;
      const queryLayers: string[] = [];
      if (showJunctions && map.getLayer('los-junctions-fill')) {
        queryLayers.push('los-junctions-fill');
      }
      if (map.getLayer('los-network-lines')) {
        queryLayers.push('los-network-lines');
      }
      if (queryLayers.length === 0) return;

      const hits = map.queryRenderedFeatures(e.point, { layers: queryLayers });
      if (!hits || hits.length === 0) return;

      hoverPopup?.remove();
      const feat = hits[0];
      const p = feat.properties || {};
      const isJunction = feat.layer.id === 'los-junctions-fill' || p.layer_type === 'junction';

      if (isJunction) {
        const center: [number, number] =
          p.center_lng != null && p.center_lat != null
            ? [Number(p.center_lng), Number(p.center_lat)]
            : featureCenter(feat.geometry) ?? [e.lngLat.lng, e.lngLat.lat];
        const bbox = featureBbox(feat.geometry);

        const junc: SelectedJunction = {
          kind: 'junction',
          junction_id: Number(p.junction_id || 0),
          name: String(p.name || 'Junction'),
          arm_count: Number(p.arm_count || 3),
          control_type: String(p.control_type || 'Unmarked priority junction'),
          is_roundabout: p.is_roundabout === true || p.is_roundabout === 'true',
          has_tactile_paving: p.has_tactile_paving === true || p.has_tactile_paving === 'true',
          max_speed_mph: Number(p.max_speed_mph || 20),
          max_aadt: Number(p.max_aadt || 0),
          radius_m: Number(p.radius_m || 12),
          grade_multimodal: String(p.grade_multimodal || 'C'),
          grade_walking: String(p.grade_walking || 'C'),
          grade_wheeling: String(p.grade_wheeling || 'C'),
          grade_cycling: String(p.grade_cycling || 'C'),
          active_grade: String(p.active_grade || 'C'),
          los_color: String(p.los_color || '#eab308'),
          center,
          bbox
        };

        onSelectFeature(junc);

        const tone = gradeTone(junc.active_grade);
        const shortLos = formatShortLosLabel(undefined, junc.active_grade);
        const html = `
          <div class="map-popup-card">
            <div class="map-badge-group">
              <span class="map-badge ${tone}">${escapeHtml(shortLos)}</span>
              <span class="map-badge civic">${escapeHtml(junc.control_type)}</span>
            </div>
            <div class="map-popup-title">${escapeHtml(junc.name)}</div>
            <div class="map-popup-table">
              <div class="map-cell"><span class="k">MMLoS / W / Wh / C</span><span class="v font-mono">${escapeHtml(junc.grade_multimodal)} · ${escapeHtml(junc.grade_walking)} · ${escapeHtml(junc.grade_wheeling)} · ${escapeHtml(junc.grade_cycling)}</span></div>
              <div class="map-cell"><span class="k">Approach arms</span><span class="v font-mono">${junc.arm_count} arms · ${junc.max_speed_mph} mph</span></div>
              <div class="map-cell"><span class="k">Max flow</span><span class="v font-mono">${junc.max_aadt.toLocaleString()} AADT</span></div>
            </div>
            ${mapButtonsHtml(center[0], center[1], null)}
          </div>
        `;

        clickPopup?.remove();
        clickPopup = new maplibregl.Popup({ offset: 12, maxWidth: '295px' })
          .setLngLat(e.lngLat)
          .setHTML(html)
          .addTo(map);
      } else {
        const center = featureCenter(feat.geometry) ?? [e.lngLat.lng, e.lngLat.lat];
        const bbox = featureBbox(feat.geometry);

        const seg: SelectedSegment = {
          kind: 'segment',
          osm_id: Number(p.osm_id),
          name: String(p.name || 'Unnamed Street'),
          highway: String(p.highway || 'road'),
          infrastructure: String(p.infrastructure || ''),
          walking_category: String(p.walking_category || ''),
          cycling_category: String(p.cycling_category || ''),
          prow_designation: p.prow_designation ? String(p.prow_designation) : null,
          rural_surface: String(p.rural_surface || 'Bound paved'),
          is_urban: p.is_urban === true || p.is_urban === 'true',
          separation: String(p.separation || 'none'),
          surface: String(p.surface || 'Smooth paved'),
          width_m: p.width_m != null ? Number(p.width_m) : null,
          incline_pct: p.incline_pct != null ? Number(p.incline_pct) : null,
          is_lit: p.is_lit === true || p.is_lit === 'true',
          speed_limit_mph: p.speed_limit_mph != null ? Number(p.speed_limit_mph) : null,
          speed_category: String(p.speed_category || ''),
          estimated_aadt: p.estimated_aadt != null ? Number(p.estimated_aadt) : null,
          aadt_category: String(p.aadt_category || ''),
          distance_to_road_m: p.distance_to_road_m != null ? Number(p.distance_to_road_m) : null,
          length_m: Number(p.length_m || 0),
          grade_multimodal: String(p.grade_multimodal || 'C'),
          grade_walking: String(p.grade_walking || 'C'),
          grade_wheeling: String(p.grade_wheeling || 'C'),
          grade_cycling: String(p.grade_cycling || 'C'),
          active_grade: String(p.active_grade || 'C'),
          cbd_label: String(p.cbd_label || ''),
          los_score: Number(p.los_score ?? 0),
          los_label: String(p.los_label || ''),
          los_color: String(p.los_color || '#059669'),
          center,
          bbox
        };

        onSelectFeature(seg);

        const tone = gradeTone(seg.active_grade);
        const shortLos = formatShortLosLabel(seg.los_label, seg.active_grade);
        const prowBadge = seg.prow_designation
          ? `<span class="map-badge">${escapeHtml(seg.prow_designation)}</span>`
          : '';
        const html = `
          <div class="map-popup-card">
            <div class="map-badge-group">
              <span class="map-badge ${tone}">${escapeHtml(shortLos)}</span>
              <span class="map-badge civic">${escapeHtml(seg.highway)}</span>
              ${prowBadge}
            </div>
            <div class="map-popup-title">${escapeHtml(seg.name)}</div>
            <div class="map-popup-table">
              <div class="map-cell"><span class="k">MMLoS / W / Wh / C</span><span class="v font-mono">${escapeHtml(seg.grade_multimodal)} · ${escapeHtml(seg.grade_walking)} · ${escapeHtml(seg.grade_wheeling)} · ${escapeHtml(seg.grade_cycling)}</span></div>
              <div class="map-cell"><span class="k">Walk / Cycle</span><span class="v">${escapeHtml(seg.walking_category)} · ${escapeHtml(seg.cycling_category)}</span></div>
              <div class="map-cell"><span class="k">Speed / Flow</span><span class="v font-mono">${seg.speed_limit_mph != null ? seg.speed_limit_mph + ' mph' : 'Traffic-free'}${seg.estimated_aadt != null ? ' · ' + seg.estimated_aadt.toLocaleString() + ' AADT' : ''}</span></div>
              <div class="map-cell"><span class="k">Length</span><span class="v font-mono">${seg.length_m} m</span></div>
            </div>
            ${mapButtonsHtml(center[0], center[1], seg.osm_id)}
          </div>
        `;

        clickPopup?.remove();
        clickPopup = new maplibregl.Popup({ offset: 12, maxWidth: '295px' })
          .setLngLat(e.lngLat)
          .setHTML(html)
          .addTo(map);
      }
    });
  });

  function initLayers() {
    if (!map || !map.isStyleLoaded()) return;

    // Insert analyzed network & junction layers beneath vector symbol labels so street/town names stay crisp on top
    const firstSymbolId = map
      .getStyle()
      ?.layers?.find((l) => l.type === 'symbol' && !l.id.startsWith('los-'))?.id;

    if (!map.getSource('los-network')) {
      map.addSource('los-network', {
        type: 'geojson',
        generateId: true,
        data: geojson || { type: 'FeatureCollection', features: [] }
      });
    }

    if (!map.getSource('los-junctions')) {
      map.addSource('los-junctions', {
        type: 'geojson',
        generateId: true,
        data: junctionsGeojson || { type: 'FeatureCollection', features: [] }
      });
    }

    const filterExpr = getFilterExpression(colorMode, hiddenValues);

    if (!map.getLayer('los-network-casing')) {
      map.addLayer(
        {
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
        },
        firstSymbolId
      );
    }

    if (!map.getLayer('los-network-lines')) {
      map.addLayer(
        {
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
        },
        firstSymbolId
      );
    }

    const juncVisibility = showJunctions ? 'visible' : 'none';

    if (!map.getLayer('los-junctions-fill')) {
      map.addLayer(
        {
          id: 'los-junctions-fill',
          type: 'fill',
          source: 'los-junctions',
          minzoom: 12,
          layout: {
            visibility: juncVisibility
          },
          paint: {
            'fill-color': ['coalesce', ['get', 'los_color'], '#eab308'],
            'fill-opacity': [
              'case',
              ['boolean', ['feature-state', 'hover'], false],
              0.52,
              0.28
            ]
          }
        },
        firstSymbolId
      );
    }

    if (!map.getLayer('los-junctions-outline')) {
      map.addLayer(
        {
          id: 'los-junctions-outline',
          type: 'line',
          source: 'los-junctions',
          minzoom: 12,
          layout: {
            visibility: juncVisibility,
            'line-join': 'round'
          },
          paint: {
            'line-color': ['coalesce', ['get', 'los_color'], '#eab308'],
            'line-width': [
              'case',
              ['boolean', ['feature-state', 'hover'], false],
              2.6,
              1.6
            ],
            'line-opacity': 0.92
          }
        },
        firstSymbolId
      );
    }

    // Satellite raster label overlay (when using Esri Satellite basemap)
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

    applyPlaceNamesVisibility(showPlaceNames);
  }

  function updateSegmentsSource(data: FeatureCollection | null) {
    if (!map) return;
    if (!map.isStyleLoaded()) {
      map.once('style.load', () => updateSegmentsSource(data));
      return;
    }
    initLayers();
    const src = map.getSource('los-network') as maplibregl.GeoJSONSource | undefined;
    if (src) {
      src.setData(data || { type: 'FeatureCollection', features: [] });
    }
  }

  function updateJunctionsSource(data: FeatureCollection | null) {
    if (!map) return;
    if (!map.isStyleLoaded()) {
      map.once('style.load', () => updateJunctionsSource(data));
      return;
    }
    initLayers();
    const src = map.getSource('los-junctions') as maplibregl.GeoJSONSource | undefined;
    if (src) {
      src.setData(data || { type: 'FeatureCollection', features: [] });
    }
  }

  let appliedBasemap: BasemapId | null = null;

  $effect(() => {
    const nextBasemap = basemap;
    if (map && appliedBasemap !== null && nextBasemap !== appliedBasemap) {
      appliedBasemap = nextBasemap;
      hoveredSegId = null;
      hoveredJuncId = null;
      map.setStyle(buildStyleSpec(nextBasemap));
      map.once('style.load', () => {
        initLayers();
        updateSegmentsSource(geojson);
        updateJunctionsSource(junctionsGeojson);
      });
    }
  });

  $effect(() => {
    const data = geojson;
    if (data && map) {
      updateSegmentsSource(data);
    }
  });

  $effect(() => {
    const jData = junctionsGeojson;
    if (jData && map) {
      updateJunctionsSource(jData);
    }
  });

  $effect(() => {
    const showJ = showJunctions;
    if (!map || !map.isStyleLoaded()) return;
    const vis = showJ ? 'visible' : 'none';
    for (const layerId of ['los-junctions-fill', 'los-junctions-outline']) {
      if (map.getLayer(layerId)) {
        map.setLayoutProperty(layerId, 'visibility', vis);
      }
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
    applyPlaceNamesVisibility(show);
  });

  export function flyToBounds(bbox: [number, number, number, number], skipIfHashPresent = false) {
    if (!map) return;
    if (skipIfHashPresent && window.location.hash.includes('map=')) return;
    map.fitBounds(
      [
        [bbox[0], bbox[1]],
        [bbox[2], bbox[3]]
      ],
      { padding: viewportPadding(), maxZoom: 17.2, duration: 1200 }
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
