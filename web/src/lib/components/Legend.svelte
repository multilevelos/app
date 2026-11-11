<script lang="ts">
  import type { ColorMode, LegendMetricRow } from '../../types';

  let {
    colorMode = $bindable('los'),
    rows = [],
    hiddenValues = [],
    onToggleItem,
    onIsolateItem,
    onShowAll
  }: {
    colorMode: ColorMode;
    rows: LegendMetricRow[];
    hiddenValues: string[];
    onToggleItem: (value: string) => void;
    onIsolateItem: (value: string) => void;
    onShowAll: () => void;
  } = $props();

  function isVisible(val: string): boolean {
    return !hiddenValues.includes(val);
  }
</script>

<div class="legend-section">
  <div class="colorby-row">
    <label for="colorby-select">Colour by</label>
    <select id="colorby-select" bind:value={colorMode}>
      <option value="los">Level of Service (LoS)</option>
      <option value="infrastructure">Infrastructure type</option>
      <option value="speed">Speed limit</option>
      <option value="traffic">Traffic volume (AADT)</option>
    </select>
    {#if hiddenValues.length > 0}
      <button type="button" class="show-all-btn" onclick={onShowAll}>Show all</button>
    {/if}
  </div>

  <div class="legend-rows">
    {#each rows as row (row.value)}
      {@const visible = isVisible(row.value)}
      <div class="legend-row" class:dimmed={!visible}>
        <button
          type="button"
          class="row-btn"
          onclick={() => onToggleItem(row.value)}
          aria-pressed={visible}
          title={visible ? `Hide ${row.label} on map` : `Show ${row.label} on map`}
        >
          <span class="check-box" class:checked={visible}>
            {#if visible}
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            {/if}
          </span>
          <span class="swatch-bar" style="background-color: {row.color};"></span>
          <span class="label-col">
            <span class="row-label">{row.label}</span>
            {#if row.desc}
              <span class="row-desc">{row.desc}</span>
            {/if}
          </span>
          <span class="metrics-col font-mono">
            <span class="km-val">{row.km.toFixed(1)} km</span>
            <span class="pct-val">{row.pct.toFixed(0)}%</span>
          </span>
        </button>
        <button
          type="button"
          class="solo-btn"
          onclick={() => onIsolateItem(row.value)}
          title="Isolate only {row.label}"
        >
          only
        </button>
      </div>
    {/each}
  </div>
</div>

<style>
  .legend-section {
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border-color);
  }

  .colorby-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .colorby-row label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .colorby-row select {
    flex: 1;
    min-width: 0;
    height: 30px;
    background: var(--bg-surface);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    padding: 0 8px;
    outline: none;
    cursor: pointer;
  }

  .colorby-row select:focus {
    border-color: var(--accent-civic);
  }

  .show-all-btn {
    background: var(--accent-civic-soft);
    color: var(--accent-civic);
    border: 1px solid var(--accent-civic-border);
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 600;
    padding: 4px 8px;
    cursor: pointer;
    white-space: nowrap;
  }

  .show-all-btn:hover {
    background: var(--accent-civic);
    color: #ffffff;
  }

  .legend-rows {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .legend-row {
    position: relative;
    display: flex;
    align-items: center;
    border-radius: var(--radius-sm);
    transition: background-color 0.12s ease;
  }

  .legend-row:hover {
    background: var(--bg-secondary);
  }

  .legend-row.dimmed {
    opacity: 0.48;
  }

  .row-btn {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 6px;
    background: transparent;
    border: none;
    text-align: left;
    cursor: pointer;
    color: inherit;
  }

  .check-box {
    width: 14px;
    height: 14px;
    border-radius: 4px;
    border: 1.5px solid var(--border-strong);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: #ffffff;
    background: var(--bg-surface);
    transition: all 0.12s ease;
  }

  .check-box.checked {
    background: var(--accent-civic);
    border-color: var(--accent-civic);
  }

  :global([data-theme='dark']) .check-box.checked {
    color: #090d16;
  }

  .swatch-bar {
    width: 12px;
    height: 12px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .label-col {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.2;
  }

  .row-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .row-desc {
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .metrics-col {
    display: flex;
    align-items: baseline;
    gap: 6px;
    flex-shrink: 0;
    transition: opacity 0.12s ease;
  }

  .legend-row:hover .metrics-col {
    opacity: 0.15;
  }

  .km-val {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .pct-val {
    font-size: 10px;
    color: var(--text-muted);
    min-width: 26px;
    text-align: right;
  }

  .solo-btn {
    position: absolute;
    right: 6px;
    opacity: 0;
    pointer-events: none;
    background: var(--bg-surface);
    color: var(--accent-civic);
    border: 1px solid var(--accent-civic-border);
    border-radius: 4px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    padding: 2px 6px;
    cursor: pointer;
    transition: opacity 0.12s ease;
  }

  .legend-row:hover .solo-btn {
    opacity: 1;
    pointer-events: auto;
  }

  .solo-btn:hover {
    background: var(--accent-civic);
    color: #ffffff;
  }
</style>
