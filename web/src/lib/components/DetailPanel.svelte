<script lang="ts">
  import MapButtons from './MapButtons.svelte';
  import { formatShortLosLabel, getLosTone } from '../palette';
  import type { SelectedSegment } from '../../types';

  let {
    segment = null,
    onClose,
    onZoomTo
  }: {
    segment: SelectedSegment | null;
    onClose: () => void;
    onZoomTo: (seg: SelectedSegment) => void;
  } = $props();

  let badgeTone = $derived(segment ? getLosTone(segment.los_label, segment.los_score) : 'civic');
  let shortLos = $derived(segment ? formatShortLosLabel(segment.los_label, segment.los_score) : '');
</script>

{#if segment}
  <div class="detail-card panel-card">
    <div class="detail-header">
      <div class="title-wrap">
        <div class="map-badge-group">
          <span class="map-badge {badgeTone}">{shortLos}</span>
          <span class="map-badge civic">{segment.highway}</span>
        </div>
        <h3>{segment.name || 'Unnamed Street'}</h3>
      </div>
      <button type="button" class="close-btn" onclick={onClose} aria-label="Close segment details">×</button>
    </div>

    <div class="detail-body">
      <div class="attr-grid">
        <div class="attr-item">
          <span class="attr-label">OSM Way ID</span>
          <span class="attr-value font-mono">{segment.osm_id}</span>
        </div>
        <div class="attr-item">
          <span class="attr-label">Length</span>
          <span class="attr-value font-mono">{segment.length_m} m</span>
        </div>
        <div class="attr-item full">
          <span class="attr-label">Infrastructure Classification</span>
          <span class="attr-value">{segment.infrastructure}</span>
        </div>
        <div class="attr-item">
          <span class="attr-label">Speed Limit</span>
          <span class="attr-value font-mono">
            {segment.speed_limit_mph != null ? `${segment.speed_limit_mph} mph` : 'Traffic-free'}
          </span>
        </div>
        <div class="attr-item">
          <span class="attr-label">Est. AADT</span>
          <span class="attr-value font-mono">
            {segment.estimated_aadt != null ? segment.estimated_aadt.toLocaleString() : 'None'}
          </span>
        </div>
        {#if segment.distance_to_road_m != null}
          <div class="attr-item full">
            <span class="attr-label">Distance to Adjacent Carriageway</span>
            <span class="attr-value font-mono">{segment.distance_to_road_m} m</span>
          </div>
        {/if}
      </div>

      <button type="button" class="civic-btn zoom-btn" onclick={() => onZoomTo(segment)}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        Zoom to Segment
      </button>

      <MapButtons center={segment.center} osmId={segment.osm_id} />
    </div>
  </div>
{/if}

<style>
  .detail-card {
    position: absolute;
    bottom: 24px;
    right: 24px;
    width: 320px;
    max-width: calc(100vw - 32px);
    z-index: 25;
    overflow: hidden;
    animation: slideUp 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes slideUp {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 8px;
    padding: 12px 14px 10px;
    border-bottom: 1px solid var(--border-color);
  }

  .title-wrap h3 {
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
    margin-top: 2px;
    line-height: 1.25;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 18px;
    line-height: 1;
    padding: 2px 6px;
    cursor: pointer;
    border-radius: 4px;
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-secondary);
  }

  .detail-body {
    padding: 12px 14px;
  }

  .attr-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px 12px;
    margin-bottom: 12px;
  }

  .attr-item {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .attr-item.full {
    grid-column: 1 / -1;
  }

  .attr-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--text-muted);
  }

  .attr-value {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .zoom-btn {
    width: 100%;
    height: 32px;
  }

  @media (max-width: 768px) {
    .detail-card {
      left: 10px;
      right: 10px;
      bottom: 10px;
      width: auto;
      max-height: 50dvh;
      overflow-y: auto;
    }
  }
</style>
