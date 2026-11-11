import type { FeatureCollection } from 'geojson';

export type TravelMode = 'multimodal' | 'walking' | 'wheeling' | 'cycling';
export type DesignStandard = 'ltn120' | 'rural2026' | 'cbd';

export interface LosConfig {
  active_mode: TravelMode;
  standard: DesignStandard;
  include_junctions: boolean;
  include_mixed_traffic: boolean;
  min_offroad_distance: number;
  min_wide_width: number;
  traffic_multiplier: number;
  aadt_estimates: Record<string, number>;
  speed_defaults: Record<string, number>;
}

export interface CategoryMetric {
  length_km: number;
  percentage: number;
  count: number;
  color: string;
}

export interface ModeSuitability {
  multimodal_ab_pct: number;
  walking_ab_pct: number;
  wheeling_ab_pct: number;
  cycling_ab_pct: number;
}

export interface JunctionSummaryStats {
  total_junctions: number;
  signalized_count: number;
  marked_crossing_count: number;
  suitable_pct: number;
  hostile_pct: number;
  grade_breakdown: Record<string, number>;
}

export interface SummaryStats {
  total_length_km: number;
  total_segments: number;
  los_breakdown: Record<string, CategoryMetric>;
  infra_breakdown: Record<string, CategoryMetric>;
  walking_breakdown: Record<string, CategoryMetric>;
  cycling_breakdown: Record<string, CategoryMetric>;
  high_med_pct: number;
  unsuitable_pct: number;
  average_speed_mph: number;
  mode_suitability: ModeSuitability;
  junctions: JunctionSummaryStats;
}

export interface LosAnalysisOutput {
  segments_count: number;
  junctions_count: number;
  summary: SummaryStats;
  geojson: FeatureCollection;
  junctions_geojson: FeatureCollection;
}

export type ColorMode =
  | 'los'
  | 'walking_category'
  | 'cycling_category'
  | 'infrastructure'
  | 'speed'
  | 'traffic';
export type ThemeMode = 'light' | 'dark';
export type BasemapId = 'light' | 'dark' | 'voyager' | 'cyclosm' | 'satellite';

export interface SelectedSegment {
  kind: 'segment';
  osm_id: number;
  name: string;
  highway: string;
  infrastructure: string;
  walking_category: string;
  cycling_category: string;
  prow_designation: string | null;
  rural_surface: string;
  separation: string;
  surface: string;
  width_m: number | null;
  incline_pct: number | null;
  is_lit: boolean;
  is_urban: boolean;
  speed_limit_mph: number | null;
  speed_category: string;
  estimated_aadt: number | null;
  aadt_category: string;
  distance_to_road_m: number | null;
  length_m: number;
  grade_multimodal: string;
  grade_walking: string;
  grade_wheeling: string;
  grade_cycling: string;
  active_grade: string;
  cbd_label: string;
  los_score: number;
  los_label: string;
  los_color: string;
  center?: [number, number] | null;
  bbox?: [number, number, number, number] | null;
}

export interface SelectedJunction {
  kind: 'junction';
  junction_id: number;
  name: string;
  arm_count: number;
  control_type: string;
  is_roundabout: boolean;
  has_tactile_paving: boolean;
  max_speed_mph: number;
  max_aadt: number;
  radius_m: number;
  grade_multimodal: string;
  grade_walking: string;
  grade_wheeling: string;
  grade_cycling: string;
  active_grade: string;
  los_color: string;
  center?: [number, number] | null;
  bbox?: [number, number, number, number] | null;
}

export type SelectedFeature = SelectedSegment | SelectedJunction;

export interface PresetLocation {
  id: string;
  name: string;
  country: string;
  description: string;
  center: [number, number]; // [lng, lat]
  zoom: number;
  bbox: [number, number, number, number]; // [minLng, minLat, maxLng, maxLat]
}

export interface SearchResult {
  place_id: number;
  display_name: string;
  lat: string;
  lon: string;
  boundingbox: [string, string, string, string]; // [minLat, maxLat, minLng, maxLng]
}

export interface LegendMetricRow {
  value: string;
  label: string;
  desc?: string;
  color: string;
  km: number;
  pct: number;
  count: number;
}
