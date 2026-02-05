# Trumpet Trainer MVP

This is a minimal Rust + WASM prototype that:
- Parses MusicXML in Rust and returns note timing data
- Draws a static staff with a moving playhead in the browser
- Uses the mic + Rust pitch detection to show your current note
- Includes beginner built-in songs for quick play-along:
  - Happy Birthday (auto-loaded on startup)
  - Hot Cross Buns
  - Ode to Joy

## Prerequisites
- Rust toolchain
- wasm-pack (`cargo install wasm-pack`)

## Build
```bash
wasm-pack build --target web --out-dir web/pkg
```

## Run
```bash
cd web
python3 -m http.server 8080
```
Then open `http://localhost:8080` in your browser.

## Play Along Controls
- On startup, the app automatically loads **Happy Birthday**.
- Use the built-in piece dropdown + **Load Built-in** button to switch songs.
- You can still upload any `.xml` / `.musicxml` file with **Choose MusicXML**.

## MVP limitations
- Single part, single voice, no tuplets
- Basic duration parsing via `<divisions>`
- Pitch detection via simple autocorrelation
# trumpet-rs
