<script lang="ts">
  import { formatModeLabel, formatStandardLabel, SCHEMES } from '../palette';
  import type { ColorMode, LegendMetricRow, LosConfig, SummaryStats } from '../../types';

  let {
    mode = 'legend',
    areaLabel = 'Study Area',
    colorMode = 'los',
    rows = [],
    hiddenValues = [],
    summary = null,
    config,
    onChangeMode,
    onDownloadPng,
    onPrint,
    onExit
  }: {
    mode: 'legend' | 'clean';
    areaLabel: string;
    colorMode: ColorMode;
    rows: LegendMetricRow[];
    hiddenValues: string[];
    summary: SummaryStats | null;
    config: LosConfig;
    onChangeMode: (m: 'legend' | 'clean') => void;
    onDownloadPng: () => void;
    onPrint: () => void;
    onExit: () => void;
  } = $props();

  let visibleRows = $derived(rows.filter((r) => !hiddenValues.includes(r.value)));
  let schemeTitle = $derived(SCHEMES[colorMode].title);
</script>

<div class="print-controls-bar panel-card no-print">
  <button
    type="button"
    class="pill-opt"
    class:active={mode === 'legend'}
    onclick={() => onChangeMode('legend')}
  >
    Legend
  </button>
  <button
    type="button"
    class="pill-opt"
    class:active={mode === 'clean'}
    onclick={() => onChangeMode('clean')}
  >
    Clean
  </button>
  <span class="sep"></span>
  <button type="button" class="pill-opt" onclick={onDownloadPng}>PNG</button>
  <button type="button" class="pill-opt" onclick={onPrint}>Print</button>
  <button type="button" class="pill-opt exit" onclick={onExit}>Exit (Esc)</button>
</div>

{#if mode === 'legend'}
  <div class="print-legend-card panel-card">
    <div class="pl-head">
      <div class="pl-brand">MultiLevelOS · {areaLabel}</div>
      <div class="pl-title">{schemeTitle}</div>
      {#if summary}
        <div class="pl-sub font-mono">
          {summary.total_length_km.toFixed(1)} km · {summary.high_med_pct}% A–B
          {#if summary.junctions?.total_junctions > 0}
            · {summary.junctions.total_junctions} jcts
          {/if}
        </div>
      {/if}
    </div>

    <div class="pl-rows">
      {#each visibleRows as r (r.value)}
        <div class="pl-row">
          <span class="pl-swatch" style="background-color: {r.color};"></span>
          <span class="pl-label">{r.label}</span>
          <span class="pl-km font-mono">{r.km.toFixed(1)} km</span>
        </div>
      {/each}
    </div>

    <div class="pl-chips">
      <span class="pl-chip">{formatModeLabel(config.active_mode)}</span>
      <span class="pl-chip">{formatStandardLabel(config.standard)}</span>
      {#if Math.abs(config.traffic_multiplier - 1.0) > 0.01}
        <span class="pl-chip">Traffic {config.traffic_multiplier.toFixed(1)}x</span>
      {/if}
      {#if !config.include_mixed_traffic}
        <span class="pl-chip">Dedicated Infra Only</span>
      {/if}
      <span class="pl-chip">© OpenStreetMap</span>
    </div>
  </div>
{/if}

<style>
  .print-controls-bar {
    position: absolute;
    top: 16px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 50;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 5px 8px;
    border-radius: 999px;
  }

  .pill-opt {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 12px;
    font-weight: 600;
    padding: 5px 10px;
    border-radius: 999px;
    cursor: pointer;
  }

  .pill-opt:hover {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .pill-opt.active {
    background: var(--accent-civic);
    color: #ffffff;
  }

  .pill-opt.exit {
    color: var(--accent-rose);
  }

  .sep {
    width: 1px;
    height: 16px;
    background: var(--border-color);
    margin: 0 2px;
  }

  .print-legend-card {
    position: absolute;
    bottom: 24px;
    left: 24px;
    width: 275px;
    padding: 12px 14px;
    z-index: 35;
  }

  .pl-head {
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 8px;
    margin-bottom: 8px;
  }

  .pl-brand {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--accent-civic);
  }

  .pl-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-primary);
    margin-top: 2px;
  }

  .pl-sub {
    font-size: 11px;
    color: var(--text-secondary);
    margin-top: 2px;
  }

  .pl-rows {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .pl-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11px;
  }

  .pl-swatch {
    width: 11px;
    height: 11px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .pl-label {
    flex: 1;
    color: var(--text-primary);
    font-weight: 500;
  }

  .pl-km {
    color: var(--text-secondary);
    font-weight: 600;
  }

  .pl-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 10px;
    padding-top: 8px;
    border-top: 1px solid var(--border-color);
  }

  .pl-chip {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-muted);
  }

  @media print {
    .no-print {
      display: none !important;
    }
  }
</style>
