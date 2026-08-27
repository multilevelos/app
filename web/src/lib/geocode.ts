import type { SearchResult } from '../types';

/**
 * Search places using OpenStreetMap Nominatim API
 */
export async function searchPlace(query: string): Promise<SearchResult[]> {
  if (!query || query.trim().length < 2) return [];

  const url = `https://nominatim.openstreetmap.org/search?format=json&q=${encodeURIComponent(query)}&limit=5&addressdetails=1`;

  try {
    const response = await fetch(url, {
      headers: {
        'Accept-Language': 'en',
        'User-Agent': 'MultiLevelOS-Cycling-LoS-App'
      }
    });

    if (!response.ok) return [];

    const data: SearchResult[] = await response.json();
    return data;
  } catch (e) {
    console.error('Nominatim search failed:', e);
    return [];
  }
}
