import type { Feature } from 'geojson';
import type { ColorMode, LegendMetricRow, PresetLocation, SummaryStats } from '../types';

export type BadgeTone = 'success' | 'warning' | 'danger' | 'purple' | 'civic' | 'neutral';

export interface SchemeEntry {
  /** Property value stored on GeoJSON features */
  value: string;
  /** Concise UI display label */
  label: string;
  /** Optional explanatory subtitle */
  desc?: string;
  /** Hex colour used across map, legend, and composition bars */
  color: string;
  /** Badge semantic tone */
  tone: BadgeTone;
}

export interface ColorSchemeDef {
  id: ColorMode;
  title: string;
  property: string;
  fallbackColor: string;
  entries: SchemeEntry[];
}

export const PRESET_LOCATIONS: PresetLocation[] = [
  {
    id: 'edinburgh',
    name: 'Edinburgh (City Centre)',
    country: 'Scotland, UK',
    description: 'Extensive 20mph network, off-road railway paths, and protected lanes.',
    center: [-3.195, 55.952],
    zoom: 14.5,
    bbox: [-3.22, 55.94, -3.17, 55.965]
  },
  {
    id: 'leeds',
    name: 'Leeds (University & Core)',
    country: 'England, UK',
    description: 'Major urban corridors, high traffic arterials, and student routes.',
    center: [-1.55, 53.8],
    zoom: 14.5,
    bbox: [-1.575, 53.788, -1.525, 53.812]
  },
  {
    id: 'cambridge',
    name: 'Cambridge (Historic Core)',
    country: 'England, UK',
    description: 'High active travel mode share, shared paths, and pedestrianised historic core.',
    center: [0.122, 52.205],
    zoom: 14.5,
    bbox: [0.1, 52.19, 0.15, 52.22]
  }
];

export const SCHEMES: Record<ColorMode, ColorSchemeDef> = {
  los: {
    id: 'los',
    title: 'Level of Service (LoS)',
    property: 'los_label',
    fallbackColor: '#16a34a',
    entries: [
      {
        value: 'High',
        label: 'High (LoS 3)',
        desc: 'Suitable for all ages & abilities',
        color: '#16a34a',
        tone: 'success'
      },
      {
        value: 'Medium',
        label: 'Medium (LoS 2)',
        desc: 'Acceptable for most users',
        color: '#eab308',
        tone: 'warning'
      },
      {
        value: 'Low',
        label: 'Low (LoS 1)',
        desc: 'High stress / narrow or busy',
        color: '#f97316',
        tone: 'warning'
      },
      {
        value: 'Should not be used (mixed traffic)',
        label: 'Unsuitable (Mixed)',
        desc: 'High-speed or high-flow mixing',
        color: '#dc2626',
        tone: 'danger'
      },
      {
        value: 'Should not be used (non-compliant intervention)',
        label: 'Non-Compliant',
        desc: 'Substandard facility for road conditions',
        color: '#9333ea',
        tone: 'purple'
      }
    ]
  },
  infrastructure: {
    id: 'infrastructure',
    title: 'Infrastructure Type',
    property: 'infrastructure',
    fallbackColor: '#64748b',
    entries: [
      {
        value: 'Segregated Track (wide)',
        label: 'Segregated Track (wide)',
        desc: 'Dedicated track >= min wide threshold',
        color: '#059669',
        tone: 'success'
      },
      {
        value: 'Off Road Path',
        label: 'Off-Road Path',
        desc: 'Greenway, towpath, or path away from traffic',
        color: '#16a34a',
        tone: 'success'
      },
      {
        value: 'Segregated Track (narrow)',
        label: 'Segregated Track (narrow)',
        desc: 'Protected cycleway < wide threshold',
        color: '#65a30d',
        tone: 'success'
      },
      {
        value: 'Shared Footway',
        label: 'Shared Footway',
        desc: 'Shared pedestrian & active travel space',
        color: '#d97706',
        tone: 'warning'
      },
      {
        value: 'Painted Cycle Lane',
        label: 'Painted Cycle Lane',
        desc: 'On-carriageway mandatory or advisory lane',
        color: '#ea580c',
        tone: 'warning'
      },
      {
        value: 'Mixed Traffic Street',
        label: 'Mixed Traffic Street',
        desc: 'General carriageway with no dedicated lane',
        color: '#64748b',
        tone: 'neutral'
      }
    ]
  },
  speed: {
    id: 'speed',
    title: 'Speed Limit',
    property: 'speed_category',
    fallbackColor: '#16a34a',
    entries: [
      { value: '<20 mph', label: '<20 mph / Traffic-free', color: '#16a34a', tone: 'success' },
      { value: '20 mph', label: '20 mph', color: '#65a30d', tone: 'success' },
      { value: '30 mph', label: '30 mph', color: '#eab308', tone: 'warning' },
      { value: '40 mph', label: '40 mph', color: '#f97316', tone: 'warning' },
      { value: '50 mph', label: '50 mph', color: '#ef4444', tone: 'danger' },
      { value: '60+ mph', label: '60+ mph', color: '#991b1b', tone: 'danger' }
    ]
  },
  traffic: {
    id: 'traffic',
    title: 'Traffic Volume (AADT)',
    property: 'aadt_category',
    fallbackColor: '#16a34a',
    entries: [
      { value: '0 to 999', label: '0–999 AADT (Quiet)', color: '#16a34a', tone: 'success' },
      { value: '1000 to 1999', label: '1,000–1,999 AADT', color: '#eab308', tone: 'warning' },
      { value: '2000 to 3999', label: '2,000–3,999 AADT', color: '#f97316', tone: 'warning' },
      { value: '4000+', label: '4,000+ AADT (Busy)', color: '#dc2626', tone: 'danger' }
    ]
  }
};

export function getColorExpression(mode: ColorMode): any {
  const scheme = SCHEMES[mode];
  const expr: any[] = ['match', ['get', scheme.property]];
  for (const entry of scheme.entries) {
    expr.push(entry.value, entry.color);
  }
  expr.push(scheme.fallbackColor);
  return expr;
}

export function getFilterExpression(mode: ColorMode, hiddenValues: string[]): any | null {
  if (!hiddenValues || hiddenValues.length === 0) return null;
  const prop = SCHEMES[mode].property;
  return ['!', ['in', ['get', prop], ['literal', hiddenValues]]];
}

export function getLosTone(losLabel: string | undefined, losScore?: number): BadgeTone {
  if (losScore === 3 || losLabel === 'High') return 'success';
  if (losScore === 2 || losLabel === 'Medium') return 'warning';
  if (losScore === 1 || losLabel === 'Low') return 'warning';
  if (losScore === -1 || losLabel?.includes('non-compliant')) return 'purple';
  if (losScore === 0 || losLabel?.includes('Should not be used')) return 'danger';
  return 'civic';
}

export function formatShortLosLabel(losLabel: string | undefined, losScore?: number): string {
  if (losScore === 3 || losLabel === 'High') return 'High (LoS 3)';
  if (losScore === 2 || losLabel === 'Medium') return 'Medium (LoS 2)';
  if (losScore === 1 || losLabel === 'Low') return 'Low (LoS 1)';
  if (losScore === -1 || losLabel?.includes('non-compliant')) return 'Non-Compliant';
  if (losScore === 0 || losLabel?.includes('mixed')) return 'Unsuitable (Mixed)';
  return losLabel || 'Classified';
}

export function buildLegendMetrics(
  mode: ColorMode,
  summary: SummaryStats | null,
  features: Feature[]
): LegendMetricRow[] {
  const scheme = SCHEMES[mode];
  if (!features || features.length === 0) {
    return scheme.entries.map((e) => ({
      value: e.value,
      label: e.label,
      desc: e.desc,
      color: e.color,
      km: 0,
      pct: 0,
      count: 0
    }));
  }

  if (mode === 'los' && summary) {
    return scheme.entries.map((e) => {
      const m = summary.los_breakdown[e.value];
      return {
        value: e.value,
        label: e.label,
        desc: e.desc,
        color: e.color,
        km: m ? m.length_km : 0,
        pct: m ? m.percentage : 0,
        count: m ? m.count : 0
      };
    });
  }

  if (mode === 'infrastructure' && summary) {
    return scheme.entries.map((e) => {
      const m = summary.infra_breakdown[e.value];
      return {
        value: e.value,
        label: e.label,
        desc: e.desc,
        color: e.color,
        km: m ? m.length_km : 0,
        pct: m ? m.percentage : 0,
        count: m ? m.count : 0
      };
    });
  }

  // Aggregate directly from GeoJSON features for speed / traffic modes
  const totals = new Map<string, { lenM: number; count: number }>();
  let totalM = 0;

  for (const f of features) {
    const props = f.properties || {};
    const key = String(props[scheme.property] ?? '');
    const lenM = Number(props.length_m) || 0;
    totalM += lenM;
    const prev = totals.get(key) || { lenM: 0, count: 0 };
    prev.lenM += lenM;
    prev.count += 1;
    totals.set(key, prev);
  }

  return scheme.entries.map((e) => {
    const bucket = totals.get(e.value) || { lenM: 0, count: 0 };
    const km = Math.round((bucket.lenM / 1000) * 100) / 100;
    const pct = totalM > 0 ? Math.round((bucket.lenM / totalM) * 1000) / 10 : 0;
    return {
      value: e.value,
      label: e.label,
      desc: e.desc,
      color: e.color,
      km,
      pct,
      count: bucket.count
    };
  });
}

export function escapeHtml(str: unknown): string {
  return String(str ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}
