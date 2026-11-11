import type { SearchResult } from '../types';

/**
 * Search places using OpenStreetMap Nominatim API with optional AbortSignal
 */
export async function searchPlace(query: string, signal?: AbortSignal): Promise<SearchResult[]> {
  if (!query || query.trim().length < 2) return [];

  const params = new URLSearchParams({
    q: query.trim(),
    format: 'jsonv2',
    addressdetails: '1',
    limit: '6',
    'accept-language': 'en'
  });

  const response = await fetch(`https://nominatim.openstreetmap.org/search?${params.toString()}`, {
    signal
  });

  if (!response.ok) {
    throw new Error(`Nominatim error (${response.status})`);
  }

  return (await response.json()) as SearchResult[];
}
