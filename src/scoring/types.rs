use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NoteEvent {
    pub start_beat: f64,
    pub duration_beats: f64,
    pub midi: i32,
    pub is_rest: bool,
    pub measure_number: u32,
    pub note_type: String,
}

// Performance tracking types

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayedNote {
    pub onset_beat: f64,
    pub midi_float: f64,
    pub midi_rounded: i32,
    pub confidence: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NoteResult {
    pub target_midi: i32,
    pub target_beat: f64,
    pub status: String, // "correct", "wrong_pitch", "missed"
    pub played_midi: Option<f64>,
    pub pitch_error_cents: Option<f64>,
    pub timing_error_beats: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PitchTrailPoint {
    pub beat: f64,
    pub midi_float: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IntervalProblem {
    pub from_note: String,
    pub to_note: String,
    pub direction: String, // "up" or "down"
    pub avg_error_cents: f64,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PerformanceAnalysis {
    pub total_notes: u32,
    pub notes_correct: u32,
    pub notes_wrong_pitch: u32,
    pub notes_missed: u32,
    pub avg_pitch_error_cents: f64,
    pub avg_timing_error_beats: f64,
    pub pitch_tendency: String,  // "sharp", "flat", "accurate"
    pub timing_tendency: String, // "early", "late", "on_time"
    pub problem_intervals: Vec<IntervalProblem>,
    pub feedback: Vec<String>,
    pub overall_score: f64, // 0-100
    pub note_results: Vec<NoteResult>,
    // Technique analysis (populated when pitch_trail is provided)
    pub pitch_stability: Option<f64>,  // std dev of pitch in cents within held notes
    pub attack_quality: Option<f64>,   // 0-1 score, how quickly pitch stabilizes
    pub breath_support: Option<f64>,   // 0-1 score, pitch sustain consistency
    pub endurance_delta: Option<f64>,  // accuracy drop: first half vs second half
    pub technique_feedback: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MeasureInfo {
    pub number: u32,
    pub start_beat: f64,
    pub duration_beats: f64,
    pub time_sig_num: u8,
    pub time_sig_den: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransposeInfo {
    pub chromatic: i32,
    pub diatonic: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Score {
    pub tempo: f64,
    pub notes: Vec<NoteEvent>,
    pub measures: Vec<MeasureInfo>,
    pub key_fifths: i32,
    pub transpose: Option<TransposeInfo>,
    pub title: Option<String>,
    pub total_beats: f64,
}
