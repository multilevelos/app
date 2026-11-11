<script lang="ts">
  import KPICards from './KPICards.svelte';
  import Legend from './Legend.svelte';
  import { PRESET_LOCATIONS } from '../palette';
  import type {
    ColorMode,
    DesignStandard,
    LegendMetricRow,
    LosConfig,
    PresetLocation,
    SummaryStats,
    TravelMode
  } from '../../types';

  let {
    open = true,
    config = $bindable(),
    summary = null,
    colorMode = $bindable('los'),
    legendRows = [],
    hiddenValues = [],
    showJunctions = true,
    selectedPresetId = null,
    isLoading = false,
    loadingMessage = '',
    onCloseMobile = () => {},
    onToggleJunctions = () => {},
    onToggleLegendItem,
    onIsolateLegendItem,
    onShowAllLegend,
    onSelectPreset,
    onConfigChange,
    onResetConfig,
    onExportGeoJSON,
    onExportJunctionsGeoJSON = () => {},
    onExportCSV,
    onFileUpload
  }: {
    open: boolean;
    config: LosConfig;
    summary: SummaryStats | null;
    colorMode: ColorMode;
    legendRows: LegendMetricRow[];
    hiddenValues: string[];
    showJunctions?: boolean;
    selectedPresetId: string | null;
    isLoading: boolean;
    loadingMessage: string;
    onCloseMobile?: () => void;
    onToggleJunctions?: () => void;
    onToggleLegendItem: (val: string) => void;
    onIsolateLegendItem: (val: string) => void;
    onShowAllLegend: () => void;
    onSelectPreset: (preset: PresetLocation) => void;
    onConfigChange: () => void;
    onResetConfig: () => void;
    onExportGeoJSON: () => void;
    onExportJunctionsGeoJSON?: () => void;
    onExportCSV: () => void;
    onFileUpload: (file: File) => void;
  } = $props();

  let paramsOpen = $state(false);
  let dataOpen = $state(true);
  let exportOpen = $state(false);
  let fileInput = $state<HTMLInputElement>();

  const MODES: { id: TravelMode; label: string; hint: string }[] = [
    { id: 'multimodal', label: 'Multi-Modal', hint: 'Composite Walking + Wheeling + Cycling MMLoS' },
    { id: 'walking', label: 'Walking', hint: 'Pedestrian footways, crossings, PRoW, lighting & traffic stress' },
    { id: 'wheeling', label: 'Wheeling', hint: 'Wheelchair & step-free accessibility, width, surface & gradient' },
    { id: 'cycling', label: 'Cycling', hint: 'Cycle separation, speed & AADT matrix (LTN 1/20, Rural 2026 or CbD)' }
  ];

  let activeOverridesCount = $derived.by(() => {
    let n = 0;
    if (!config.include_mixed_traffic) n++;
    if (Math.abs(config.traffic_multiplier - 1.0) > 0.01) n++;
    if (Math.abs(config.min_offroad_distance - 10) > 0.1) n++;
    if (Math.abs(config.min_wide_width - 2.0) > 0.01) n++;
    return n;
  });

  function selectMode(m: TravelMode) {
    if (config.active_mode === m) return;
    config.active_mode = m;
    onConfigChange();
  }

  function selectStandard(std: DesignStandard) {
    if (config.standard === std) return;
    config.standard = std;
    onConfigChange();
  }

  function handleFileSelected(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      onFileUpload(target.files[0]);
      target.value = '';
    }
  }
</script>

{#if open}
  <div class="mobile-scrim" onclick={onCloseMobile} aria-hidden="true"></div>
{/if}

<aside class="sidebar-drawer" class:closed={!open} aria-hidden={!open}>
  <div class="mobile-drawer-head">
    <span class="drawer-title">Network Analytics & Controls</span>
    <button type="button" class="outline-btn mobile-map-btn" onclick={onCloseMobile}>
      Map →
    </button>
  </div>

  {#if isLoading}
    <div class="loading-strip">
      <span class="strip-spinner"></span>
      <span>{loadingMessage || 'Processing network in Rust WASM...'}</span>
    </div>
  {/if}

  <div class="drawer-scroll">
    <!-- Multi-Modal Mode Switcher & Design Standard Toggle -->
    <div class="mode-switcher-card">
      <div class="ms-row-head">
        <span class="ms-label">Active Mode</span>
        <div class="std-segmented" role="group" aria-label="Design standard">
          <button
            type="button"
            class="std-btn"
            class:active={config.standard === 'ltn120'}
            onclick={() => selectStandard('ltn120')}
            title="DfT Local Transport Note 1/20 (England)"
          >
            LTN 1/20
          </button>
          <button
            type="button"
            class="std-btn"
            class:active={config.standard === 'rural2026'}
            onclick={() => selectStandard('rural2026')}
            title="Active Travel England Rural Design Guide (July 2026) — Table 4-3, PRoW & surfacing"
          >
            Rural 2026
          </button>
          <button
            type="button"
            class="std-btn"
            class:active={config.standard === 'cbd'}
            onclick={() => selectStandard('cbd')}
            title="Transport Scotland Cycling by Design (Scotland)"
          >
            CbD
          </button>
        </div>
      </div>

      <div class="mode-pills" role="tablist" aria-label="Travel mode">
        {#each MODES as m (m.id)}
          <button
            type="button"
            role="tab"
            aria-selected={config.active_mode === m.id}
            class="mode-pill"
            class:active={config.active_mode === m.id}
            onclick={() => selectMode(m.id)}
            title={m.hint}
          >
            {m.label}
          </button>
        {/each}
      </div>
    </div>

    <!-- Always-On Headline KPIs, Composition Bar & Cross-Modal Suitability -->
    <KPICards
      {summary}
      rows={legendRows}
      activeMode={config.active_mode}
      onSelectMode={selectMode}
    />

    <!-- Actionable Interactive Map Legend + Junctions Toggle -->
    <Legend
      bind:colorMode
      rows={legendRows}
      {hiddenValues}
      {showJunctions}
      junctionStats={summary?.junctions ?? null}
      {onToggleJunctions}
      onToggleItem={onToggleLegendItem}
      onIsolateItem={onIsolateLegendItem}
      onShowAll={onShowAllLegend}
    />

    <!-- Accordion 1: Model Parameters -->
    <div class="accordion">
      <button
        type="button"
        class="acc-header"
        onclick={() => (paramsOpen = !paramsOpen)}
        aria-expanded={paramsOpen}
      >
        <span class="acc-title">
          Traffic & LoS Parameters
          {#if activeOverridesCount > 0}
            <span class="count-pill">{activeOverridesCount}</span>
          {/if}
        </span>
        <svg class="chev" class:rotated={paramsOpen} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </button>

      {#if paramsOpen}
        <div class="acc-body">
          <label class="toggle-row">
            <span>Include Mixed Traffic Streets</span>
            <input
              type="checkbox"
              bind:checked={config.include_mixed_traffic}
              onchange={onConfigChange}
            />
          </label>
          <p class="hint">When unchecked, only dedicated tracks, footways, lanes, and off-road paths are scored.</p>

          <div class="sub-divider"></div>

          <div class="slider-group">
            <div class="s-head">
              <span>Global Traffic Scaling</span>
              <span class="s-val font-mono">{config.traffic_multiplier.toFixed(1)}x</span>
            </div>
            <input
              type="range"
              min="0.2"
              max="3.0"
              step="0.1"
              bind:value={config.traffic_multiplier}
              oninput={onConfigChange}
            />
          </div>

          <div class="slider-group">
            <div class="s-head">
              <span>Primary Roads AADT</span>
              <span class="s-val font-mono">{(config.aadt_estimates['primary'] ?? 6000).toLocaleString()}</span>
            </div>
            <input
              type="range"
              min="1000"
              max="15000"
              step="500"
              bind:value={config.aadt_estimates['primary']}
              oninput={() => {
                config.aadt_estimates['primary_link'] = config.aadt_estimates['primary'];
                onConfigChange();
              }}
            />
          </div>

          <div class="slider-group">
            <div class="s-head">
              <span>Secondary Roads AADT</span>
              <span class="s-val font-mono">{(config.aadt_estimates['secondary'] ?? 5000).toLocaleString()}</span>
            </div>
            <input
              type="range"
              min="1000"
              max="10000"
              step="500"
              bind:value={config.aadt_estimates['secondary']}
              oninput={() => {
                config.aadt_estimates['secondary_link'] = config.aadt_estimates['secondary'];
                onConfigChange();
              }}
            />
          </div>

          <div class="slider-group">
            <div class="s-head">
              <span>Tertiary Roads AADT</span>
              <span class="s-val font-mono">{(config.aadt_estimates['tertiary'] ?? 3000).toLocaleString()}</span>
            </div>
            <input
              type="range"
              min="500"
              max="6000"
              step="250"
              bind:value={config.aadt_estimates['tertiary']}
              oninput={() => {
                config.aadt_estimates['tertiary_link'] = config.aadt_estimates['tertiary'];
                onConfigChange();
              }}
            />
          </div>

          <div class="slider-group">
            <div class="s-head">
              <span>Residential / Minor AADT</span>
              <span class="s-val font-mono">{(config.aadt_estimates['residential'] ?? 500).toLocaleString()}</span>
            </div>
            <input
              type="range"
              min="100"
              max="2500"
              step="100"
              bind:value={config.aadt_estimates['residential']}
              oninput={() => {
                config.aadt_estimates['unclassified'] = config.aadt_estimates['residential'];
                onConfigChange();
              }}
            />
          </div>

          <div class="sub-divider"></div>

          <div class="slider-group">
            <div class="s-head">
              <span>Untagged Arterial Speed</span>
              <span class="s-val font-mono">{config.speed_defaults['primary'] ?? 40} mph</span>
            </div>
            <input
              type="range"
              min="20"
              max="60"
              step="5"
              bind:value={config.speed_defaults['primary']}
              oninput={() => {
                config.speed_defaults['secondary'] = config.speed_defaults['primary'];
                config.speed_defaults['tertiary'] = config.speed_defaults['primary'];
                onConfigChange();
              }}
            />
          </div>

          <div class="slider-group">
            <div class="s-head">
              <span>Untagged Residential Speed</span>
              <span class="s-val font-mono">{config.speed_defaults['residential'] ?? 30} mph</span>
            </div>
            <input
              type="range"
              min="15"
              max="40"
              step="5"
              bind:value={config.speed_defaults['residential']}
              oninput={() => {
                config.speed_defaults['unclassified'] = config.speed_defaults['residential'];
                onConfigChange();
              }}
            />
          </div>

          <div class="sub-divider"></div>

          <div class="slider-group">
            <div class="s-head">
              <span>Off-Road / Sidepath Buffer</span>
              <span class="s-val font-mono">{config.min_offroad_distance} m</span>
            </div>
            <input
              type="range"
              min="5"
              max="30"
              step="1"
              bind:value={config.min_offroad_distance}
              oninput={onConfigChange}
            />
          </div>

          <div class="slider-group">
            <div class="s-head">
              <span>Wide Track Threshold</span>
              <span class="s-val font-mono">{config.min_wide_width.toFixed(1)} m</span>
            </div>
            <input
              type="range"
              min="1.5"
              max="3.5"
              step="0.1"
              bind:value={config.min_wide_width}
              oninput={onConfigChange}
            />
          </div>

          <button type="button" class="outline-btn full-btn" onclick={onResetConfig}>
            Reset Default Parameters
          </button>
        </div>
      {/if}
    </div>

    <!-- Accordion 2: Case Study Presets & Upload -->
    <div class="accordion">
      <button
        type="button"
        class="acc-header"
        onclick={() => (dataOpen = !dataOpen)}
        aria-expanded={dataOpen}
      >
        <span class="acc-title">Case Studies & Custom OSM</span>
        <svg class="chev" class:rotated={dataOpen} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </button>

      {#if dataOpen}
        <div class="acc-body">
          <div class="preset-stack">
            {#each PRESET_LOCATIONS as p (p.id)}
              <button
                type="button"
                class="preset-card"
                class:selected={selectedPresetId === p.id}
                onclick={() => onSelectPreset(p)}
              >
                <div class="p-top">
                  <span class="p-name">{p.name}</span>
                  <span class="p-country">{p.country}</span>
                </div>
                <p class="p-desc">{p.description}</p>
              </button>
            {/each}
          </div>

          <input
            type="file"
            accept=".json,.geojson"
            bind:this={fileInput}
            onchange={handleFileSelected}
            style="display: none;"
          />
          <button type="button" class="outline-btn full-btn" style="margin-top: 8px;" onclick={() => fileInput?.click()}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="17 8 12 3 7 8" />
              <line x1="12" y1="3" x2="12" y2="15" />
            </svg>
            Upload Overpass JSON
          </button>
        </div>
      {/if}
    </div>

    <!-- Accordion 3: Data Export & Methodology -->
    <div class="accordion">
      <button
        type="button"
        class="acc-header"
        onclick={() => (exportOpen = !exportOpen)}
        aria-expanded={exportOpen}
      >
        <span class="acc-title">Export & Methodology</span>
        <svg class="chev" class:rotated={exportOpen} width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9"></polyline>
        </svg>
      </button>

      {#if exportOpen}
        <div class="acc-body">
          <div class="export-grid">
            <button type="button" class="civic-btn exp-btn" onclick={onExportGeoJSON}>
              Links GeoJSON
            </button>
            <button type="button" class="outline-btn exp-btn" onclick={onExportJunctionsGeoJSON}>
              Junctions GeoJSON
            </button>
            <button type="button" class="outline-btn exp-btn full-span" onclick={onExportCSV}>
              Summary Metrics CSV
            </button>
          </div>
          <p class="method-note">
            Powered by <strong>multilevelos-core</strong> in Rust/WebAssembly. Scores street links &amp; polygonized junctions in &lt;15ms across Walking, Wheeling &amp; Cycling using 5m R-tree sidepath road association, DfT LTN 1/20, ATE Rural Design Guide (2026) &amp; Transport Scotland CbD tables.
          </p>
        </div>
      {/if}
    </div>
  </div>
</aside>

<style>
  .sidebar-drawer {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    width: 356px;
    max-width: 100vw;
    background: var(--bg-surface);
    border-right: 1px solid var(--border-color);
    box-shadow: var(--shadow-lg);
    z-index: 30;
    display: flex;
    flex-direction: column;
    transform: translateX(0);
    transition: transform 0.24s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .sidebar-drawer.closed {
    transform: translateX(-100%);
    pointer-events: none;
  }

  .mobile-scrim {
    display: none;
  }

  .mobile-drawer-head {
    display: none;
    align-items: center;
    justify-content: space-between;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }

  .drawer-title {
    font-size: 12px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .mobile-map-btn {
    height: 28px;
    padding: 0 10px;
    font-size: 11px;
  }

  .loading-strip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    background: var(--accent-civic-soft);
    color: var(--accent-civic);
    border-bottom: 1px solid var(--accent-civic-border);
    font-size: 11px;
    font-weight: 600;
  }

  .strip-spinner {
    width: 12px;
    height: 12px;
    border: 2px solid var(--accent-civic-border);
    border-top-color: var(--accent-civic);
    border-radius: 50%;
    animation: spin 0.75s linear infinite;
    flex-shrink: 0;
  }

  .drawer-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .mode-switcher-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    padding: 9px 10px;
    display: flex;
    flex-direction: column;
    gap: 7px;
  }

  .ms-row-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .ms-label {
    font-size: 10.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-muted);
  }

  .std-segmented {
    display: inline-flex;
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    padding: 2px;
    gap: 2px;
  }

  .std-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 10px;
    font-weight: 700;
    padding: 2px 7px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.12s ease;
  }

  .std-btn.active {
    background: var(--accent-civic);
    color: #ffffff;
  }

  .mode-pills {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
  }

  .mode-pill {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    font-size: 11px;
    font-weight: 600;
    padding: 5px 4px;
    border-radius: 6px;
    cursor: pointer;
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: all 0.14s ease;
  }

  .mode-pill:hover {
    border-color: var(--border-strong);
    color: var(--text-primary);
  }

  .mode-pill.active {
    background: var(--accent-civic);
    border-color: var(--accent-civic);
    color: #ffffff;
    font-weight: 700;
  }

  .accordion {
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md);
    background: var(--bg-card);
    overflow: hidden;
  }

  .acc-header {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    background: var(--bg-secondary);
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
  }

  .acc-header:hover {
    background: var(--bg-muted);
  }

  .acc-title {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    font-weight: 700;
  }

  .count-pill {
    font-size: 10px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 999px;
    background: var(--accent-civic);
    color: #ffffff;
  }

  .chev {
    color: var(--text-muted);
    transition: transform 0.18s ease;
  }

  .chev.rotated {
    transform: rotate(180deg);
  }

  .acc-body {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    border-top: 1px solid var(--border-color);
  }

  .toggle-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    cursor: pointer;
  }

  .toggle-row input {
    accent-color: var(--accent-civic);
    width: 15px;
    height: 15px;
    cursor: pointer;
  }

  .hint {
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.35;
    margin-top: -4px;
  }

  .sub-divider {
    height: 1px;
    background: var(--border-color);
    margin: 2px 0;
  }

  .slider-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .s-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 11px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .s-val {
    font-weight: 700;
    color: var(--accent-civic);
  }

  input[type='range'] {
    width: 100%;
    height: 5px;
    border-radius: 3px;
    accent-color: var(--accent-civic);
    cursor: pointer;
  }

  .full-btn {
    width: 100%;
    height: 32px;
  }

  .preset-stack {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .preset-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 8px 10px;
    text-align: left;
    cursor: pointer;
    transition: all 0.14s ease;
  }

  .preset-card:hover {
    border-color: var(--border-strong);
    background: var(--bg-secondary);
  }

  .preset-card.selected {
    border-color: var(--accent-civic);
    background: var(--accent-civic-soft);
  }

  .p-top {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 6px;
    margin-bottom: 2px;
  }

  .p-name {
    font-size: 12px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .p-country {
    font-size: 10px;
    color: var(--text-muted);
  }

  .p-desc {
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.3;
  }

  .export-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .full-span {
    grid-column: 1 / -1;
  }

  .exp-btn {
    height: 32px;
    font-size: 11.5px;
  }

  .method-note {
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.45;
  }

  @media (max-width: 768px) {
    .sidebar-drawer {
      width: min(340px, 88vw);
    }
    .mobile-scrim {
      display: block;
      position: absolute;
      inset: 0;
      background: rgba(15, 23, 42, 0.35);
      z-index: 28;
    }
    .mobile-drawer-head {
      display: flex;
    }
  }
</style>
