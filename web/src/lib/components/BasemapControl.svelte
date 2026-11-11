<script lang="ts">
  import type { BasemapId } from '../../types';

  let {
    basemap,
    showPlaceNames = true,
    onChangeBasemap,
    onTogglePlaceNames
  }: {
    basemap: BasemapId;
    showPlaceNames: boolean;
    onChangeBasemap: (style: BasemapId) => void;
    onTogglePlaceNames: () => void;
  } = $props();

  let menuOpen = $state(false);

  const basemapOptions: { id: BasemapId; label: string; description: string }[] = [
    { id: 'light', label: 'Light Canvas', description: 'Clean civic greyscale cartography' },
    { id: 'dark', label: 'Dark Canvas', description: 'Muted dark street backdrop' },
    { id: 'voyager', label: 'Streets', description: 'Detailed urban street map' },
    { id: 'cyclosm', label: 'CyclOSM', description: 'Active travel routes, surfaces & contours' },
    { id: 'satellite', label: 'Satellite', description: 'High-resolution aerial imagery' }
  ];
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === 'Escape') menuOpen = false;
  }}
/>

<div class="basemap-control-wrapper">
  {#if menuOpen}
    <div class="backdrop" onclick={() => (menuOpen = false)} aria-hidden="true"></div>

    <div class="basemap-menu panel-card" role="menu">
      <div class="menu-title">Select Basemap</div>
      {#each basemapOptions as opt}
        <button
          type="button"
          class="menu-item"
          class:active={basemap === opt.id}
          onclick={() => {
            onChangeBasemap(opt.id);
            menuOpen = false;
          }}
          role="menuitemradio"
          aria-checked={basemap === opt.id}
        >
          <span class="radio-indicator"></span>
          <div class="opt-text">
            <span class="opt-label">{opt.label}</span>
            <span class="opt-desc">{opt.description}</span>
          </div>
        </button>
      {/each}

      <div class="menu-divider"></div>
      <label class="placenames-toggle" title="Show or hide street and town names above analyzed corridors">
        <input type="checkbox" checked={showPlaceNames} onchange={onTogglePlaceNames} />
        <span class="switch-track"><span class="switch-thumb"></span></span>
        <span class="opt-label">Place names</span>
      </label>
    </div>
  {/if}

  <div class="btn-group">
    <button
      type="button"
      class="outline-btn basemap-btn"
      class:active={menuOpen}
      onclick={() => (menuOpen = !menuOpen)}
      title="Change Basemap Style"
      aria-label="Change Basemap Style"
      aria-expanded={menuOpen}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <polygon points="12 2 2 7 12 12 22 7 12 2"></polygon>
        <polyline points="2 17 12 22 22 17"></polyline>
        <polyline points="2 12 12 17 22 12"></polyline>
      </svg>
      <span class="btn-text">Basemap</span>
    </button>

    <button
      type="button"
      class="outline-btn basemap-btn placenames-btn"
      class:active={showPlaceNames}
      onclick={onTogglePlaceNames}
      title={showPlaceNames ? 'Hide place names above corridors' : 'Show place names above corridors'}
      aria-label="Toggle place names"
      aria-pressed={showPlaceNames}
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <polyline points="4 7 4 4 20 4 20 7"></polyline>
        <line x1="9" y1="20" x2="15" y2="20"></line>
        <line x1="12" y1="4" x2="12" y2="20"></line>
      </svg>
      <span class="btn-text">Labels</span>
    </button>
  </div>
</div>

<style>
  .basemap-control-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 40;
  }

  .btn-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .basemap-btn {
    position: relative;
    z-index: 45;
    height: 34px;
    padding: 0 10px;
    font-size: 12px;
  }

  .basemap-menu {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    width: 245px;
    padding: 6px;
    z-index: 55;
  }

  .menu-title {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    padding: 4px 8px 6px;
  }

  .menu-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 8px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    text-align: left;
    cursor: pointer;
  }

  .menu-item:hover {
    background: var(--bg-secondary);
  }

  .radio-indicator {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid var(--border-strong);
    position: relative;
    flex-shrink: 0;
  }

  .menu-item.active .radio-indicator {
    border-color: var(--accent-civic);
  }

  .menu-item.active .radio-indicator::after {
    content: '';
    position: absolute;
    inset: 2px;
    border-radius: 50%;
    background: var(--accent-civic);
  }

  .opt-text {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .opt-label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .opt-desc {
    font-size: 11px;
    color: var(--text-muted);
  }

  .menu-divider {
    height: 1px;
    background: var(--border-color);
    margin: 5px 4px;
  }

  .placenames-toggle {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 7px 8px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    user-select: none;
  }

  .placenames-toggle:hover {
    background: var(--bg-secondary);
  }

  .placenames-toggle input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .switch-track {
    width: 30px;
    height: 17px;
    background: var(--border-strong);
    border-radius: 999px;
    position: relative;
    transition: background 0.15s ease;
    flex-shrink: 0;
  }

  .placenames-toggle input:checked + .switch-track {
    background: var(--accent-civic);
  }

  .switch-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background: #ffffff;
    transition: transform 0.15s ease;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
  }

  .placenames-toggle input:checked + .switch-track .switch-thumb {
    transform: translateX(13px);
  }

  @media (max-width: 960px) {
    .placenames-btn {
      display: none;
    }
    .btn-text {
      display: none;
    }
    .basemap-btn {
      padding: 0 8px;
    }
  }
</style>
