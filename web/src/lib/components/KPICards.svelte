<script lang="ts">
  import type { LegendMetricRow, SummaryStats, TravelMode } from '../../types';

  let {
    summary,
    rows = [],
    activeMode = 'multimodal',
    onSelectMode = (_m: TravelMode) => {}
  }: {
    summary: SummaryStats | null;
    rows: LegendMetricRow[];
    activeMode?: TravelMode;
    onSelectMode?: (mode: TravelMode) => void;
  } = $props();

  let compositionAria = $derived(
    rows
      .filter((r) => r.km > 0)
      .map((r) => `${r.label}: ${r.km.toFixed(1)} km (${r.pct.toFixed(1)}%)`)
      .join(', ')
  );
</script>

{#if summary}
  <div class="kpi-section">
    <div class="headline-row">
      <div class="total-wrap">
        <span class="total-km font-mono">{summary.total_length_km.toFixed(1)}</span>
        <span class="total-unit">km</span>
      </div>
      <div class="counts-wrap font-mono">
        <span>{summary.total_segments.toLocaleString()} links</span>
        {#if summary.junctions?.total_junctions > 0}
          <span class="dot">·</span>
          <span>{summary.junctions.total_junctions.toLocaleString()} junctions</span>
        {/if}
      </div>
    </div>

    <div class="comp-bar" role="img" aria-label="Network composition: {compositionAria}">
      {#each rows as row (row.value)}
        {#if row.pct > 0}
          <div
            class="comp-seg"
            style="width: {row.pct}%; background-color: {row.color};"
            title="{row.label}: {row.km.toFixed(2)} km ({row.pct}%)"
          ></div>
        {/if}
      {/each}
    </div>

    <div class="mini-kpi-grid">
      <div class="mini-kpi emerald">
        <span class="k-label">Suitable (A–B)</span>
        <span class="k-val font-mono">{summary.high_med_pct}%</span>
      </div>
      <div class="mini-kpi rose">
        <span class="k-label">Hostile (E)</span>
        <span class="k-val font-mono">{summary.unsuitable_pct}%</span>
      </div>
      <div class="mini-kpi civic">
        <span class="k-label">Junctions A–B</span>
        <span class="k-val font-mono">
          {summary.junctions?.total_junctions > 0 ? `${summary.junctions.suitable_pct}%` : '—'}
        </span>
      </div>
    </div>

    {#if summary.mode_suitability}
      <div class="cross-modal-strip" aria-label="Cross-modal Grades A–B suitability">
        <button
          type="button"
          class="cm-pill"
          class:active={activeMode === 'multimodal'}
          onclick={() => onSelectMode('multimodal')}
          title="Switch to Multi-Modal composite MMLoS"
        >
          <span class="cm-name">MMLoS</span>
          <span class="cm-pct font-mono">{summary.mode_suitability.multimodal_ab_pct}%</span>
        </button>
        <button
          type="button"
          class="cm-pill"
          class:active={activeMode === 'walking'}
          onclick={() => onSelectMode('walking')}
          title="Switch to Walking Level of Service"
        >
          <span class="cm-name">Walk</span>
          <span class="cm-pct font-mono">{summary.mode_suitability.walking_ab_pct}%</span>
        </button>
        <button
          type="button"
          class="cm-pill"
          class:active={activeMode === 'wheeling'}
          onclick={() => onSelectMode('wheeling')}
          title="Switch to Wheeling / Wheelchair Level of Service"
        >
          <span class="cm-name">Wheel</span>
          <span class="cm-pct font-mono">{summary.mode_suitability.wheeling_ab_pct}%</span>
        </button>
        <button
          type="button"
          class="cm-pill"
          class:active={activeMode === 'cycling'}
          onclick={() => onSelectMode('cycling')}
          title="Switch to Cycling Level of Service"
        >
          <span class="cm-name">Cycle</span>
          <span class="cm-pct font-mono">{summary.mode_suitability.cycling_ab_pct}%</span>
        </button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .kpi-section {
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border-color);
  }

  .headline-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 8px;
  }

  .total-wrap {
    display: flex;
    align-items: baseline;
    gap: 4px;
  }

  .total-km {
    font-size: 23px;
    font-weight: 700;
    color: var(--text-primary);
    line-height: 1.05;
  }

  .total-unit {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .counts-wrap {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 500;
  }

  .dot {
    opacity: 0.6;
  }

  .comp-bar {
    height: 10px;
    background: var(--bg-muted);
    border-radius: 999px;
    overflow: hidden;
    display: flex;
    gap: 1px;
    margin-bottom: 10px;
  }

  .comp-seg {
    height: 100%;
    transition: width 0.25s ease;
  }

  .mini-kpi-grid {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 6px;
  }

  .mini-kpi {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .mini-kpi.emerald {
    border-left: 3px solid var(--accent-emerald);
  }

  .mini-kpi.rose {
    border-left: 3px solid var(--accent-rose);
  }

  .mini-kpi.civic {
    border-left: 3px solid var(--accent-civic);
  }

  .k-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .k-val {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-primary);
  }

  .mini-kpi.emerald .k-val {
    color: var(--accent-emerald);
  }

  .mini-kpi.rose .k-val {
    color: var(--accent-rose);
  }

  .mini-kpi.civic .k-val {
    color: var(--accent-civic);
  }

  .cross-modal-strip {
    margin-top: 8px;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
  }

  .cm-pill {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1px;
    padding: 5px 4px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color);
    background: var(--bg-surface);
    cursor: pointer;
    transition: all 0.14s ease;
  }

  .cm-pill:hover {
    border-color: var(--border-strong);
    background: var(--bg-secondary);
  }

  .cm-pill.active {
    background: var(--accent-civic-soft);
    border-color: var(--accent-civic);
  }

  .cm-name {
    font-size: 9.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--text-muted);
  }

  .cm-pill.active .cm-name {
    color: var(--accent-civic);
  }

  .cm-pct {
    font-size: 11.5px;
    font-weight: 700;
    color: var(--text-primary);
  }
</style>
