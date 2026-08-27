<script lang="ts">
  import { onMount } from 'svelte';
  import type { FeatureCollection } from 'geojson';
  import type { ColorMode, LosConfig, PresetLocation, SearchResult, SummaryStats } from './types';
  import { engineInstance, ensureWasmInitialized } from './lib/engine';
  import { fetchOsmNetworkForBbox, sanitizeBbox } from './lib/overpass';
  import MapContainer from './lib/map/MapContainer.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import Legend from './lib/components/Legend.svelte';

  let config = $state<LosConfig>(engineInstance.getDefaultConfig());
  let geojson = $state<FeatureCollection | null>(null);
  let summary = $state<SummaryStats | null>(null);
  let colorMode = $state<ColorMode>('los');
  let isLoading = $state(true);
  let loadingMessage = $state('Initializing WebAssembly engine...');
  let rawOsmJson = $state<string | null>(null);
  let notificationToast = $state<string | null>(null);
  let toastTimeout: any = null;

  let mapComponent: MapContainer;

  function showToast(msg: string, durationMs: number = 4000) {
    if (toastTimeout) clearTimeout(toastTimeout);
    notificationToast = msg;
    toastTimeout = setTimeout(() => {
      notificationToast = null;
    }, durationMs);
  }

  onMount(async () => {
    try {
      loadingMessage = 'Compiling WebAssembly modules...';
      await ensureWasmInitialized();

      // Load Edinburgh default preset
      await loadPreset({
        id: 'edinburgh',
        name: 'Edinburgh (City Centre)',
        country: 'Scotland, UK',
        description: 'Extensive 20mph network, off-road railway paths, and protected lanes.',
        center: [-3.195, 55.952],
        zoom: 14.5,
        bbox: [-3.22, 55.94, -3.17, 55.965]
      });
    } catch (err: any) {
      console.error('Initialization error:', err);
      loadingMessage = `Error: ${err.message || err}`;
    } finally {
      isLoading = false;
    }
  });

  async function loadPreset(preset: PresetLocation) {
    isLoading = true;
    loadingMessage = `Loading ${preset.name} dataset...`;

    try {
      const base = import.meta.env.BASE_URL || './';
      const cleanBase = base.endsWith('/') ? base : `${base}/`;
      const response = await fetch(`${cleanBase}presets/${preset.id}.json`);

      if (!response.ok) {
        throw new Error(`Failed to load preset: HTTP ${response.status}`);
      }

      const osmText = await response.text();
      rawOsmJson = osmText;

      loadingMessage = 'Processing road network & computing Level of Service...';
      await engineInstance.initFromOsmJson(osmText);

      const result = engineInstance.calculateLos(config);
      geojson = result.geojson;
      summary = result.summary;

      if (mapComponent) {
        mapComponent.flyToBounds(preset.bbox);
      }
      showToast(`Loaded ${preset.name}: ${result.segments_count} segments (${result.summary.total_length_km} km)`);
    } catch (err: any) {
      console.error('Failed to load preset:', err);
      alert(`Could not load preset ${preset.name}: ${err.message}`);
    } finally {
      isLoading = false;
    }
  }

  async function handleFetchViewport() {
    if (!mapComponent) return;
    const rawBbox = mapComponent.getMapBounds();
    if (!rawBbox) return;

    const [minLng, minLat, maxLng, maxLat] = sanitizeBbox(
      rawBbox[0],
      rawBbox[1],
      rawBbox[2],
      rawBbox[3]
    );

    isLoading = true;
    loadingMessage = `Querying OpenStreetMap Overpass API for area [${minLat.toFixed(3)}, ${minLng.toFixed(3)}]...`;

    try {
      const osmText = await fetchOsmNetworkForBbox(minLng, minLat, maxLng, maxLat);
      rawOsmJson = osmText;

      loadingMessage = 'Running Cycling by Design Level of Service analysis in Rust...';
      await engineInstance.initFromOsmJson(osmText);

      const result = engineInstance.calculateLos(config);
      geojson = result.geojson;
      summary = result.summary;

      if (result.segments_count === 0) {
        alert('No highway/cycleway segments found in this area. Try zooming into a populated town or city.');
      } else {
        showToast(`Fetched OSM data: ${result.segments_count} segments (${result.summary.total_length_km} km analyzed)`);
      }
    } catch (err: any) {
      console.error('Fetch viewport error:', err);
      alert(`Overpass download failed: ${err.message || err}. Please try again in a few seconds.`);
    } finally {
      isLoading = false;
    }
  }

  async function handleSearchSelect(result: SearchResult) {
    const minLat = parseFloat(result.boundingbox[0]);
    const maxLat = parseFloat(result.boundingbox[1]);
    const minLng = parseFloat(result.boundingbox[2]);
    const maxLng = parseFloat(result.boundingbox[3]);

    const centerLng = parseFloat(result.lon);
    const centerLat = parseFloat(result.lat);

    const [w, s, e, n] = sanitizeBbox(minLng, minLat, maxLng, maxLat, 0.05);

    if (mapComponent) {
      mapComponent.flyToBounds([w, s, e, n]);
    }

    isLoading = true;
    const name = result.display_name.split(',')[0];
    loadingMessage = `Downloading OSM network for ${name}...`;

    try {
      const osmText = await fetchOsmNetworkForBbox(w, s, e, n);
      rawOsmJson = osmText;

      loadingMessage = 'Computing Level of Service in Rust WebAssembly...';
      await engineInstance.initFromOsmJson(osmText);

      const res = engineInstance.calculateLos(config);
      geojson = res.geojson;
      summary = res.summary;

      showToast(`Loaded ${name}: ${res.segments_count} segments (${res.summary.total_length_km} km)`);
    } catch (err: any) {
      console.error('Search load error:', err);
      alert(`Failed to fetch OSM data for ${name}: ${err.message}`);
    } finally {
      isLoading = false;
    }
  }

  async function handleFileUpload(file: File) {
    isLoading = true;
    loadingMessage = `Reading ${file.name}...`;

    try {
      const osmText = await file.text();
      rawOsmJson = osmText;

      loadingMessage = 'Parsing OSM data & calculating Level of Service in Rust...';
      await engineInstance.initFromOsmJson(osmText);

      const result = engineInstance.calculateLos(config);
      geojson = result.geojson;
      summary = result.summary;

      // Auto-fit to loaded geometry
      if (geojson && geojson.features.length > 0 && mapComponent) {
        let minX = 180, minY = 90, maxX = -180, maxY = -90;
        for (const f of geojson.features) {
          if (f.geometry && f.geometry.type === 'LineString') {
            for (const pt of f.geometry.coordinates) {
              minX = Math.min(minX, pt[0]);
              minY = Math.min(minY, pt[1]);
              maxX = Math.max(maxX, pt[0]);
              maxY = Math.max(maxY, pt[1]);
            }
          }
        }
        if (minX <= maxX) {
          mapComponent.flyToBounds([minX, minY, maxX, maxY]);
        }
      }
      showToast(`Loaded ${file.name}: ${result.segments_count} segments (${result.summary.total_length_km} km)`);
    } catch (err: any) {
      console.error('File upload error:', err);
      alert(`Invalid OSM file: ${err.message}`);
    } finally {
      isLoading = false;
    }
  }

  function handleConfigChange() {
    if (!engineInstance.isReady()) return;
    try {
      const result = engineInstance.calculateLos(config);
      geojson = result.geojson;
      summary = result.summary;
    } catch (err) {
      console.error('Recalculation error:', err);
    }
  }

  function handleResetConfig() {
    config = engineInstance.getDefaultConfig();
    handleConfigChange();
    showToast('Reset parameters to Scottish Cycling by Design defaults');
  }

  function handleExportGeoJSON() {
    if (!geojson) return;
    const blob = new Blob([JSON.stringify(geojson, null, 2)], {
      type: 'application/json'
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `cycling_level_of_service_${new Date().toISOString().slice(0, 10)}.geojson`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleExportCSV() {
    if (!summary) return;
    let csv = 'Category_Type,Category_Name,Length_km,Percentage,Segments_Count\n';

    for (const [key, val] of Object.entries(summary.los_breakdown)) {
      csv += `"Level of Service","${key}",${val.length_km},${val.percentage}%,${val.count}\n`;
    }

    for (const [key, val] of Object.entries(summary.infra_breakdown)) {
      csv += `"Infrastructure","${key}",${val.length_km},${val.percentage}%,${val.count}\n`;
    }

    csv += `\n"Summary Metric","Total Analyzed Network Length",${summary.total_length_km},"km",${summary.total_segments}\n`;
    csv += `"Summary Metric","Suitable Network (High/Medium)",${summary.high_med_pct}%,,\n`;
    csv += `"Summary Metric","Unsuitable / Non-Compliant",${summary.unsuitable_pct}%,,\n`;
    csv += `"Summary Metric","Average Speed Limit",${summary.average_speed_mph},"mph",\n`;

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `cycling_los_summary_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<main class="app-layout">
  <!-- Left Side Panel Controls -->
  <Sidebar
    bind:config
    {summary}
    bind:colorMode
    {isLoading}
    {loadingMessage}
    onSelectPreset={loadPreset}
    onSearchSelect={handleSearchSelect}
    onConfigChange={handleConfigChange}
    onResetConfig={handleResetConfig}
    onExportGeoJSON={handleExportGeoJSON}
    onExportCSV={handleExportCSV}
    onFileUpload={handleFileUpload}
  />

  <!-- Main Map Canvas -->
  <div class="map-section">
    <MapContainer
      bind:this={mapComponent}
      {geojson}
      {colorMode}
      {isLoading}
      onFetchViewport={handleFetchViewport}
    />

    <!-- Floating Map Legend -->
    <div class="legend-overlay">
      <Legend {colorMode} />
    </div>

    <!-- Notification Toast -->
    {#if notificationToast}
      <div class="toast-popup">
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"/>
        </svg>
        <span>{notificationToast}</span>
      </div>
    {/if}
  </div>
</main>

<style>
  .app-layout {
    display: flex;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: var(--bg-main);
  }

  .map-section {
    position: relative;
    flex: 1;
    height: 100%;
    overflow: hidden;
  }

  .legend-overlay {
    position: absolute;
    bottom: 24px;
    left: 20px;
    z-index: 10;
    pointer-events: none;
  }

  .toast-popup {
    position: absolute;
    top: 16px;
    right: 56px;
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(15, 23, 42, 0.92);
    backdrop-filter: blur(10px);
    border: 1px solid #16a34a;
    color: #f8fafc;
    padding: 8px 14px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 500;
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.4);
    z-index: 20;
    animation: fadeIn 0.2s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
