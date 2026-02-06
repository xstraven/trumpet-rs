use wasm_bindgen::prelude::*;

mod exercises;
mod parser;
mod pitch;
pub mod scoring;
pub mod transposition;

use scoring::types::{PlayedNote, Score};

#[wasm_bindgen]
pub fn parse_musicxml(xml: &str) -> Result<JsValue, JsValue> {
    let score =
        parser::musicxml::parse_musicxml(xml).map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&score).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// YIN-based pitch detection returning {hz, confidence, midi_float}.
#[wasm_bindgen]
pub fn detect_pitch(samples: &[f32], sample_rate: f32) -> JsValue {
    let result = pitch::yin::detect_pitch_yin(samples, sample_rate);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Analyze a performance: compare played notes against score.
#[wasm_bindgen]
pub fn analyze_performance(
    score_js: JsValue,
    played_notes_js: JsValue,
    tolerance_cents: f64,
    timing_tolerance_beats: f64,
) -> Result<JsValue, JsValue> {
    let score: Score =
        serde_wasm_bindgen::from_value(score_js).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let played_notes: Vec<PlayedNote> = serde_wasm_bindgen::from_value(played_notes_js)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let analysis = scoring::analyzer::analyze_performance(
        &score,
        &played_notes,
        tolerance_cents,
        timing_tolerance_beats,
    );

    serde_wasm_bindgen::to_value(&analysis).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Generate a warmup exercise, returning a Score.
#[wasm_bindgen]
pub fn generate_exercise(
    exercise_type: &str,
    key: &str,
    tempo: f64,
) -> Result<JsValue, JsValue> {
    let score = exercises::generators::generate(exercise_type, key, tempo)
        .map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&score).map_err(|e| JsValue::from_str(&e.to_string()))
}
