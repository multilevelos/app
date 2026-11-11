<script lang="ts">
  import BasemapControl from './BasemapControl.svelte';
  import PlaceSearch from './PlaceSearch.svelte';
  import { formatModeLabel, formatStandardLabel } from '../palette';
  import type { BasemapId, DesignStandard, SearchResult, ThemeMode, TravelMode } from '../../types';

  let {
    sidebarOpen,
    onToggleSidebar,
    areaLabel = 'Edinburgh',
    activeMode = 'multimodal',
    standard = 'ltn120',
    isLoading = false,
    basemap = 'light',
    showPlaceNames = true,
    theme = 'light',
    onChangeBasemap,
    onTogglePlaceNames,
    onToggleTheme,
    onSelectPlace,
    onFetchViewport,
    onEnterPrintView,
    onCopyShareLink
  }: {
    sidebarOpen: boolean;
    onToggleSidebar: () => void;
    areaLabel?: string;
    activeMode?: TravelMode;
    standard?: DesignStandard;
    isLoading?: boolean;
    basemap: BasemapId;
    showPlaceNames: boolean;
    theme: ThemeMode;
    onChangeBasemap: (style: BasemapId) => void;
    onTogglePlaceNames: () => void;
    onToggleTheme: () => void;
    onSelectPlace: (result: SearchResult) => void;
    onFetchViewport: () => void;
    onEnterPrintView: () => void;
    onCopyShareLink: () => Promise<void>;
  } = $props();

  let copyStatus: 'idle' | 'copied' | 'error' = $state('idle');
  let mobileSearchOpen = $state(false);

  async function handleCopy() {
    try {
      await onCopyShareLink();
      copyStatus = 'copied';
      setTimeout(() => {
        if (copyStatus === 'copied') copyStatus = 'idle';
      }, 2000);
    } catch {
      copyStatus = 'error';
      setTimeout(() => {
        copyStatus = 'idle';
      }, 2500);
    }
  }
</script>

<header class="app-header">
  <div class="left-section">
    <button
      type="button"
      class="outline-btn toggle-btn"
      onclick={onToggleSidebar}
      title={sidebarOpen ? 'Close side panel' : 'Open side panel'}
      aria-label={sidebarOpen ? 'Close side panel' : 'Open side panel'}
      aria-expanded={sidebarOpen}
    >
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <line x1="3" y1="6" x2="21" y2="6"></line>
        <line x1="3" y1="12" x2="21" y2="12"></line>
        <line x1="3" y1="18" x2="21" y2="18"></line>
      </svg>
      <span class="toggle-label">{sidebarOpen ? 'Hide' : 'Panel'}</span>
    </button>

    <div class="brand">
      <div class="logo-icon" title="MultiLevelOS — Multi-Modal Level of Service">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polygon points="12 3 21 7.5 12 12 3 7.5" fill="#22c55e" fill-opacity="0.22" stroke="#16a34a" stroke-width="2" />
          <polyline points="3 12 12 16.5 21 12" stroke="#0284c7" stroke-width="2" />
          <polyline points="3 16.5 12 21 21 16.5" stroke="#d97706" stroke-width="2" />
        </svg>
      </div>
      <div class="title-group">
        <div class="title-row">
          <h1>MultiLevelOS</h1>
          {#if areaLabel}
            <span class="area-badge" title="Active study network">{areaLabel}</span>
          {/if}
        </div>
        <span class="subtitle">
          {formatModeLabel(activeMode)} · {formatStandardLabel(standard)}
        </span>
      </div>
    </div>
  </div>

  <div class="center-section">
    <div class="desktop-search">
      <PlaceSearch onSelect={onSelectPlace} />
    </div>

    <button
      type="button"
      class="outline-btn nav-btn mobile-search-btn"
      class:active={mobileSearchOpen}
      onclick={() => (mobileSearchOpen = !mobileSearchOpen)}
      title="Search places"
      aria-label="Search places"
      aria-expanded={mobileSearchOpen}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="7"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
    </button>

    <button
      type="button"
      class="civic-btn fetch-btn"
      onclick={onFetchViewport}
      disabled={isLoading}
      title="Download OpenStreetMap network for the current visible map area"
    >
      {#if isLoading}
        <span class="btn-spinner" aria-hidden="true"></span>
        <span class="fetch-label">Loading...</span>
      {:else}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
          <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16" />
          <path d="M16 16h5v5" />
        </svg>
        <span class="fetch-label">Fetch View</span>
      {/if}
    </button>

    <BasemapControl
      {basemap}
      {showPlaceNames}
      {onChangeBasemap}
      {onTogglePlaceNames}
    />
  </div>

  <div class="right-section">
    <button
      type="button"
      class="outline-btn nav-btn"
      onclick={onToggleTheme}
      title={theme === 'light' ? 'Switch to Dark UI theme' : 'Switch to Civic Light UI theme'}
      aria-label={theme === 'light' ? 'Switch to Dark theme' : 'Switch to Light theme'}
    >
      {#if theme === 'light'}
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
        </svg>
      {:else}
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="5"></circle>
          <line x1="12" y1="1" x2="12" y2="3"></line>
          <line x1="12" y1="21" x2="12" y2="23"></line>
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
          <line x1="1" y1="12" x2="3" y2="12"></line>
          <line x1="21" y1="12" x2="23" y2="12"></line>
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
        </svg>
      {/if}
    </button>

    <button
      type="button"
      class="outline-btn nav-btn print-btn"
      onclick={onEnterPrintView}
      title="Publication Print & PNG Export View (Shortcut: P)"
      aria-label="Print or export PNG"
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <polyline points="6 9 6 2 18 2 18 9"></polyline>
        <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"></path>
        <rect x="6" y="14" width="12" height="8"></rect>
      </svg>
      <span class="action-text">Print</span>
    </button>

    <button
      type="button"
      class="outline-btn nav-btn share-btn"
      class:copied={copyStatus === 'copied'}
      onclick={handleCopy}
      title="Copy shareable link to current map view & parameters"
    >
      {#if copyStatus === 'copied'}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
        <span class="action-text">Copied</span>
      {:else}
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="18" cy="5" r="3"></circle>
          <circle cx="6" cy="12" r="3"></circle>
          <circle cx="18" cy="19" r="3"></circle>
          <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"></line>
          <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"></line>
        </svg>
        <span class="action-text">Share</span>
      {/if}
    </button>
  </div>
</header>

{#if mobileSearchOpen}
  <div class="mobile-search-bar">
    <PlaceSearch
      onSelect={(res) => {
        mobileSearchOpen = false;
        onSelectPlace(res);
      }}
    />
  </div>
{/if}

<style>
  .app-header {
    height: 56px;
    min-height: 56px;
    width: 100%;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 14px;
    gap: 12px;
    z-index: 40;
    box-shadow: var(--shadow-sm);
  }

  .left-section {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }

  .toggle-btn {
    height: 34px;
    padding: 0 10px;
    flex-shrink: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .logo-icon {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .title-group {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 7px;
  }

  h1 {
    font-size: 15px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.015em;
    line-height: 1.15;
  }

  .area-badge {
    font-size: 10px;
    font-weight: 600;
    padding: 1px 7px;
    border-radius: 999px;
    background: var(--accent-civic-soft);
    color: var(--accent-civic);
    border: 1px solid var(--accent-civic-border);
    max-width: 150px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .subtitle {
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.15;
    white-space: nowrap;
  }

  .center-section {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    justify-content: center;
    max-width: 660px;
  }

  .desktop-search {
    flex: 1;
    max-width: 300px;
  }

  .mobile-search-btn {
    display: none;
  }

  .fetch-btn {
    height: 34px;
    padding: 0 12px;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .btn-spinner {
    width: 13px;
    height: 13px;
    border: 2px solid rgba(255, 255, 255, 0.35);
    border-top-color: currentColor;
    border-radius: 50%;
    animation: spin 0.75s linear infinite;
  }

  .right-section {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .nav-btn {
    height: 34px;
    padding: 0 10px;
  }

  .share-btn.copied {
    background: var(--accent-emerald-soft);
    color: var(--accent-emerald);
    border-color: var(--accent-emerald-border);
  }

  .mobile-search-bar {
    display: none;
    padding: 8px 12px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-color);
    z-index: 38;
  }

  @media (max-width: 1080px) {
    .subtitle,
    .toggle-label,
    .action-text {
      display: none;
    }
    .nav-btn,
    .toggle-btn {
      padding: 0 8px;
    }
  }

  @media (max-width: 768px) {
    .app-header {
      padding: 0 10px;
      gap: 6px;
    }
    .desktop-search,
    .area-badge,
    .print-btn {
      display: none;
    }
    .mobile-search-btn {
      display: inline-flex;
    }
    .mobile-search-bar {
      display: block;
    }
    .mobile-search-bar :global(.place-search) {
      max-width: 100%;
    }
    .fetch-label {
      display: none;
    }
    .fetch-btn {
      padding: 0 9px;
    }
  }
</style>
