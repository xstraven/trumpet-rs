use wasm_bindgen::prelude::*;

mod exercises;
mod parser;
mod pitch;
pub mod scoring;
pub mod transposition;

use scoring::types::{PitchTrailPoint, PlayedNote, Score};

use std::cell::RefCell;

thread_local! {
    static DETECTOR: RefCell<Option<pitch::yin::PitchDetector>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn parse_musicxml(xml: &str) -> Result<JsValue, JsValue> {
    let score =
        parser::musicxml::parse_musicxml(xml).map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&score).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// YIN-based pitch detection returning Float64Array [hz, confidence, midi_float].
/// Uses a thread-local pre-allocated PitchDetector to avoid per-call allocations.
#[wasm_bindgen]
pub fn detect_pitch(samples: &[f32], sample_rate: f32) -> js_sys::Float64Array {
    let result = DETECTOR.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let detector = borrow.get_or_insert_with(|| {
            pitch::yin::PitchDetector::new(sample_rate, 80.0, 1200.0, 2048)
        });
        detector.detect(samples)
    });

    let arr = js_sys::Float64Array::new_with_length(3);
    arr.set_index(0, result.hz as f64);
    arr.set_index(1, result.confidence as f64);
    arr.set_index(2, result.midi_float as f64);
    arr
}

/// Analyze a performance: compare played notes against score.
#[wasm_bindgen]
pub fn analyze_performance(
    score_js: JsValue,
    played_notes_js: JsValue,
    tolerance_cents: f64,
    timing_tolerance_beats: f64,
    pitch_trail_js: JsValue,
) -> Result<JsValue, JsValue> {
    let score: Score =
        serde_wasm_bindgen::from_value(score_js).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let played_notes: Vec<PlayedNote> = serde_wasm_bindgen::from_value(played_notes_js)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let pitch_trail: Option<Vec<PitchTrailPoint>> = if pitch_trail_js.is_null() || pitch_trail_js.is_undefined() {
        None
    } else {
        Some(
            serde_wasm_bindgen::from_value(pitch_trail_js)
                .map_err(|e| JsValue::from_str(&e.to_string()))?,
        )
    };

    let analysis = match &pitch_trail {
        Some(trail) => scoring::analyzer::analyze_performance_with_trail(
            &score,
            &played_notes,
            tolerance_cents,
            timing_tolerance_beats,
            Some(trail),
        ),
        None => scoring::analyzer::analyze_performance(
            &score,
            &played_notes,
            tolerance_cents,
            timing_tolerance_beats,
        ),
    };

    serde_wasm_bindgen::to_value(&analysis).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Generate a warmup exercise, returning a Score.
#[wasm_bindgen]
pub fn generate_exercise(
    exercise_type: &str,
    key: &str,
    tempo: f64,
    difficulty: Option<u8>,
    midi_low: Option<i32>,
    midi_high: Option<i32>,
) -> Result<JsValue, JsValue> {
    let midi_range = match (midi_low, midi_high) {
        (Some(low), Some(high)) => Some((low, high)),
        _ => None,
    };
    let score =
        exercises::generators::generate_with_options(exercise_type, key, tempo, difficulty, midi_range)
            .map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&score).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get the 4-stage curriculum structure.
#[wasm_bindgen]
pub fn get_curriculum() -> Result<JsValue, JsValue> {
    let curriculum = exercises::curriculum::get_curriculum();
    serde_wasm_bindgen::to_value(&curriculum).map_err(|e| JsValue::from_str(&e.to_string()))
}
