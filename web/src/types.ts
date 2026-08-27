import type { FeatureCollection } from 'geojson';

export interface LosConfig {
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

export interface SummaryStats {
  total_length_km: number;
  total_segments: number;
  los_breakdown: Record<string, CategoryMetric>;
  infra_breakdown: Record<string, CategoryMetric>;
  high_med_pct: number;
  unsuitable_pct: number;
  average_speed_mph: number;
}

export interface LosAnalysisOutput {
  segments_count: number;
  summary: SummaryStats;
  geojson: FeatureCollection;
}

export type ColorMode = 'los' | 'infrastructure' | 'speed' | 'traffic';

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
