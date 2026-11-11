<script lang="ts">
  import MapButtons from './MapButtons.svelte';
  import { formatShortLosLabel, gradeColor, gradeTone } from '../palette';
  import type { SelectedFeature } from '../../types';

  let {
    feature = null,
    onClose,
    onZoomTo
  }: {
    feature: SelectedFeature | null;
    onClose: () => void;
    onZoomTo: (feat: SelectedFeature) => void;
  } = $props();

  let badgeTone = $derived(feature ? gradeTone(feature.active_grade) : 'civic');
  let shortLos = $derived(
    feature
      ? feature.kind === 'segment'
        ? formatShortLosLabel(feature.los_label, feature.active_grade)
        : formatShortLosLabel(undefined, feature.active_grade)
      : ''
  );
</script>

{#if feature}
  <div class="detail-card panel-card">
    <div class="detail-header">
      <div class="title-wrap">
        <div class="map-badge-group">
          <span class="map-badge {badgeTone}">{shortLos}</span>
          {#if feature.kind === 'segment'}
            <span class="map-badge civic">{feature.highway}</span>
          {:else}
            <span class="map-badge civic">Junction · {feature.arm_count} arms</span>
          {/if}
        </div>
        <h3>{feature.name || (feature.kind === 'junction' ? `Junction #${feature.junction_id}` : 'Unnamed Street')}</h3>
      </div>
      <button type="button" class="close-btn" onclick={onClose} aria-label="Close feature details">×</button>
    </div>

    <div class="detail-body">
      <!-- Cross-Modal A–E Scorecard -->
      <div class="scorecard" aria-label="Cross-modal A to E grade scorecard">
        <div class="sc-cell">
          <span class="sc-mode">MMLoS</span>
          <span class="sc-grade" style="background-color: {gradeColor(feature.grade_multimodal)};">
            {feature.grade_multimodal}
          </span>
        </div>
        <div class="sc-cell">
          <span class="sc-mode">Walk</span>
          <span class="sc-grade" style="background-color: {gradeColor(feature.grade_walking)};">
            {feature.grade_walking}
          </span>
        </div>
        <div class="sc-cell">
          <span class="sc-mode">Wheel</span>
          <span class="sc-grade" style="background-color: {gradeColor(feature.grade_wheeling)};">
            {feature.grade_wheeling}
          </span>
        </div>
        <div class="sc-cell">
          <span class="sc-mode">Cycle</span>
          <span class="sc-grade" style="background-color: {gradeColor(feature.grade_cycling)};">
            {feature.grade_cycling}
          </span>
        </div>
      </div>

      {#if feature.kind === 'segment'}
        <div class="attr-grid">
          <div class="attr-item">
            <span class="attr-label">OSM Way ID</span>
            <span class="attr-value font-mono">{feature.osm_id}</span>
          </div>
          <div class="attr-item">
            <span class="attr-label">Length</span>
            <span class="attr-value font-mono">{feature.length_m} m</span>
          </div>
          <div class="attr-item full">
            <span class="attr-label">Infrastructure & Facility</span>
            <span class="attr-value">{feature.infrastructure}</span>
          </div>
          <div class="attr-item">
            <span class="attr-label">Surface / Lit</span>
            <span class="attr-value">{feature.surface}{feature.is_lit ? ' · Lit' : ''}</span>
          </div>
          <div class="attr-item">
            <span class="attr-label">Separation</span>
            <span class="attr-value">{feature.separation === 'none' ? 'None' : feature.separation}</span>
          </div>
          <div class="attr-item">
            <span class="attr-label">Effective Speed</span>
            <span class="attr-value font-mono">
              {feature.speed_limit_mph != null ? `${feature.speed_limit_mph} mph` : 'Traffic-free'}
            </span>
          </div>
          <div class="attr-item">
            <span class="attr-label">Est. AADT</span>
            <span class="attr-value font-mono">
              {feature.estimated_aadt != null ? feature.estimated_aadt.toLocaleString() : 'None'}
            </span>
          </div>
          {#if feature.width_m != null || feature.incline_pct != null}
            <div class="attr-item">
              <span class="attr-label">Tagged Width</span>
              <span class="attr-value font-mono">
                {feature.width_m != null ? `${feature.width_m} m` : 'Untagged'}
              </span>
            </div>
            <div class="attr-item">
              <span class="attr-label">Gradient</span>
              <span class="attr-value font-mono">
                {feature.incline_pct != null ? `${feature.incline_pct}%` : 'Flat / untagged'}
              </span>
            </div>
          {/if}
          {#if feature.distance_to_road_m != null}
            <div class="attr-item full">
              <span class="attr-label">Median Distance to Carriageway</span>
              <span class="attr-value font-mono">{feature.distance_to_road_m} m</span>
            </div>
          {/if}
        </div>
      {:else}
        <div class="attr-grid">
          <div class="attr-item full">
            <span class="attr-label">Junction Control</span>
            <span class="attr-value">{feature.control_type}</span>
          </div>
          <div class="attr-item">
            <span class="attr-label">Max Approach Speed</span>
            <span class="attr-value font-mono">{feature.max_speed_mph} mph</span>
          </div>
          <div class="attr-item">
            <span class="attr-label">Max Approach Flow</span>
            <span class="attr-value font-mono">{feature.max_aadt.toLocaleString()} AADT</span>
          </div>
          <div class="attr-item">
            <span class="attr-label">Tactile / Kerbs</span>
            <span class="attr-value">{feature.has_tactile_paving ? 'Tagged tactile' : 'Unverified'}</span>
          </div>
          <div class="attr-item">
            <span class="attr-label">Footprint Radius</span>
            <span class="attr-value font-mono">{feature.radius_m} m</span>
          </div>
        </div>
      {/if}

      <button type="button" class="civic-btn zoom-btn" onclick={() => onZoomTo(feature)}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        Zoom to {feature.kind === 'junction' ? 'Junction' : 'Segment'}
      </button>

      <MapButtons
        center={feature.center}
        osmId={feature.kind === 'segment' ? feature.osm_id : null}
      />
    </div>
  </div>
{/if}

<style>
  .detail-card {
    position: absolute;
    bottom: 24px;
    right: 24px;
    width: 332px;
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

  .scorecard {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
    padding: 7px;
    margin-bottom: 11px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
  }

  .sc-cell {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 3px;
  }

  .sc-mode {
    font-size: 9.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--text-muted);
  }

  .sc-grade {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 22px;
    border-radius: 5px;
    color: #ffffff;
    font-size: 12px;
    font-weight: 800;
    font-family: var(--font-mono);
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
      max-height: 55dvh;
      overflow-y: auto;
    }
  }
</style>
