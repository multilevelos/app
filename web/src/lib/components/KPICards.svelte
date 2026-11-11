<script lang="ts">
  import type { LegendMetricRow, SummaryStats } from '../../types';

  let {
    summary,
    rows = []
  }: {
    summary: SummaryStats | null;
    rows: LegendMetricRow[];
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
      <span class="seg-count font-mono">{summary.total_segments.toLocaleString()} links</span>
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
        <span class="k-label">Suitable (High/Med)</span>
        <span class="k-val font-mono">{summary.high_med_pct}%</span>
      </div>
      <div class="mini-kpi rose">
        <span class="k-label">Unsuitable</span>
        <span class="k-val font-mono">{summary.unsuitable_pct}%</span>
      </div>
      <div class="mini-kpi neutral">
        <span class="k-label">Mean Speed</span>
        <span class="k-val font-mono">{summary.average_speed_mph} <small>mph</small></span>
      </div>
    </div>
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

  .seg-count {
    font-size: 11px;
    color: var(--text-muted);
    font-weight: 500;
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
    grid-template-columns: 1fr 1fr 0.9fr;
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

  .k-val small {
    font-size: 10px;
    font-weight: 500;
    color: var(--text-secondary);
  }
</style>
