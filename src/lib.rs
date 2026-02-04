use serde::Serialize;
use wasm_bindgen::prelude::*;

use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Serialize)]
struct NoteEvent {
    start_beat: f32,
    duration_beats: f32,
    midi: i32,
    is_rest: bool,
}

#[derive(Serialize)]
struct Score {
    tempo: f32,
    divisions: f32,
    notes: Vec<NoteEvent>,
}

#[wasm_bindgen]
pub fn parse_musicxml(xml: &str) -> Result<JsValue, JsValue> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut buf = Vec::new();

    let mut divisions: f32 = 1.0;
    let mut tempo: f32 = 120.0;
    let mut notes: Vec<NoteEvent> = Vec::new();

    let mut current_beat: f32 = 0.0;
    let mut last_note_start: f32 = 0.0;
    let mut last_note_duration: f32 = 0.0;

    let mut current_tag: Option<&'static str> = None;

    let mut in_note = false;
    let mut note_is_rest = false;
    let mut note_is_chord = false;
    let mut note_duration_divs: Option<f32> = None;
    let mut step: Option<char> = None;
    let mut alter: i32 = 0;
    let mut octave: Option<i32> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"note" => {
                        in_note = true;
                        note_is_rest = false;
                        note_is_chord = false;
                        note_duration_divs = None;
                        step = None;
                        alter = 0;
                        octave = None;
                    }
                    b"rest" => {
                        if in_note {
                            note_is_rest = true;
                        }
                    }
                    b"chord" => {
                        if in_note {
                            note_is_chord = true;
                        }
                    }
                    b"divisions" => current_tag = Some("divisions"),
                    b"duration" => current_tag = Some("duration"),
                    b"step" => current_tag = Some("step"),
                    b"alter" => current_tag = Some("alter"),
                    b"octave" => current_tag = Some("octave"),
                    b"per-minute" => current_tag = Some("per-minute"),
                    b"sound" => {
                        if let Some(attr) = e.attributes().flatten().find(|a| a.key.as_ref() == b"tempo") {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                if let Ok(t) = val.parse::<f32>() {
                                    tempo = t;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"sound" {
                    if let Some(attr) = e.attributes().flatten().find(|a| a.key.as_ref() == b"tempo") {
                        if let Ok(val) = std::str::from_utf8(&attr.value) {
                            if let Ok(t) = val.parse::<f32>() {
                                tempo = t;
                            }
                        }
                    }
                }
                if name.as_ref() == b"rest" && in_note {
                    note_is_rest = true;
                }
                if name.as_ref() == b"chord" && in_note {
                    note_is_chord = true;
                }
            }
            Ok(Event::Text(e)) => {
                if let Some(tag) = current_tag.take() {
                    let text = e.unescape().map_err(|e| JsValue::from_str(&e.to_string()))?;
                    match tag {
                        "divisions" => {
                            if let Ok(v) = text.parse::<f32>() {
                                if v > 0.0 {
                                    divisions = v;
                                }
                            }
                        }
                        "per-minute" => {
                            if let Ok(v) = text.parse::<f32>() {
                                tempo = v;
                            }
                        }
                        "duration" => {
                            if let Ok(v) = text.parse::<f32>() {
                                note_duration_divs = Some(v);
                            }
                        }
                        "step" => {
                            step = text.chars().next();
                        }
                        "alter" => {
                            if let Ok(v) = text.parse::<i32>() {
                                alter = v;
                            }
                        }
                        "octave" => {
                            if let Ok(v) = text.parse::<i32>() {
                                octave = Some(v);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"note" && in_note {
                    let duration_divs = note_duration_divs.unwrap_or(0.0);
                    let duration_beats = if divisions > 0.0 {
                        duration_divs / divisions
                    } else {
                        0.0
                    };

                    let start_beat = if note_is_chord {
                        last_note_start
                    } else {
                        current_beat
                    };

                    let midi = if note_is_rest {
                        -1
                    } else {
                        let step = step.ok_or_else(|| JsValue::from_str("Missing pitch step"))?;
                        let octave = octave.ok_or_else(|| JsValue::from_str("Missing pitch octave"))?;
                        midi_from_pitch(step, alter, octave)
                    };

                    notes.push(NoteEvent {
                        start_beat,
                        duration_beats,
                        midi,
                        is_rest: note_is_rest,
                    });

                    if !note_is_chord {
                        last_note_start = start_beat;
                        last_note_duration = duration_beats;
                        current_beat += duration_beats;
                    } else if last_note_duration == 0.0 {
                        last_note_duration = duration_beats;
                    }

                    in_note = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(JsValue::from_str(&format!("XML parse error: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    let score = Score {
        tempo,
        divisions,
        notes,
    };

    serde_wasm_bindgen::to_value(&score).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn detect_pitch(samples: &[f32], sample_rate: f32) -> f32 {
    if samples.len() < 2 || sample_rate <= 0.0 {
        return 0.0;
    }

    let mean = samples.iter().sum::<f32>() / samples.len() as f32;
    let mut centered: Vec<f32> = Vec::with_capacity(samples.len());
    let mut energy = 0.0;
    for &s in samples {
        let v = s - mean;
        energy += v * v;
        centered.push(v);
    }

    let rms = (energy / samples.len() as f32).sqrt();
    if rms < 0.01 {
        return 0.0;
    }

    let min_freq = 80.0;
    let max_freq = 1200.0;
    let min_lag = (sample_rate / max_freq) as usize;
    let max_lag = (sample_rate / min_freq) as usize;

    if max_lag >= centered.len() {
        return 0.0;
    }

    let mut best_lag = 0usize;
    let mut best_corr = 0.0;

    for lag in min_lag..=max_lag {
        let mut corr = 0.0;
        let mut i = 0usize;
        while i + lag < centered.len() {
            corr += centered[i] * centered[i + lag];
            i += 1;
        }
        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    if best_lag == 0 {
        return 0.0;
    }

    sample_rate / best_lag as f32
}

fn midi_from_pitch(step: char, alter: i32, octave: i32) -> i32 {
    let base = match step {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => 0,
    };
    (octave + 1) * 12 + base + alter
}
