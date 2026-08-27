# MultiLevelOS: Cycling Level of Service (LoS) Web App

An interactive, high-performance, Rust- and WebAssembly-powered web application for downloading OpenStreetMap (OSM) highway and cycling networks on the fly and estimating **Cycling Level of Service (LoS)** using road class-based Annual Average Daily Traffic (AADT) volume and speed models.

Based on Transport Scotland's **Cycling by Design (CbD)** and Design Manual for Roads and Bridges (CD 195) standards, adapted from the R package [`nptscot/osmactive`](https://github.com/nptscot/osmactive) and modern web architecture from [`bicischools-web`](https://github.com/Robinlovelace/bicischools-web).

---

## Key Features

- **On-the-Fly OpenStreetMap Downloads**:
  - Fetch routable street and active travel networks directly from Overpass API for the current visible map viewport.
  - Search any town, city, or neighborhood worldwide using integrated Nominatim geocoding.
  - Pre-packaged presets for instant offline testing (Edinburgh, Leeds, Cambridge).
  - Custom Overpass / OSM JSON drag-and-drop file upload.
- **Rust Core & WebAssembly Engine (`multilevelos-core`)**:
  - Blazing fast network parsing and spatial analysis running client-side directly in the browser via WebAssembly.
  - R-tree spatial indexing (`rstar`) for computing real-time physical distance between active travel paths and adjacent general traffic carriageways.
  - Sub-15ms recalculation times when adjusting traffic volumes, road class assumptions, or speed limit thresholds.
- **Cycling by Design (CbD) Level of Service Matrix**:
  - Classifies infrastructure into 6 physical segregation types:
    - *Segregated Track (wide)* (width $\ge$ 2.0m)
    - *Segregated Track (narrow)* (width < 2.0m)
    - *Off Road Path* (trails, greenways, towpaths, paths $> 10\text{m}$ from carriageways)
    - *Shared Footway* (shared pavement / pedestrian space)
    - *Painted Cycle Lane* (on-road advisory or mandatory painted lane)
    - *Mixed Traffic Street* (carriageway with no dedicated cycle infrastructure)
  - Scores links into 5 standard Level of Service tiers:
    - **High (LoS 3)**: Suitable for all ages and abilities (Green)
    - **Medium (LoS 2)**: Acceptable for most adults (Yellow)
    - **Low (LoS 1)**: High stress / low quality (Orange)
    - **Should not be used - Mixed Traffic (LoS 0)**: Unsafe high speed/traffic mixing (Red)
    - **Should not be used - Non-Compliant (LoS -1)**: Substandard intervention (Purple)
- **Road Class-Based Traffic (AADT) & Speed Estimation**:
  - Automatically estimates Annual Average Daily Traffic based on OSM road hierarchies (Motorway: 20k, Trunk: 8k, Primary: 6k, Secondary: 5k, Tertiary: 3k, Residential/Service: 500).
  - Interactive sliders to dynamically scale global traffic multipliers or tune per-class traffic volumes and speed limits.
- **Map Visualization & Analytics**:
  - Interactive MapLibre GL rendering with customizable basemaps (Dark, Light, Streets).
  - Toggle color schemes by *Level of Service*, *Infrastructure Type*, *Speed Limit*, or *AADT Traffic Volume*.
  - Click inspection popups with detailed link attributes and direct OpenStreetMap way links.
  - Summary KPI cards and distribution breakdown charts.
- **Data Export**:
  - Export classified network directly to standard GeoJSON for GIS software (QGIS, R `sf`, Python `geopandas`).
  - Export statistical breakdown report to CSV.

---

## Project Structure

```
app/
├── Cargo.toml                    # Rust workspace manifest
├── crates/
│   ├── multilevelos-core/        # Pure Rust library: OSM parsing, spatial R-tree index, CbD matrix
│   └── multilevelos-cli/         # CLI tool for processing OSM files in terminal
├── bindings/
│   └── wasm/                     # wasm-bindgen bindings for browser execution
├── scripts/
│   └── build-wasm.sh             # Compiles Rust to WebAssembly (wasm-pack)
├── web/                          # Modern Svelte 5 + Vite + MapLibre GL frontend
│   ├── src/
│   │   ├── lib/
│   │   │   ├── engine.ts         # WASM engine wrapper
│   │   │   ├── overpass.ts       # Overpass API fetcher with fallbacks
│   │   │   ├── geocode.ts        # Nominatim place search
│   │   │   ├── map/              # MapLibre map component & styling
│   │   │   └── components/       # Sidebar, Legend, KPI cards, controls
│   │   ├── App.svelte
│   │   └── main.ts
│   └── public/presets/           # Cached sample Overpass datasets
└── package.json                  # Root npm scripts
```

---

## Quickstart & Local Development

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) (version 1.80+)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (`cargo install wasm-pack` or npm)
- [Node.js](https://nodejs.org/) (v18+) and npm

### 1. Build the WebAssembly Module

```bash
npm run build:wasm
```

### 2. Start the Local Development Server

```bash
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

---

## Command Line Interface (CLI)

You can also run the Level of Service analysis directly on local OSM JSON files from your terminal:

```bash
cargo run -p multilevelos-cli -- web/public/presets/edinburgh.json output_los.geojson
```

---

## License

MIT License.
