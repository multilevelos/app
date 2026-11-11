<script lang="ts">
  import { searchPlace } from '../geocode';
  import type { SearchResult } from '../../types';

  interface PlaceOption {
    raw: SearchResult;
    name: string;
    context: string;
  }

  let {
    onSelect
  }: {
    onSelect: (result: SearchResult) => void;
  } = $props();

  let query = $state('');
  let options: PlaceOption[] = $state([]);
  let activeIndex = $state(-1);
  let loading = $state(false);
  let statusMsg = $state('');
  let open = $state(false);

  let timer: ReturnType<typeof setTimeout> | undefined;
  let controller: AbortController | undefined;

  function toOption(r: SearchResult): PlaceOption {
    const parts = String(r.display_name || '')
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean);
    const name = parts[0] || 'Place';
    const context = parts
      .slice(1)
      .filter((p) => !/^[A-Z]{1,2}\d/.test(p))
      .slice(0, 3)
      .join(' · ');
    return { raw: r, name, context };
  }

  function handleInput() {
    statusMsg = '';
    activeIndex = -1;
    if (timer) clearTimeout(timer);
    if (query.trim().length < 2) {
      controller?.abort();
      options = [];
      open = false;
      loading = false;
      return;
    }
    timer = setTimeout(runSearch, 350);
  }

  async function runSearch() {
    controller?.abort();
    controller = new AbortController();
    loading = true;
    try {
      const data = await searchPlace(query, controller.signal);
      const seen = new Set<string>();
      options = data
        .filter((d) => Array.isArray(d.boundingbox) && d.boundingbox.length === 4)
        .map(toOption)
        .filter((opt) => {
          const key = `${opt.name}|${opt.context}`;
          if (seen.has(key)) return false;
          seen.add(key);
          return true;
        })
        .slice(0, 5);
      statusMsg = options.length === 0 ? 'No matching places found.' : '';
      open = true;
    } catch (err: any) {
      if (err?.name === 'AbortError') return;
      options = [];
      statusMsg = 'Place search unavailable right now.';
      open = true;
    } finally {
      loading = false;
    }
  }

  function choose(opt: PlaceOption) {
    query = opt.name;
    options = [];
    open = false;
    activeIndex = -1;
    onSelect(opt.raw);
  }

  function clear() {
    query = '';
    handleInput();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown' && options.length > 0) {
      e.preventDefault();
      open = true;
      activeIndex = (activeIndex + 1) % options.length;
    } else if (e.key === 'ArrowUp' && options.length > 0) {
      e.preventDefault();
      activeIndex = (activeIndex - 1 + options.length) % options.length;
    } else if (e.key === 'Enter' && options.length > 0) {
      e.preventDefault();
      choose(options[Math.max(activeIndex, 0)]);
    } else if (e.key === 'Escape') {
      open = false;
    }
  }
</script>

<div class="place-search">
  <div class="input-wrap">
    <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
      <circle cx="11" cy="11" r="7"></circle>
      <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
    </svg>
    <input
      type="text"
      bind:value={query}
      oninput={handleInput}
      onkeydown={handleKeydown}
      onfocus={() => {
        if (options.length || statusMsg) open = true;
      }}
      onblur={() => {
        setTimeout(() => {
          open = false;
        }, 150);
      }}
      placeholder="Search town, city or area (e.g. Bristol, York)..."
      aria-label="Search place"
      role="combobox"
      aria-expanded={open && options.length > 0}
      aria-controls="place-results-listbox"
      aria-autocomplete="list"
      aria-activedescendant={activeIndex >= 0 ? `place-opt-${activeIndex}` : undefined}
    />
    {#if loading}
      <span class="mini-spinner" aria-hidden="true"></span>
    {:else if query}
      <button type="button" class="clear-btn" onclick={clear} aria-label="Clear search">×</button>
    {/if}
  </div>

  {#if open && (options.length > 0 || statusMsg)}
    <div id="place-results-listbox" class="dropdown panel-card" role="listbox">
      {#if statusMsg}
        <div class="status-row">{statusMsg}</div>
      {:else}
        {#each options as opt, idx}
          <button
            id="place-opt-{idx}"
            type="button"
            role="option"
            aria-selected={idx === activeIndex}
            class="opt-row"
            class:active={idx === activeIndex}
            onmousedown={(e) => {
              e.preventDefault();
              choose(opt);
            }}
          >
            <span class="opt-name">{opt.name}</span>
            {#if opt.context}
              <span class="opt-ctx">{opt.context}</span>
            {/if}
          </button>
        {/each}
      {/if}
    </div>
  {/if}
</div>

<style>
  .place-search {
    position: relative;
    width: 100%;
    max-width: 310px;
  }

  .input-wrap {
    position: relative;
    display: flex;
    align-items: center;
    height: 34px;
    background: var(--bg-surface);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-sm);
    padding: 0 8px 0 10px;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
  }

  .input-wrap:focus-within {
    border-color: var(--accent-civic);
    box-shadow: 0 0 0 2px var(--accent-civic-soft);
  }

  .search-icon {
    color: var(--text-muted);
    flex-shrink: 0;
    margin-right: 7px;
  }

  input {
    flex: 1;
    min-width: 0;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
  }

  input::placeholder {
    color: var(--text-muted);
  }

  .clear-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 15px;
    line-height: 1;
    padding: 2px 4px;
    cursor: pointer;
    border-radius: 4px;
  }

  .clear-btn:hover {
    color: var(--text-primary);
    background: var(--bg-secondary);
  }

  .mini-spinner {
    width: 13px;
    height: 13px;
    border: 2px solid var(--border-color);
    border-top-color: var(--accent-civic);
    border-radius: 50%;
    animation: spin 0.75s linear infinite;
    flex-shrink: 0;
  }

  .dropdown {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    right: 0;
    z-index: 60;
    overflow: hidden;
    padding: 4px;
  }

  .status-row {
    padding: 8px 10px;
    font-size: 12px;
    color: var(--text-muted);
  }

  .opt-row {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1px;
    padding: 7px 10px;
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    text-align: left;
    cursor: pointer;
  }

  .opt-row:hover,
  .opt-row.active {
    background: var(--bg-secondary);
  }

  .opt-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .opt-ctx {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }
</style>
