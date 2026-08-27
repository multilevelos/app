import { WasmLosEngine, init as initWasm } from './wasm/multilevelos_wasm';
import type { LosAnalysisOutput, LosConfig } from '../types';

let wasmInitialized = false;

export async function ensureWasmInitialized(): Promise<void> {
  if (!wasmInitialized) {
    try {
      initWasm();
    } catch {
      // already initialized or module loaded
    }
    wasmInitialized = true;
  }
}

export class LosEngineWrapper {
  private engine: WasmLosEngine | null = null;

  async initFromOsmJson(osmJson: string): Promise<void> {
    await ensureWasmInitialized();
    this.engine = new WasmLosEngine(osmJson);
  }

  isReady(): boolean {
    return this.engine !== null;
  }

  calculateLos(config: LosConfig): LosAnalysisOutput {
    if (!this.engine) {
      throw new Error('Engine not initialized with OSM network data');
    }
    const configJson = JSON.stringify(config);
    const resultJson = this.engine.calculateLos(configJson);
    return JSON.parse(resultJson);
  }

  getDefaultConfig(): LosConfig {
    try {
      return JSON.parse(WasmLosEngine.getDefaultConfig());
    } catch {
      return {
        include_mixed_traffic: true,
        min_offroad_distance: 10.0,
        min_wide_width: 2.0,
        traffic_multiplier: 1.0,
        aadt_estimates: {
          motorway: 20000,
          motorway_link: 20000,
          trunk: 8000,
          trunk_link: 8000,
          primary: 6000,
          primary_link: 6000,
          secondary: 5000,
          secondary_link: 5000,
          tertiary: 3000,
          tertiary_link: 3000,
          residential: 500,
          service: 500,
          unclassified: 500,
          living_street: 500
        },
        speed_defaults: {
          motorway: 70,
          motorway_link: 70,
          trunk: 60,
          trunk_link: 60,
          primary: 40,
          primary_link: 40,
          secondary: 40,
          secondary_link: 40,
          tertiary: 40,
          tertiary_link: 40,
          residential: 30,
          unclassified: 30,
          living_street: 10,
          service: 10
        }
      };
    }
  }
}

export const engineInstance = new LosEngineWrapper();
