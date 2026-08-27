<script lang="ts">
  import type { ColorMode, LosConfig, PresetLocation, SearchResult, SummaryStats } from '../../types';
  import { searchPlace } from '../geocode';

  let {
    config = $bindable(),
    summary = null,
    colorMode = $bindable('los'),
    isLoading = false,
    loadingMessage = '',
    onSelectPreset = (preset: PresetLocation) => {},
    onSearchSelect = (result: SearchResult) => {},
    onConfigChange = () => {},
    onResetConfig = () => {},
    onExportGeoJSON = () => {},
    onExportCSV = () => {},
    onFileUpload = (file: File) => {}
  }: {
    config: LosConfig;
    summary: SummaryStats | null;
    colorMode: ColorMode;
    isLoading: boolean;
    loadingMessage: string;
    onSelectPreset: (preset: PresetLocation) => void;
    onSearchSelect: (result: SearchResult) => void;
    onConfigChange: () => void;
    onResetConfig: () => void;
    onExportGeoJSON: () => void;
    onExportCSV: () => void;
    onFileUpload: (file: File) => void;
  } = $props();

  let activeTab = $state<'data' | 'params' | 'analytics' | 'export'>('data');
  let searchQuery = $state('');
  let searchResults = $state<SearchResult[]>([]);
  let isSearching = $state(false);
  let fileInput = $state<HTMLInputElement>();

  const PRESETS: PresetLocation[] = [
    {
      id: 'edinburgh',
      name: 'Edinburgh (City Centre)',
      country: 'Scotland, UK',
      description: 'Extensive 20mph network, off-road railway paths, and protected lanes.',
      center: [-3.195, 55.952],
      zoom: 14.5,
      bbox: [-3.22, 55.94, -3.17, 55.965]
    },
    {
      id: 'leeds',
      name: 'Leeds (University & Core)',
      country: 'England, UK',
      description: 'Major urban corridors, high traffic arterials, and student routes.',
      center: [-1.550, 53.800],
      zoom: 14.5,
      bbox: [-1.575, 53.788, -1.525, 53.812]
    },
    {
      id: 'cambridge',
      name: 'Cambridge (Historic Core)',
      country: 'England, UK',
      description: 'High cycling mode share, shared paths, and pedestrianized historic center.',
      center: [0.122, 52.205],
      zoom: 14.5,
      bbox: [0.10, 52.19, 0.15, 52.22]
    }
  ];

  let selectedPresetId = $state<string>('edinburgh');

  async function handleSearchInput() {
    if (searchQuery.trim().length < 2) {
      searchResults = [];
      return;
    }
    isSearching = true;
    try {
      searchResults = await searchPlace(searchQuery);
    } finally {
      isSearching = false;
    }
  }

  function handleResultClick(result: SearchResult) {
    searchResults = [];
    searchQuery = result.display_name.split(',')[0];
    onSearchSelect(result);
  }

  function handleFileSelected(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      onFileUpload(target.files[0]);
    }
  }
</script>

<aside class="sidebar">
  <!-- Brand Header -->
  <div class="brand-header">
    <div class="logo-row">
      <div class="logo-badge">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#22c55e" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="18.5" cy="17.5" r="3.5"/>
          <circle cx="5.5" cy="17.5" r="3.5"/>
          <circle cx="15" cy="5" r="1"/>
          <path d="M12 17.5V14l-3-3 4-3 2 3h2"/>
        </svg>
      </div>
      <div>
        <h1 class="brand-title">MultiLevelOS</h1>
        <div class="brand-subtitle">Cycling Level of Service (LoS) & AADT</div>
      </div>
    </div>
  </div>

  <!-- Navigation Tabs -->
  <div class="nav-tabs">
    <button
      class="tab-btn {activeTab === 'data' ? 'active' : ''}"
      onclick={() => activeTab = 'data'}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
      Data
    </button>
    <button
      class="tab-btn {activeTab === 'params' ? 'active' : ''}"
      onclick={() => activeTab = 'params'}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" y1="21" x2="4" y2="14"/><line x1="4" y1="10" x2="4" y2="3"/><line x1="12" y1="21" x2="12" y2="12"/><line x1="12" y1="8" x2="12" y2="3"/><line x1="20" y1="21" x2="20" y2="16"/><line x1="20" y1="12" x2="20" y2="3"/><line x1="1" y1="14" x2="7" y2="14"/><line x1="9" y1="8" x2="15" y2="8"/><line x1="17" y1="16" x2="23" y2="16"/></svg>
      Parameters
    </button>
    <button
      class="tab-btn {activeTab === 'analytics' ? 'active' : ''}"
      onclick={() => activeTab = 'analytics'}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="20" x2="18" y2="10"/><line x1="12" y1="20" x2="12" y2="4"/><line x1="6" y1="20" x2="6" y2="14"/></svg>
      Analytics
    </button>
    <button
      class="tab-btn {activeTab === 'export' ? 'active' : ''}"
      onclick={() => activeTab = 'export'}
    >
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/><polyline points="16 6 12 2 8 6"/><line x1="12" y1="2" x2="12" y2="15"/></svg>
      Export
    </button>
  </div>

  <!-- Loading State Banner -->
  {#if isLoading}
    <div class="loading-banner">
      <div class="spinner"></div>
      <div class="loading-text">{loadingMessage || 'Processing network in WebAssembly...'}</div>
    </div>
  {/if}

  <!-- Tab Content Area -->
  <div class="tab-content">
    {#if activeTab === 'data'}
      <!-- Location Presets -->
      <div class="section-card">
        <h3 class="section-heading">Preset Case Studies</h3>
        <div class="preset-list">
          {#each PRESETS as p}
            <button
              class="preset-item {selectedPresetId === p.id ? 'selected' : ''}"
              onclick={() => {
                selectedPresetId = p.id;
                onSelectPreset(p);
              }}
            >
              <div class="preset-name-row">
                <span class="preset-name">{p.name}</span>
                <span class="preset-country">{p.country}</span>
              </div>
              <p class="preset-desc">{p.description}</p>
            </button>
          {/each}
        </div>
      </div>

      <!-- Search Place Online -->
      <div class="section-card">
        <h3 class="section-heading">Search OSM by Location</h3>
        <div class="search-box">
          <input
            type="text"
            placeholder="e.g. Manchester, Bristol, Dublin, Glasgow..."
            bind:value={searchQuery}
            oninput={handleSearchInput}
            class="search-input"
          />
          {#if isSearching}
            <div class="search-spinner"></div>
          {/if}
        </div>

        {#if searchResults.length > 0}
          <div class="search-dropdown">
            {#each searchResults as res}
              <button class="search-result-row" onclick={() => handleResultClick(res)}>
                <span class="search-title">{res.display_name.split(',').slice(0, 2).join(',')}</span>
                <span class="search-subtitle">{res.display_name}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Upload Local OSM / Overpass JSON -->
      <div class="section-card">
        <h3 class="section-heading">Load Custom Overpass File</h3>
        <p class="section-subtext">Upload raw Overpass JSON (.json) directly into the Rust engine.</p>
        <input
          type="file"
          accept=".json,.geojson"
          bind:this={fileInput}
          onchange={handleFileSelected}
          style="display: none;"
        />
        <button class="btn-secondary" onclick={() => fileInput.click()}>
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="17 8 12 3 7 8"/><line x1="12" y1="3" x2="12" y2="15"/></svg>
          Select Local JSON File
        </button>
      </div>

    {:else if activeTab === 'params'}
      <!-- Display / Color Mode -->
      <div class="section-card">
        <h3 class="section-heading">Map Visualization Layer</h3>
        <div class="pill-group">
          <button
            class="pill-btn {colorMode === 'los' ? 'active' : ''}"
            onclick={() => colorMode = 'los'}
          >
            Level of Service
          </button>
          <button
            class="pill-btn {colorMode === 'infrastructure' ? 'active' : ''}"
            onclick={() => colorMode = 'infrastructure'}
          >
            Infrastructure
          </button>
          <button
            class="pill-btn {colorMode === 'speed' ? 'active' : ''}"
            onclick={() => colorMode = 'speed'}
          >
            Speed Limits
          </button>
          <button
            class="pill-btn {colorMode === 'traffic' ? 'active' : ''}"
            onclick={() => colorMode = 'traffic'}
          >
            Traffic (AADT)
          </button>
        </div>
      </div>

      <!-- General Filters -->
      <div class="section-card">
        <h3 class="section-heading">Scope & Filter</h3>
        <label class="toggle-row">
          <span>Include Mixed Traffic Carriageways</span>
          <input
            type="checkbox"
            bind:checked={config.include_mixed_traffic}
            onchange={onConfigChange}
          />
        </label>
        <p class="field-hint">When disabled, only dedicated cycle tracks, painted lanes, and shared footways are analyzed.</p>
      </div>

      <!-- Road Class AADT Estimates -->
      <div class="section-card">
        <div class="flex-between">
          <h3 class="section-heading">Road Class AADT Estimates</h3>
          <span class="badge-tag">Traffic</span>
        </div>
        <p class="section-subtext">Estimated motor vehicle Annual Average Daily Traffic based on OSM road class hierarchy.</p>

        <!-- Global Multiplier -->
        <div class="slider-field">
          <div class="field-header">
            <span>Global Traffic Scaling</span>
            <span class="field-val">{config.traffic_multiplier.toFixed(1)}x</span>
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

        <div class="slider-field">
          <div class="field-header">
            <span>Primary Roads AADT</span>
            <span class="field-val">{config.aadt_estimates['primary']?.toLocaleString() || 6000}</span>
          </div>
          <input
            type="range"
            min="1000"
            max="15000"
            step="500"
            bind:value={config.aadt_estimates['primary']}
            oninput={onConfigChange}
          />
        </div>

        <div class="slider-field">
          <div class="field-header">
            <span>Secondary Roads AADT</span>
            <span class="field-val">{config.aadt_estimates['secondary']?.toLocaleString() || 5000}</span>
          </div>
          <input
            type="range"
            min="1000"
            max="10000"
            step="500"
            bind:value={config.aadt_estimates['secondary']}
            oninput={onConfigChange}
          />
        </div>

        <div class="slider-field">
          <div class="field-header">
            <span>Tertiary Roads AADT</span>
            <span class="field-val">{config.aadt_estimates['tertiary']?.toLocaleString() || 3000}</span>
          </div>
          <input
            type="range"
            min="500"
            max="6000"
            step="250"
            bind:value={config.aadt_estimates['tertiary']}
            oninput={onConfigChange}
          />
        </div>

        <div class="slider-field">
          <div class="field-header">
            <span>Residential / Minor Streets AADT</span>
            <span class="field-val">{config.aadt_estimates['residential']?.toLocaleString() || 500}</span>
          </div>
          <input
            type="range"
            min="100"
            max="2500"
            step="100"
            bind:value={config.aadt_estimates['residential']}
            oninput={onConfigChange}
          />
        </div>
      </div>

      <!-- Road Class Default Speeds -->
      <div class="section-card">
        <div class="flex-between">
          <h3 class="section-heading">Default Speeds (Untagged)</h3>
          <span class="badge-tag">Speed</span>
        </div>

        <div class="slider-field">
          <div class="field-header">
            <span>Primary / Secondary Default</span>
            <span class="field-val">{config.speed_defaults['primary'] || 40} mph</span>
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

        <div class="slider-field">
          <div class="field-header">
            <span>Residential Street Default</span>
            <span class="field-val">{config.speed_defaults['residential'] || 30} mph</span>
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
      </div>

      <!-- Infrastructure Distance Thresholds -->
      <div class="section-card">
        <h3 class="section-heading">Infrastructure Thresholds</h3>

        <div class="slider-field">
          <div class="field-header">
            <span>Min Distance for Off-Road Path</span>
            <span class="field-val">{config.min_offroad_distance} m</span>
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

        <div class="slider-field">
          <div class="field-header">
            <span>Min Width for Wide Track</span>
            <span class="field-val">{config.min_wide_width.toFixed(1)} m</span>
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

        <button class="btn-secondary" style="margin-top: 10px;" onclick={onResetConfig}>
          Reset to Cycling by Design Defaults
        </button>
      </div>

    {:else if activeTab === 'analytics'}
      {#if summary}
        <!-- KPI Cards Grid -->
        <div class="kpi-grid">
          <div class="kpi-card">
            <div class="kpi-label">Analyzed Length</div>
            <div class="kpi-val">{summary.total_length_km.toFixed(1)} <span class="kpi-unit">km</span></div>
            <div class="kpi-sub">{summary.total_segments.toLocaleString()} segments</div>
          </div>
          <div class="kpi-card" style="border-left: 3px solid #16a34a;">
            <div class="kpi-label">Suitable (High/Med)</div>
            <div class="kpi-val" style="color: #22c55e;">{summary.high_med_pct}%</div>
            <div class="kpi-sub">All-abilities / protected</div>
          </div>
          <div class="kpi-card" style="border-left: 3px solid #ef4444;">
            <div class="kpi-label">Unsuitable / Non-Compliant</div>
            <div class="kpi-val" style="color: #ef4444;">{summary.unsuitable_pct}%</div>
            <div class="kpi-sub">High stress carriageway</div>
          </div>
          <div class="kpi-card">
            <div class="kpi-label">Average Speed Limit</div>
            <div class="kpi-val">{summary.average_speed_mph} <span class="kpi-unit">mph</span></div>
            <div class="kpi-sub">Length-weighted mean</div>
          </div>
        </div>

        <!-- Level of Service Breakdown -->
        <div class="section-card">
          <h3 class="section-heading">Level of Service Breakdown</h3>
          <div class="distribution-bar">
            {#each Object.entries(summary.los_breakdown) as [key, val]}
              {#if val.percentage > 0}
                <div
                  class="dist-segment"
                  style="width: {val.percentage}%; background-color: {val.color};"
                  title="{key}: {val.percentage}% ({val.length_km} km)"
                ></div>
              {/if}
            {/each}
          </div>

          <div class="metric-list">
            {#each Object.entries(summary.los_breakdown) as [label, metric]}
              <div class="metric-row">
                <div class="metric-label-group">
                  <span class="metric-dot" style="background-color: {metric.color};"></span>
                  <span class="metric-name">{label}</span>
                </div>
                <div class="metric-numbers">
                  <span class="metric-len">{metric.length_km} km</span>
                  <span class="metric-pct">{metric.percentage}%</span>
                </div>
              </div>
            {/each}
          </div>
        </div>

        <!-- Infrastructure Breakdown -->
        <div class="section-card">
          <h3 class="section-heading">Infrastructure Classification</h3>
          <div class="distribution-bar">
            {#each Object.entries(summary.infra_breakdown) as [key, val]}
              {#if val.percentage > 0}
                <div
                  class="dist-segment"
                  style="width: {val.percentage}%; background-color: {val.color};"
                  title="{key}: {val.percentage}% ({val.length_km} km)"
                ></div>
              {/if}
            {/each}
          </div>

          <div class="metric-list">
            {#each Object.entries(summary.infra_breakdown) as [label, metric]}
              <div class="metric-row">
                <div class="metric-label-group">
                  <span class="metric-dot" style="background-color: {metric.color};"></span>
                  <span class="metric-name">{label}</span>
                </div>
                <div class="metric-numbers">
                  <span class="metric-len">{metric.length_km} km</span>
                  <span class="metric-pct">{metric.percentage}%</span>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="empty-state">
          <p>No network loaded yet. Select a preset or fetch OSM data to view analytics.</p>
        </div>
      {/if}

    {:else if activeTab === 'export'}
      <div class="section-card">
        <h3 class="section-heading">Export Analysis Results</h3>
        <p class="section-subtext">Export the complete Level of Service network with all tags, AADT estimates, and scores for GIS modeling (QGIS, R sf, Python GeoPandas).</p>

        <div class="export-actions">
          <button class="btn-primary" onclick={onExportGeoJSON}>
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
            Download GeoJSON Layer
          </button>
          <button class="btn-secondary" onclick={onExportCSV}>
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></svg>
            Download Summary CSV
          </button>
        </div>
      </div>

      <div class="section-card">
        <h3 class="section-heading">About Scottish Cycling by Design (CbD)</h3>
        <p class="section-subtext">
          This implementation follows the <strong>Cycling by Design (Transport Scotland)</strong> and CD 195 standard.
          It classifies Level of Service based on motor traffic speed (85th percentile / limit), two-way traffic volume (AADT), and physical segregation type.
        </p>
      </div>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 380px;
    height: 100%;
    background-color: var(--bg-sidebar);
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    z-index: 20;
    flex-shrink: 0;
  }

  .brand-header {
    padding: 16px 18px 12px;
    border-bottom: 1px solid var(--border-color);
    background: #090d16;
  }

  .logo-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .logo-badge {
    width: 36px;
    height: 36px;
    background: rgba(22, 163, 74, 0.15);
    border: 1px solid rgba(34, 197, 94, 0.35);
    border-radius: 9px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .brand-title {
    font-size: 16px;
    font-weight: 700;
    color: #f8fafc;
    letter-spacing: -0.01em;
  }

  .brand-subtitle {
    font-size: 11px;
    color: var(--text-secondary);
  }

  .nav-tabs {
    display: flex;
    background: #090d16;
    border-bottom: 1px solid var(--border-color);
    padding: 6px 12px;
    gap: 4px;
  }

  .tab-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 500;
    padding: 7px 4px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab-btn:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .tab-btn.active {
    background: var(--bg-card);
    color: #38bdf8;
    font-weight: 600;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
  }

  .loading-banner {
    display: flex;
    align-items: center;
    gap: 10px;
    background: #0369a1;
    color: #f0f9ff;
    padding: 8px 16px;
    font-size: 12px;
    font-weight: 500;
  }

  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: #ffffff;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .tab-content {
    flex: 1;
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .section-card {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 10px;
    padding: 14px;
  }

  .section-heading {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 6px;
  }

  .section-subtext {
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.4;
    margin-bottom: 10px;
  }

  .preset-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .preset-item {
    background: #0f172a;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 10px 12px;
    text-align: left;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .preset-item:hover {
    border-color: #38bdf8;
    background: #1e293b;
  }

  .preset-item.selected {
    border-color: #0284c7;
    background: rgba(2, 132, 199, 0.15);
  }

  .preset-name-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 3px;
  }

  .preset-name {
    font-size: 13px;
    font-weight: 600;
    color: #f8fafc;
  }

  .preset-country {
    font-size: 11px;
    color: #94a3b8;
  }

  .preset-desc {
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.3;
  }

  .search-box {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-input {
    width: 100%;
    background: #0f172a;
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 13px;
    outline: none;
  }

  .search-input:focus {
    border-color: #38bdf8;
  }

  .search-spinner {
    position: absolute;
    right: 10px;
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.2);
    border-top-color: #38bdf8;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .search-dropdown {
    margin-top: 6px;
    background: #0f172a;
    border: 1px solid var(--border-color);
    border-radius: 6px;
    max-height: 180px;
    overflow-y: auto;
  }

  .search-result-row {
    width: 100%;
    background: transparent;
    border: none;
    border-bottom: 1px solid rgba(51, 65, 85, 0.4);
    padding: 8px 10px;
    text-align: left;
    cursor: pointer;
    display: flex;
    flex-direction: column;
  }

  .search-result-row:hover {
    background: #1e293b;
  }

  .search-title {
    font-size: 12px;
    font-weight: 600;
    color: #f8fafc;
  }

  .search-subtitle {
    font-size: 11px;
    color: #94a3b8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .pill-group {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }

  .pill-btn {
    background: #0f172a;
    border: 1px solid var(--border-color);
    color: var(--text-secondary);
    padding: 7px 10px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .pill-btn:hover {
    color: var(--text-primary);
  }

  .pill-btn.active {
    background: #0284c7;
    border-color: #38bdf8;
    color: #ffffff;
    font-weight: 600;
  }

  .toggle-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    cursor: pointer;
  }

  .field-hint {
    font-size: 11px;
    color: var(--text-muted);
    margin-top: 4px;
  }

  .flex-between {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .badge-tag {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    background: rgba(56, 189, 248, 0.15);
    color: #38bdf8;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .slider-field {
    margin-top: 10px;
  }

  .field-header {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-secondary);
    margin-bottom: 4px;
  }

  .field-val {
    color: #38bdf8;
    font-family: var(--font-mono);
    font-weight: 600;
  }

  input[type="range"] {
    width: 100%;
    accent-color: #0284c7;
  }

  .kpi-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .kpi-card {
    background: var(--bg-card);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 10px 12px;
  }

  .kpi-label {
    font-size: 11px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .kpi-val {
    font-size: 18px;
    font-weight: 700;
    color: #f8fafc;
    margin: 2px 0;
  }

  .kpi-unit {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-muted);
  }

  .kpi-sub {
    font-size: 10px;
    color: var(--text-muted);
  }

  .distribution-bar {
    display: flex;
    height: 12px;
    border-radius: 6px;
    overflow: hidden;
    margin: 10px 0 12px;
    background: #0f172a;
    border: 1px solid var(--border-color);
  }

  .dist-segment {
    height: 100%;
    transition: width 0.3s ease;
  }

  .metric-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .metric-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
  }

  .metric-label-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .metric-dot {
    width: 10px;
    height: 10px;
    border-radius: 2px;
  }

  .metric-name {
    color: var(--text-primary);
    font-weight: 500;
  }

  .metric-numbers {
    display: flex;
    gap: 8px;
    font-family: var(--font-mono);
  }

  .metric-len {
    color: var(--text-secondary);
  }

  .metric-pct {
    font-weight: 600;
    color: #f8fafc;
    min-width: 38px;
    text-align: right;
  }

  .btn-primary {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    background: #0284c7;
    color: #ffffff;
    border: none;
    padding: 10px 14px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
    box-shadow: 0 4px 6px -1px rgba(2, 132, 199, 0.25);
    transition: all 0.15s ease;
  }

  .btn-primary:hover {
    background: #0369a1;
  }

  .btn-secondary {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    background: #0f172a;
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    padding: 9px 14px;
    border-radius: 8px;
    font-weight: 500;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-secondary:hover {
    background: #1e293b;
    border-color: #475569;
  }

  .export-actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .empty-state {
    padding: 24px 12px;
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }
</style>
