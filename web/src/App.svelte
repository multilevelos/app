<script lang="ts">
  import { onMount } from 'svelte';
  import type { FeatureCollection } from 'geojson';
  import type {
    BasemapId,
    ColorMode,
    LosConfig,
    PresetLocation,
    SearchResult,
    SelectedSegment,
    SummaryStats,
    ThemeMode
  } from './types';
  import { engineInstance, ensureWasmInitialized } from './lib/engine';
  import { fetchOsmNetworkForBbox, sanitizeBbox } from './lib/overpass';
  import { buildLegendMetrics, PRESET_LOCATIONS, SCHEMES } from './lib/palette';
  import { buildShareUrl, readUrlState, syncUrlStateDebounced } from './lib/urlState';
  import Header from './lib/components/Header.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import DetailPanel from './lib/components/DetailPanel.svelte';
  import PrintLegend from './lib/components/PrintLegend.svelte';
  import MapContainer from './lib/map/MapContainer.svelte';

  const initialUrl = readUrlState();
  const isMobileInit = typeof window !== 'undefined' && window.innerWidth <= 768;

  const baseConfig = engineInstance.getDefaultConfig();
  if (initialUrl.trafficMultiplier !== undefined) {
    baseConfig.traffic_multiplier = initialUrl.trafficMultiplier;
  }
  if (initialUrl.includeMixed !== undefined) {
    baseConfig.include_mixed_traffic = initialUrl.includeMixed;
  }
  if (initialUrl.minOffroad !== undefined) {
    baseConfig.min_offroad_distance = initialUrl.minOffroad;
  }
  if (initialUrl.minWide !== undefined) {
    baseConfig.min_wide_width = initialUrl.minWide;
  }

  let config = $state<LosConfig>(baseConfig);

  let geojson = $state<FeatureCollection | null>(null);
  let summary = $state<SummaryStats | null>(null);
  let colorMode = $state<ColorMode>(initialUrl.colorMode ?? 'los');
  let hiddenValues = $state<string[]>(initialUrl.hiddenValues ?? []);
  let basemap = $state<BasemapId>(initialUrl.basemap ?? 'light');
  let showPlaceNames = $state<boolean>(initialUrl.showPlaceNames ?? true);
  let theme = $state<ThemeMode>(initialUrl.theme ?? 'light');
  let sidebarOpen = $state<boolean>(initialUrl.sidebarOpen ?? !isMobileInit);
  let printMode = $state<'legend' | 'clean' | null>(initialUrl.printMode ?? null);

  let selectedPresetId = $state<string | null>(initialUrl.preset ?? 'edinburgh');
  let areaLabel = $state<string>('Edinburgh');
  let selectedSegment = $state<SelectedSegment | null>(null);

  let isLoading = $state(true);
  let loadingMessage = $state('Initializing WebAssembly engine...');
  let toast = $state<{ text: string; kind: 'ok' | 'error' } | null>(null);
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  let mapComponent = $state<MapContainer>();

  let legendRows = $derived(
    buildLegendMetrics(colorMode, summary, geojson?.features ?? [])
  );

  function notify(text: string, kind: 'ok' | 'error' = 'ok', durationMs = 4200) {
    if (toastTimer) clearTimeout(toastTimer);
    toast = { text, kind };
    toastTimer = setTimeout(() => {
      toast = null;
    }, durationMs);
  }

  // Keep <html data-theme="..."> synchronized
  $effect(() => {
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.theme = theme;
    }
  });

  // Clear hidden legend filters when switching colour mode so user isn't left with an empty map
  let lastColorMode: ColorMode | null = null;
  $effect(() => {
    if (lastColorMode !== null && colorMode !== lastColorMode) {
      hiddenValues = [];
    }
    lastColorMode = colorMode;
  });

  // Debounced URL state sync
  $effect(() => {
    syncUrlStateDebounced({
      presetId: selectedPresetId,
      colorMode,
      hiddenValues,
      basemap,
      showPlaceNames,
      theme,
      sidebarOpen,
      printMode,
      config
    });
  });

  onMount(async () => {
    const hadMapHash = window.location.hash.includes('map=');
    try {
      loadingMessage = 'Compiling WebAssembly modules...';
      await ensureWasmInitialized();

      const startPreset =
        PRESET_LOCATIONS.find((p) => p.id === selectedPresetId) ?? PRESET_LOCATIONS[0];
      await loadPreset(startPreset, hadMapHash);
    } catch (err: any) {
      console.error('Initialization error:', err);
      notify(`Engine initialization failed: ${err?.message || err}`, 'error', 6500);
    } finally {
      isLoading = false;
    }
  });

  async function loadPreset(preset: PresetLocation, preserveExistingHash = false) {
    isLoading = true;
    selectedPresetId = preset.id;
    areaLabel = preset.name.split(' (')[0];
    selectedSegment = null;
    loadingMessage = `Loading ${preset.name}...`;

    try {
      const base = import.meta.env.BASE_URL || './';
      const cleanBase = base.endsWith('/') ? base : `${base}/`;
      const response = await fetch(`${cleanBase}presets/${preset.id}.json`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }

      const osmText = await response.text();
      loadingMessage = 'Computing Level of Service in Rust WASM...';
      await engineInstance.initFromOsmJson(osmText);

      const result = engineInstance.calculateLos(config);
      geojson = result.geojson;
      summary = result.summary;

      mapComponent?.flyToBounds(preset.bbox, preserveExistingHash);
      notify(
        `${preset.name}: ${result.segments_count.toLocaleString()} links (${result.summary.total_length_km} km)`
      );
    } catch (err: any) {
      console.error('Preset load failed:', err);
      notify(`Could not load ${preset.name}: ${err?.message || err}`, 'error', 6000);
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
    selectedSegment = null;
    loadingMessage = 'Downloading OpenStreetMap ways from Overpass...';

    try {
      const osmText = await fetchOsmNetworkForBbox(minLng, minLat, maxLng, maxLat);
      loadingMessage = 'Running Level of Service analysis in Rust WASM...';
      await engineInstance.initFromOsmJson(osmText);

      const result = engineInstance.calculateLos(config);
      geojson = result.geojson;
      summary = result.summary;
      selectedPresetId = null;
      areaLabel = 'Live OSM View';

      if (result.segments_count === 0) {
        notify('No street or path segments found in this area. Zoom into a town or city.', 'error');
      } else {
        notify(
          `Fetched live OSM: ${result.segments_count.toLocaleString()} links (${result.summary.total_length_km} km)`
        );
      }
    } catch (err: any) {
      console.error('Fetch viewport error:', err);
      notify(`Overpass download failed: ${err?.message || err}`, 'error', 6000);
    } finally {
      isLoading = false;
    }
  }

  async function handleSearchSelect(result: SearchResult) {
    const minLat = parseFloat(result.boundingbox[0]);
    const maxLat = parseFloat(result.boundingbox[1]);
    const minLng = parseFloat(result.boundingbox[2]);
    const maxLng = parseFloat(result.boundingbox[3]);

    const [w, s, e, n] = sanitizeBbox(minLng, minLat, maxLng, maxLat, 0.05);
    const placeName = result.display_name.split(',')[0].trim();

    mapComponent?.flyToBounds([w, s, e, n]);
    isLoading = true;
    selectedSegment = null;
    loadingMessage = `Downloading OSM network for ${placeName}...`;

    try {
      const osmText = await fetchOsmNetworkForBbox(w, s, e, n);
      loadingMessage = 'Computing Level of Service in Rust WASM...';
      await engineInstance.initFromOsmJson(osmText);

      const res = engineInstance.calculateLos(config);
      geojson = res.geojson;
      summary = res.summary;
      selectedPresetId = null;
      areaLabel = placeName;

      notify(`${placeName}: ${res.segments_count.toLocaleString()} links (${res.summary.total_length_km} km)`);
    } catch (err: any) {
      console.error('Search load error:', err);
      notify(`Could not fetch OSM for ${placeName}: ${err?.message || err}`, 'error', 6000);
    } finally {
      isLoading = false;
    }
  }

  async function handleFileUpload(file: File) {
    isLoading = true;
    selectedSegment = null;
    loadingMessage = `Parsing ${file.name}...`;

    try {
      const osmText = await file.text();
      await engineInstance.initFromOsmJson(osmText);
      const result = engineInstance.calculateLos(config);
      geojson = result.geojson;
      summary = result.summary;
      selectedPresetId = null;
      areaLabel = file.name.replace(/\.(geo)?json$/i, '');

      if (geojson && geojson.features.length > 0) {
        let minX = 180,
          minY = 90,
          maxX = -180,
          maxY = -90;
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
          mapComponent?.flyToBounds([minX, minY, maxX, maxY]);
        }
      }
      notify(`Loaded ${file.name}: ${result.segments_count.toLocaleString()} links (${result.summary.total_length_km} km)`);
    } catch (err: any) {
      console.error('Upload error:', err);
      notify(`Invalid Overpass JSON file: ${err?.message || err}`, 'error', 6000);
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
    } catch (err: any) {
      console.error('Recalculation error:', err);
    }
  }

  function handleResetConfig() {
    config = engineInstance.getDefaultConfig();
    handleConfigChange();
    notify('Reset traffic & geometry parameters to defaults');
  }

  function handleToggleLegendItem(value: string) {
    if (hiddenValues.includes(value)) {
      hiddenValues = hiddenValues.filter((v) => v !== value);
    } else {
      hiddenValues = [...hiddenValues, value];
    }
  }

  function handleIsolateLegendItem(value: string) {
    const allValues = SCHEMES[colorMode].entries.map((e) => e.value);
    const others = allValues.filter((v) => v !== value);
    // Clicking 'only' again when already isolated restores all
    if (hiddenValues.length === others.length && !hiddenValues.includes(value)) {
      hiddenValues = [];
    } else {
      hiddenValues = others;
    }
  }

  function handleZoomToSegment(seg: SelectedSegment) {
    if (seg.bbox) {
      mapComponent?.flyToBounds(seg.bbox);
    }
  }

  async function handleCopyShareLink() {
    const url = buildShareUrl({
      presetId: selectedPresetId,
      colorMode,
      hiddenValues,
      basemap,
      showPlaceNames,
      theme,
      sidebarOpen,
      printMode,
      config
    });
    await navigator.clipboard.writeText(url);
    notify('Shareable view link copied to clipboard');
  }

  function handleExportGeoJSON() {
    if (!geojson) return;
    const blob = new Blob([JSON.stringify(geojson, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `multilevelos_${areaLabel.toLowerCase().replace(/\s+/g, '_')}_${new Date().toISOString().slice(0, 10)}.geojson`;
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
    a.download = `multilevelos_summary_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleGlobalKeydown(e: KeyboardEvent) {
    const tag = (e.target as HTMLElement | null)?.tagName;
    if (tag === 'INPUT' || tag === 'SELECT' || tag === 'TEXTAREA') return;
    if (e.key === 'p' || e.key === 'P') {
      printMode = printMode ? null : 'legend';
    } else if (e.key === 'Escape') {
      if (printMode) printMode = null;
      else if (selectedSegment) selectedSegment = null;
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

{#if !printMode}
  <Header
    {sidebarOpen}
    onToggleSidebar={() => (sidebarOpen = !sidebarOpen)}
    {areaLabel}
    {isLoading}
    {basemap}
    {showPlaceNames}
    {theme}
    onChangeBasemap={(b) => (basemap = b)}
    onTogglePlaceNames={() => (showPlaceNames = !showPlaceNames)}
    onToggleTheme={() => {
      const next = theme === 'light' ? 'dark' : 'light';
      theme = next;
      if (next === 'dark' && basemap === 'light') basemap = 'dark';
      else if (next === 'light' && basemap === 'dark') basemap = 'light';
    }}
    onSelectPlace={handleSearchSelect}
    onFetchViewport={handleFetchViewport}
    onEnterPrintView={() => (printMode = 'legend')}
    onCopyShareLink={handleCopyShareLink}
  />
{/if}

<main class="workspace">
  {#if !printMode}
    <Sidebar
      open={sidebarOpen}
      bind:config
      {summary}
      bind:colorMode
      {legendRows}
      {hiddenValues}
      {selectedPresetId}
      {isLoading}
      {loadingMessage}
      onCloseMobile={() => (sidebarOpen = false)}
      onToggleLegendItem={handleToggleLegendItem}
      onIsolateLegendItem={handleIsolateLegendItem}
      onShowAllLegend={() => (hiddenValues = [])}
      onSelectPreset={(p) => loadPreset(p, false)}
      onConfigChange={handleConfigChange}
      onResetConfig={handleResetConfig}
      onExportGeoJSON={handleExportGeoJSON}
      onExportCSV={handleExportCSV}
      onFileUpload={handleFileUpload}
    />
  {/if}

  <div class="map-stage">
    <MapContainer
      bind:this={mapComponent}
      {geojson}
      {colorMode}
      {hiddenValues}
      {basemap}
      {showPlaceNames}
      sidebarOpen={!printMode && sidebarOpen}
      onSelectSegment={(seg) => (selectedSegment = seg)}
    />

    {#if !printMode}
      <DetailPanel
        segment={selectedSegment}
        onClose={() => (selectedSegment = null)}
        onZoomTo={handleZoomToSegment}
      />
    {:else}
      <PrintLegend
        mode={printMode}
        {areaLabel}
        {colorMode}
        rows={legendRows}
        {hiddenValues}
        {summary}
        {config}
        onChangeMode={(m) => (printMode = m)}
        onDownloadPng={() =>
          mapComponent?.exportPng(
            `multilevelos_${areaLabel.toLowerCase().replace(/\s+/g, '_')}.png`
          )}
        onPrint={() => window.print()}
        onExit={() => (printMode = null)}
      />
    {/if}

    {#if toast}
      <div class="toast-banner panel-card" class:err={toast.kind === 'error'} role="status">
        {#if toast.kind === 'error'}
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" aria-hidden="true">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
        {:else}
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" aria-hidden="true">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
        {/if}
        <span>{toast.text}</span>
      </div>
    {/if}
  </div>
</main>

<style>
  .workspace {
    position: relative;
    flex: 1;
    width: 100%;
    min-height: 0;
    overflow: hidden;
  }

  .map-stage {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .toast-banner {
    position: absolute;
    top: 14px;
    right: 54px;
    z-index: 45;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 13px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    border-left: 3px solid var(--accent-emerald);
    max-width: min(420px, calc(100vw - 80px));
  }

  .toast-banner svg {
    color: var(--accent-emerald);
    flex-shrink: 0;
  }

  .toast-banner.err {
    border-left-color: var(--accent-rose);
  }

  .toast-banner.err svg {
    color: var(--accent-rose);
  }
</style>
