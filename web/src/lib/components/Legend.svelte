<script lang="ts">
  import type { ColorMode } from '../../types';

  let { colorMode = 'los' }: { colorMode: ColorMode } = $props();

  const losItems = [
    { label: 'High (LoS 3)', color: '#16a34a', desc: 'Suitable for all ages & abilities' },
    { label: 'Medium (LoS 2)', color: '#eab308', desc: 'Acceptable for most adults' },
    { label: 'Low (LoS 1)', color: '#f97316', desc: 'High stress / low quality' },
    { label: 'Unsuitable (Mixed)', color: '#dc2626', desc: 'High traffic mixing' },
    { label: 'Non-Compliant', color: '#9333ea', desc: 'Substandard intervention' }
  ];

  const infraItems = [
    { label: 'Segregated Track (wide)', color: '#054d05' },
    { label: 'Off Road Path', color: '#3a9120' },
    { label: 'Segregated Track (narrow)', color: '#87d668' },
    { label: 'Shared Footway', color: '#ffbf00' },
    { label: 'Painted Cycle Lane', color: '#f97316' },
    { label: 'Mixed Traffic Street', color: '#64748b' }
  ];

  const speedItems = [
    { label: '<20 mph', color: '#22c55e' },
    { label: '20 mph', color: '#84cc16' },
    { label: '30 mph', color: '#eab308' },
    { label: '40 mph', color: '#f97316' },
    { label: '50 mph', color: '#ef4444' },
    { label: '60+ mph', color: '#7f1d1d' }
  ];

  const trafficItems = [
    { label: '0 to 999 AADT (Quiet)', color: '#22c55e' },
    { label: '1,000 to 1,999 AADT', color: '#eab308' },
    { label: '2,000 to 3,999 AADT', color: '#f97316' },
    { label: '4,000+ AADT (Busy)', color: '#ef4444' }
  ];
</script>

<div class="legend-box">
  <div class="legend-header">
    <span class="legend-title">
      {#if colorMode === 'los'}
        Level of Service (LoS)
      {:else if colorMode === 'infrastructure'}
        Cycle Infrastructure
      {:else if colorMode === 'speed'}
        Speed Limit
      {:else}
        Traffic Volume (AADT)
      {/if}
    </span>
  </div>

  <div class="legend-items">
    {#if colorMode === 'los'}
      {#each losItems as item}
        <div class="legend-row">
          <span class="legend-color-dot" style="background-color: {item.color};"></span>
          <div class="legend-label-group">
            <span class="legend-label">{item.label}</span>
            <span class="legend-subtext">{item.desc}</span>
          </div>
        </div>
      {/each}
    {:else if colorMode === 'infrastructure'}
      {#each infraItems as item}
        <div class="legend-row">
          <span class="legend-color-dot" style="background-color: {item.color};"></span>
          <span class="legend-label">{item.label}</span>
        </div>
      {/each}
    {:else if colorMode === 'speed'}
      {#each speedItems as item}
        <div class="legend-row">
          <span class="legend-color-dot" style="background-color: {item.color};"></span>
          <span class="legend-label">{item.label}</span>
        </div>
      {/each}
    {:else}
      {#each trafficItems as item}
        <div class="legend-row">
          <span class="legend-color-dot" style="background-color: {item.color};"></span>
          <span class="legend-label">{item.label}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .legend-box {
    background: rgba(15, 23, 42, 0.85);
    backdrop-filter: blur(12px);
    border: 1px solid var(--border-color);
    border-radius: 10px;
    padding: 12px 14px;
    color: var(--text-primary);
    box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.4);
    pointer-events: auto;
    min-width: 210px;
    max-width: 260px;
    font-size: 13px;
  }

  .legend-header {
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 6px;
    margin-bottom: 8px;
  }

  .legend-title {
    font-weight: 600;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
  }

  .legend-items {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .legend-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .legend-color-dot {
    width: 12px;
    height: 12px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .legend-label-group {
    display: flex;
    flex-direction: column;
    line-height: 1.2;
  }

  .legend-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .legend-subtext {
    font-size: 10px;
    color: var(--text-muted);
  }
</style>
