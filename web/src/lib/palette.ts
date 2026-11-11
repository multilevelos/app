import type { Feature } from 'geojson';
import type {
  ColorMode,
  DesignStandard,
  LegendMetricRow,
  PresetLocation,
  SummaryStats,
  TravelMode
} from '../types';

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
    description: 'Extensive 20mph network, off-road railway paths, steps, and protected lanes.',
    center: [-3.195, 55.952],
    zoom: 14.5,
    bbox: [-3.22, 55.94, -3.17, 55.965]
  },
  {
    id: 'leeds',
    name: 'Leeds (University & Core)',
    country: 'England, UK',
    description: 'Major LTN 1/20 urban corridors, signalized junctions, and student routes.',
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
  },
  {
    id: 'bakewell',
    name: 'Peak District (Bakewell & Monsal Trail)',
    country: 'Derbyshire, UK',
    description:
      'ATE Rural Design Guide showcase: Monsal Trail greenway, bridleways, footpaths, restricted byways & country lanes.',
    center: [-1.678, 53.216],
    zoom: 14.2,
    bbox: [-1.705, 53.205, -1.655, 53.232]
  }
];

export const SCHEMES: Record<ColorMode, ColorSchemeDef> = {
  los: {
    id: 'los',
    title: 'Multi-Modal Level of Service (A–E)',
    property: 'los_label',
    fallbackColor: '#059669',
    entries: [
      {
        value: 'Grade A (Excellent)',
        label: 'Grade A · Excellent',
        desc: 'All ages & abilities quality',
        color: '#059669',
        tone: 'success'
      },
      {
        value: 'Grade B (Good)',
        label: 'Grade B · Good',
        desc: 'Low-stress active travel link',
        color: '#65a30d',
        tone: 'success'
      },
      {
        value: 'Grade C (Moderate)',
        label: 'Grade C · Moderate',
        desc: 'Moderate stress or shared space',
        color: '#eab308',
        tone: 'warning'
      },
      {
        value: 'Grade D (Poor)',
        label: 'Grade D · Poor',
        desc: 'High traffic exposure or constraint',
        color: '#f97316',
        tone: 'warning'
      },
      {
        value: 'Grade E (Hostile / Barrier)',
        label: 'Grade E · Hostile / Barrier',
        desc: 'Unsuitable or inaccessible',
        color: '#dc2626',
        tone: 'danger'
      }
    ]
  },
  walking_category: {
    id: 'walking_category',
    title: 'Walking Categories (ATE Rural Guide)',
    property: 'walking_category',
    fallbackColor: '#94a3b8',
    entries: [
      {
        value: 'Motor-Free Path / PRoW',
        label: 'Motor-Free Path / PRoW',
        desc: 'Public Footpaths, Bridleways, Restricted Byways & greenways (Table 8.1)',
        color: '#15803d',
        tone: 'success'
      },
      {
        value: 'Protected Footway',
        label: 'Protected Footway',
        desc: 'Kerbed or verge-buffered roadside footway / pedestrian zone (Table 4-3)',
        color: '#059669',
        tone: 'success'
      },
      {
        value: 'Shared-Use Path',
        label: 'Shared-Use Path',
        desc: 'Shared walking, wheeling & cycling route (Table 4-4, >=3.0m desirable)',
        color: '#0d9488',
        tone: 'success'
      },
      {
        value: 'Virtual / Implied Footway',
        label: 'Virtual / Implied Footway',
        desc: 'Edge-marked rural carriageway footway (Ch. 7.5 & Table 4-3)',
        color: '#0284c7',
        tone: 'civic'
      },
      {
        value: 'Quiet Lane (Mixed)',
        label: 'Quiet Lane (Mixed)',
        desc: 'Designated Quiet Lane, BOAT, Home Zone, or <=20mph <1,000 vpd lane (Ch. 7.2)',
        color: '#65a30d',
        tone: 'success'
      },
      {
        value: 'Carriageway (No Footway)',
        label: 'Carriageway (No Footway)',
        desc: 'Walking in mixed motor traffic with no footway (Table 4-3)',
        color: '#dc2626',
        tone: 'danger'
      },
      {
        value: 'Steps (Stepped Access)',
        label: 'Steps (Stepped Access)',
        desc: 'Walkable stairs — barrier for wheeling',
        color: '#9333ea',
        tone: 'purple'
      }
    ]
  },
  cycling_category: {
    id: 'cycling_category',
    title: 'Cycling Categories (ATE Rural Guide)',
    property: 'cycling_category',
    fallbackColor: '#94a3b8',
    entries: [
      {
        value: 'Motor-Free Greenway / Bridleway',
        label: 'Motor-Free Greenway / Bridleway',
        desc: 'Off-road cycleway, bridleway, restricted byway or towpath (Ch. 8 & Table 8.1)',
        color: '#15803d',
        tone: 'success'
      },
      {
        value: 'Fully Protected Cycle Track',
        label: 'Fully Protected Cycle Track',
        desc: 'Kerb-, barrier- or verge-protected cycle track (Table 4-3 & Ch. 6.2)',
        color: '#059669',
        tone: 'success'
      },
      {
        value: 'Stepped Cycle Track',
        label: 'Stepped Cycle Track',
        desc: 'Intermediate-height stepped track <=30 mph (Table 4-3)',
        color: '#65a30d',
        tone: 'success'
      },
      {
        value: 'Shared-Use Path / Track',
        label: 'Shared-Use Path / Track',
        desc: 'Shared walking, equestrian & cycling path (Table 4-4 & Ch. 5.3)',
        color: '#d97706',
        tone: 'warning'
      },
      {
        value: 'Light Segregation',
        label: 'Light Segregation',
        desc: 'Intermittent bollards, wands, or planters (Table 4-3)',
        color: '#0d9488',
        tone: 'success'
      },
      {
        value: 'Cycle Lane (Mandatory / Advisory)',
        label: 'Cycle Lane (Mandatory / Advisory)',
        desc: 'On-carriageway cycle lane or centreline-removed lane (Ch. 6.4 & Table 4-3)',
        color: '#ea580c',
        tone: 'warning'
      },
      {
        value: 'Quiet Lane / Country Lane',
        label: 'Quiet Lane / Country Lane',
        desc: 'Quiet Lane, BOAT, or <=20mph / rural <=30mph <1,000 vpd lane (Ch. 7 & Footnote 5)',
        color: '#0284c7',
        tone: 'civic'
      },
      {
        value: 'Mixed Traffic Road',
        label: 'Mixed Traffic Road',
        desc: 'General carriageway mixed with motor traffic (Table 4-3)',
        color: '#64748b',
        tone: 'neutral'
      },
      {
        value: 'No Cycling (Footpath / Steps)',
        label: 'No Cycling (Footpath / Steps)',
        desc: 'Public footpath, pedestrian-only footway or steps (Table 8.1)',
        color: '#94a3b8',
        tone: 'neutral'
      }
    ]
  },
  infrastructure: {
    id: 'infrastructure',
    title: 'Infrastructure & Facility Type',
    property: 'infrastructure',
    fallbackColor: '#64748b',
    entries: [
      {
        value: 'Segregated Track (wide)',
        label: 'Segregated Track (wide)',
        desc: 'Protected cycleway >= wide threshold',
        color: '#059669',
        tone: 'success'
      },
      {
        value: 'Off Road Path',
        label: 'Off-Road Path',
        desc: 'Greenway, towpath, or park path away from traffic',
        color: '#16a34a',
        tone: 'success'
      },
      {
        value: 'Bridleway / Rural Greenway',
        label: 'Bridleway / Rural Greenway',
        desc: 'Public Bridleway or Restricted Byway open to walkers, cyclists & horses',
        color: '#15803d',
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
        value: 'Stepped Cycle Track',
        label: 'Stepped Cycle Track',
        desc: 'Intermediate-height stepped track beside carriageway',
        color: '#84cc16',
        tone: 'success'
      },
      {
        value: 'Light Segregation',
        label: 'Light Segregation',
        desc: 'Wands, bollards, planters, or light physical protection',
        color: '#0d9488',
        tone: 'success'
      },
      {
        value: 'Bus / Shared Priority Lane',
        label: 'Bus / Priority Lane',
        desc: 'Dedicated bus & cycle priority lane',
        color: '#0284c7',
        tone: 'civic'
      },
      {
        value: 'Pedestrian Footway / Zone',
        label: 'Pedestrian Footway / Zone',
        desc: 'Dedicated footway or pedestrianised street',
        color: '#2563eb',
        tone: 'civic'
      },
      {
        value: 'Virtual / Implied Footway',
        label: 'Virtual / Implied Footway',
        desc: 'Edge-marked rural carriageway footway',
        color: '#0ea5e9',
        tone: 'civic'
      },
      {
        value: 'Quiet Lane / Country Lane',
        label: 'Quiet Lane / Country Lane',
        desc: 'Designated Quiet Lane, BOAT, or Home Zone',
        color: '#22c55e',
        tone: 'success'
      },
      {
        value: 'Shared Footway',
        label: 'Shared Footway',
        desc: 'Shared pedestrian & cycling path',
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
      },
      {
        value: 'Steps (Stepped Access)',
        label: 'Steps (Stepped Access)',
        desc: 'Walkable stairs — barrier for wheeling/cycling',
        color: '#9333ea',
        tone: 'purple'
      }
    ]
  },
  speed: {
    id: 'speed',
    title: 'Effective Speed Limit',
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

export function gradeColor(grade: string | undefined): string {
  switch ((grade || '').trim().toUpperCase()) {
    case 'A':
      return '#059669';
    case 'B':
      return '#65a30d';
    case 'C':
      return '#eab308';
    case 'D':
      return '#f97316';
    case 'E':
      return '#dc2626';
    default:
      return '#64748b';
  }
}

export function gradeTone(grade: string | undefined): BadgeTone {
  switch ((grade || '').trim().toUpperCase()) {
    case 'A':
    case 'B':
      return 'success';
    case 'C':
    case 'D':
      return 'warning';
    case 'E':
      return 'danger';
    default:
      return 'civic';
  }
}

export function getLosTone(losLabel: string | undefined, losScore?: number): BadgeTone {
  if (losScore === 4 || losScore === 3 || losLabel?.startsWith('Grade A') || losLabel?.startsWith('Grade B')) {
    return 'success';
  }
  if (losScore === 2 || losScore === 1 || losLabel?.startsWith('Grade C') || losLabel?.startsWith('Grade D')) {
    return 'warning';
  }
  if (losScore === 0 || losLabel?.startsWith('Grade E')) {
    return 'danger';
  }
  return 'civic';
}

export function formatShortLosLabel(losLabel: string | undefined, activeGrade?: string): string {
  if (activeGrade) {
    const g = activeGrade.toUpperCase();
    if (g === 'A') return 'Grade A · Excellent';
    if (g === 'B') return 'Grade B · Good';
    if (g === 'C') return 'Grade C · Moderate';
    if (g === 'D') return 'Grade D · Poor';
    if (g === 'E') return 'Grade E · Hostile';
  }
  if (losLabel?.startsWith('Grade A')) return 'Grade A · Excellent';
  if (losLabel?.startsWith('Grade B')) return 'Grade B · Good';
  if (losLabel?.startsWith('Grade C')) return 'Grade C · Moderate';
  if (losLabel?.startsWith('Grade D')) return 'Grade D · Poor';
  if (losLabel?.startsWith('Grade E')) return 'Grade E · Hostile';
  return losLabel || 'Classified';
}

export function formatModeLabel(mode: TravelMode): string {
  switch (mode) {
    case 'multimodal':
      return 'Multi-Modal';
    case 'walking':
      return 'Walking';
    case 'wheeling':
      return 'Wheeling';
    case 'cycling':
      return 'Cycling';
  }
}

export function formatStandardLabel(std: DesignStandard): string {
  if (std === 'rural2026') return 'ATE Rural Guide (2026)';
  return std === 'ltn120' ? 'England LTN 1/20' : 'Scotland CbD';
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

  if (mode === 'walking_category' && summary) {
    return scheme.entries.map((e) => {
      const m = summary.walking_breakdown?.[e.value];
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

  if (mode === 'cycling_category' && summary) {
    return scheme.entries.map((e) => {
      const m = summary.cycling_breakdown?.[e.value];
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
