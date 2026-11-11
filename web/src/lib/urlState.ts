import type {
  BasemapId,
  ColorMode,
  DesignStandard,
  LosConfig,
  ThemeMode,
  TravelMode
} from '../types';

export interface UrlStateSnapshot {
  preset?: string;
  activeMode?: TravelMode;
  standard?: DesignStandard;
  showJunctions?: boolean;
  colorMode?: ColorMode;
  hiddenValues?: string[];
  basemap?: BasemapId;
  showPlaceNames?: boolean;
  theme?: ThemeMode;
  sidebarOpen?: boolean;
  printMode?: 'legend' | 'clean' | null;
  trafficMultiplier?: number;
  includeMixed?: boolean;
  minOffroad?: number;
  minWide?: number;
}

const VALID_MODES: TravelMode[] = ['multimodal', 'walking', 'wheeling', 'cycling'];
const VALID_STANDARDS: DesignStandard[] = ['ltn120', 'rural2026', 'cbd'];
const VALID_COLORS: ColorMode[] = [
  'los',
  'walking_category',
  'cycling_category',
  'infrastructure',
  'speed',
  'traffic'
];
const VALID_BASEMAPS: BasemapId[] = ['light', 'dark', 'voyager', 'cyclosm', 'satellite'];

export function readUrlState(): UrlStateSnapshot {
  if (typeof window === 'undefined') return {};
  const params = new URLSearchParams(window.location.search);
  const out: UrlStateSnapshot = {};

  const preset = params.get('preset');
  if (preset) out.preset = preset;

  const mode = params.get('mode') as TravelMode | null;
  if (mode && VALID_MODES.includes(mode)) out.activeMode = mode;

  const std = params.get('std') as DesignStandard | null;
  if (std && VALID_STANDARDS.includes(std)) out.standard = std;

  const junc = params.get('junc');
  if (junc === '0' || junc === 'false') out.showJunctions = false;
  if (junc === '1' || junc === 'true') out.showJunctions = true;

  const color = params.get('color') as ColorMode | null;
  if (color && VALID_COLORS.includes(color)) out.colorMode = color;

  const hide = params.get('hide');
  if (hide) {
    out.hiddenValues = hide
      .split('|')
      .map((s) => s.trim())
      .filter(Boolean);
  }

  const rawBasemap = params.get('basemap');
  const mappedBasemap = (rawBasemap === 'opencyclemap' ? 'cyclosm' : rawBasemap) as BasemapId | null;
  if (mappedBasemap && VALID_BASEMAPS.includes(mappedBasemap)) out.basemap = mappedBasemap;

  const labels = params.get('labels');
  if (labels === '0' || labels === 'false') out.showPlaceNames = false;
  if (labels === '1' || labels === 'true') out.showPlaceNames = true;

  const theme = params.get('theme');
  if (theme === 'dark' || theme === 'light') out.theme = theme;

  const sidebar = params.get('sidebar');
  if (sidebar === '0' || sidebar === 'false') out.sidebarOpen = false;
  if (sidebar === '1' || sidebar === 'true') out.sidebarOpen = true;

  const print = params.get('print');
  if (print === 'clean') out.printMode = 'clean';
  else if (print === '1' || print === 'true' || print === 'legend') out.printMode = 'legend';

  const mult = params.get('mult');
  if (mult && !Number.isNaN(Number(mult))) {
    out.trafficMultiplier = Math.min(3.0, Math.max(0.2, Number(mult)));
  }

  const mix = params.get('mix');
  if (mix === '0' || mix === 'false') out.includeMixed = false;

  const offroad = params.get('offroad');
  if (offroad && !Number.isNaN(Number(offroad))) {
    out.minOffroad = Math.min(30, Math.max(5, Number(offroad)));
  }

  const wide = params.get('wide');
  if (wide && !Number.isNaN(Number(wide))) {
    out.minWide = Math.min(3.5, Math.max(1.5, Number(wide)));
  }

  return out;
}

let replaceTimer: ReturnType<typeof setTimeout> | null = null;

export function buildShareUrl(state: {
  presetId: string | null;
  colorMode: ColorMode;
  hiddenValues: string[];
  showJunctions: boolean;
  basemap: BasemapId;
  showPlaceNames: boolean;
  theme: ThemeMode;
  sidebarOpen: boolean;
  printMode: 'legend' | 'clean' | null;
  config: LosConfig;
}): string {
  if (typeof window === 'undefined') return '';
  const params = new URLSearchParams();

  if (state.presetId && state.presetId !== 'edinburgh') {
    params.set('preset', state.presetId);
  }
  if (state.config.active_mode !== 'multimodal') {
    params.set('mode', state.config.active_mode);
  }
  if (state.config.standard !== 'ltn120') {
    params.set('std', state.config.standard);
  }
  if (!state.showJunctions) {
    params.set('junc', '0');
  }
  if (state.colorMode !== 'los') {
    params.set('color', state.colorMode);
  }
  if (state.hiddenValues.length > 0) {
    params.set('hide', state.hiddenValues.join('|'));
  }
  if (state.basemap !== 'light') {
    params.set('basemap', state.basemap);
  }
  if (!state.showPlaceNames) {
    params.set('labels', '0');
  }
  if (state.theme !== 'light') {
    params.set('theme', state.theme);
  }
  if (!state.sidebarOpen) {
    params.set('sidebar', '0');
  }
  if (state.printMode) {
    params.set('print', state.printMode);
  }
  if (Math.abs(state.config.traffic_multiplier - 1.0) > 0.01) {
    params.set('mult', state.config.traffic_multiplier.toFixed(1));
  }
  if (!state.config.include_mixed_traffic) {
    params.set('mix', '0');
  }
  if (Math.abs(state.config.min_offroad_distance - 10) > 0.1) {
    params.set('offroad', String(state.config.min_offroad_distance));
  }
  if (Math.abs(state.config.min_wide_width - 2.0) > 0.01) {
    params.set('wide', state.config.min_wide_width.toFixed(1));
  }

  const qs = params.toString();
  const hash = window.location.hash || '';
  return `${window.location.origin}${window.location.pathname}${qs ? '?' + qs : ''}${hash}`;
}

export function syncUrlStateDebounced(state: Parameters<typeof buildShareUrl>[0]): void {
  if (typeof window === 'undefined') return;
  if (replaceTimer) clearTimeout(replaceTimer);
  replaceTimer = setTimeout(() => {
    const nextUrl = buildShareUrl(state);
    window.history.replaceState(null, '', nextUrl);
  }, 180);
}
