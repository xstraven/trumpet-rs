# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Build WASM (outputs to web/pkg/)
wasm-pack build --target web --out-dir web/pkg

# Run all Rust tests (26 tests across parser, pitch, scoring, exercises, transposition)
cargo test

# Run a single test
cargo test test_name
# e.g. cargo test test_yin_a440

# Serve the web app locally
cd web && python3 -m http.server 8080
# Then open http://localhost:8080
```

No linter or formatter config exists. No JS test framework is used.

## Architecture

This is a Bb trumpet practice app. Rust handles computation (parsing, pitch detection, analysis), compiled to WASM via `wasm-pack`. The frontend is vanilla JS with Canvas 2D rendering. Data crosses the WASM boundary via `serde-wasm-bindgen` (Rust structs serialize to JS objects automatically).

### Rust Modules (`src/`)

- **`lib.rs`** — Thin WASM facade only. Four `#[wasm_bindgen]` exports: `parse_musicxml`, `detect_pitch`, `analyze_performance`, `generate_exercise`. Each wraps a pure Rust function and converts between `JsValue` and Rust types. Core logic is testable without WASM.
- **`parser/musicxml.rs`** — Streaming MusicXML parser (quick-xml). Extracts notes, measures, tempo, key/time signatures, transpose metadata. Returns `Result<Score, String>`.
- **`pitch/yin.rs`** — YIN pitch detection algorithm tuned for trumpet (80–1200 Hz). Returns `PitchResult { hz, confidence, midi_float }`. The `midi_float` is fractional (e.g. 69.3 for slightly sharp A4) enabling intonation visualization.
- **`scoring/types.rs`** — All shared data structures: `Score`, `NoteEvent`, `MeasureInfo`, `PlayedNote`, `PerformanceAnalysis`, etc.
- **`scoring/analyzer.rs`** — Greedy note-matching algorithm comparing played notes to score. Produces pitch/timing error stats, interval problem detection, natural-language feedback, and an overall score (0–100).
- **`exercises/generators.rs`** — Generates synthetic `Score` objects for 6 exercise types (long_tones, major_scale, chromatic, lip_slurs, intervals, arpeggios). No XML involved.
- **`transposition.rs`** — Concert pitch ↔ written pitch conversion. Bb trumpet: chromatic=-2.

### JS Modules (`web/`)

- **`main.js`** — Entry point. Boots WASM, wires DOM events, runs the game loop (`requestAnimationFrame` tick), handles note onset detection, calls `analyze_performance` when playback ends, renders results panel.
- **`state.js`** — Single shared state object (score, playback position, audio context, pitch data, performance tracking arrays).
- **`audio.js`** — Mic capture via Web Audio API. Polls `detect_pitch` WASM function on each animation frame.
- **`score-loader.js`** — Loads MusicXML from files (.xml, .musicxml, .mxl via JSZip) or built-in assets. Shows redirect message for PDF uploads.
- **`constants.js`** — Built-in pieces list, color palette, default config.
- **`renderers/game-view.js`** — Scrolling piano-roll with fixed playhead at 25% from left. Renders pitch trail, accuracy band, pitch indicator dot.
- **`renderers/notation-view.js`** — Traditional staff notation with treble clef, barlines, note heads.
- **`renderers/base.js`** — Shared utils: `midiToName`, `roundRect`, `getCurrentNote`, `sizeCanvas`.

### Data Flow

1. MusicXML → `parse_musicxml()` → `Score` object stored in `state.score`
2. Play pressed → game loop starts, playhead advances by elapsed time / `secondsPerBeat`
3. Mic active → `audio.js` calls `detect_pitch()` each frame → result in `state.currentPitch`
4. `trackPerformance()` in main.js does note onset detection (silence→sound, pitch change) → pushes to `state.playedNotes`
5. Playback ends → `analyze_performance(score, playedNotes, 50.0, 0.3)` → results overlay

### Key Design Decisions

- `lib.rs` is deliberately thin — all logic lives in submodules so it's testable with `cargo test` without WASM.
- `serde-wasm-bindgen` is used instead of `JsValue` manual conversion — Rust structs with `#[derive(Serialize, Deserialize)]` cross the boundary cleanly.
- Crate type is `["cdylib", "rlib"]`: cdylib for WASM output, rlib for `cargo test`.
- Release builds use `opt-level = "s"` (optimize for WASM binary size).
- External dependency: JSZip 3.10.1 loaded from CDN (for .mxl ZIP extraction only).
