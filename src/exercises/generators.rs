use crate::scoring::types::{MeasureInfo, NoteEvent, Score};

use crate::parser::musicxml::midi_from_pitch;

pub fn generate(exercise_type: &str, key: &str, tempo: f64) -> Result<Score, String> {
    let root_midi = key_to_midi(key)?;

    match exercise_type {
        "long_tones" => Ok(generate_long_tones(root_midi, tempo)),
        "major_scale" => Ok(generate_major_scale(root_midi, tempo)),
        "chromatic" => Ok(generate_chromatic(root_midi, tempo)),
        "lip_slurs" => Ok(generate_lip_slurs(root_midi, tempo)),
        "intervals" => Ok(generate_intervals(root_midi, tempo)),
        "arpeggios" => Ok(generate_arpeggios(root_midi, tempo)),
        _ => Err(format!("Unknown exercise type: {}", exercise_type)),
    }
}

fn key_to_midi(key: &str) -> Result<i32, String> {
    // Parse key like "C4", "F4", "Bb3", etc.
    let key = key.trim();
    if key.is_empty() {
        return Err("Empty key".to_string());
    }

    let mut chars = key.chars();
    let step = chars.next().unwrap();
    let rest: String = chars.collect();

    let (alter, octave_str) = if rest.starts_with('#') {
        (1, &rest[1..])
    } else if rest.starts_with('b') {
        (-1, &rest[1..])
    } else {
        (0, rest.as_str())
    };

    let octave: i32 = if octave_str.is_empty() {
        4 // default octave
    } else {
        octave_str
            .parse()
            .map_err(|_| format!("Invalid octave in key: {}", key))?
    };

    Ok(midi_from_pitch(step, alter, octave))
}

fn build_score(notes: Vec<NoteEvent>, tempo: f64) -> Score {
    let total_beats = notes
        .iter()
        .map(|n| n.start_beat + n.duration_beats)
        .fold(0.0_f64, f64::max);

    // Build measure info (assume 4/4)
    let num_measures = (total_beats / 4.0).ceil() as u32;
    let measures: Vec<MeasureInfo> = (0..num_measures)
        .map(|i| MeasureInfo {
            number: i + 1,
            start_beat: i as f64 * 4.0,
            duration_beats: 4.0,
            time_sig_num: 4,
            time_sig_den: 4,
        })
        .collect();

    Score {
        tempo,
        notes,
        measures,
        key_fifths: 0,
        transpose: None,
        title: None,
        total_beats,
    }
}

fn make_note(start_beat: f64, duration_beats: f64, midi: i32, measure: u32) -> NoteEvent {
    let note_type = match duration_beats as u32 {
        4 => "whole",
        2 => "half",
        1 => "quarter",
        _ => "quarter",
    }
    .to_string();

    NoteEvent {
        start_beat,
        duration_beats,
        midi,
        is_rest: false,
        measure_number: measure,
        note_type,
    }
}

fn make_rest(start_beat: f64, duration_beats: f64, measure: u32) -> NoteEvent {
    NoteEvent {
        start_beat,
        duration_beats,
        midi: -1,
        is_rest: true,
        measure_number: measure,
        note_type: "quarter".to_string(),
    }
}

fn generate_long_tones(root_midi: i32, tempo: f64) -> Score {
    // Play each note for 4 beats (whole note), ascending chromatically
    // from root to root+12, then back down
    let mut notes = Vec::new();
    let mut beat = 0.0;

    for i in 0..=12 {
        let midi = root_midi + i;
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_note(beat, 4.0, midi, measure));
        beat += 4.0;
    }
    // Back down
    for i in (0..12).rev() {
        let midi = root_midi + i;
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_note(beat, 4.0, midi, measure));
        beat += 4.0;
    }

    build_score(notes, tempo)
}

fn generate_major_scale(root_midi: i32, tempo: f64) -> Score {
    let intervals = [0, 2, 4, 5, 7, 9, 11, 12];
    let mut notes = Vec::new();
    let mut beat = 0.0;

    // Up
    for &interval in &intervals {
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_note(beat, 1.0, root_midi + interval, measure));
        beat += 1.0;
    }
    // Down
    for &interval in intervals[..7].iter().rev() {
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_note(beat, 1.0, root_midi + interval, measure));
        beat += 1.0;
    }
    // End on root whole note
    let measure = (beat / 4.0) as u32 + 1;
    notes.push(make_note(beat, 4.0, root_midi, measure));

    build_score(notes, tempo)
}

fn generate_chromatic(root_midi: i32, tempo: f64) -> Score {
    let mut notes = Vec::new();
    let mut beat = 0.0;

    // Up one octave
    for i in 0..=12 {
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_note(beat, 1.0, root_midi + i, measure));
        beat += 1.0;
    }
    // Down
    for i in (0..12).rev() {
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_note(beat, 1.0, root_midi + i, measure));
        beat += 1.0;
    }
    // End on root
    let measure = (beat / 4.0) as u32 + 1;
    notes.push(make_note(beat, 2.0, root_midi, measure));

    build_score(notes, tempo)
}

fn generate_lip_slurs(root_midi: i32, tempo: f64) -> Score {
    // Lip slurs move between harmonics on same fingering
    // Open: C4(60)-G4(67)-C5(72)
    // Approximate patterns relative to root
    let patterns: Vec<Vec<i32>> = vec![
        vec![0, 7, 12, 7],    // root, 5th, octave, 5th
        vec![0, 12, 0, 12],   // root, octave, root, octave
        vec![2, 9, 14, 9],    // up a step
        vec![0, 7, 12, 7, 0], // full pattern
    ];

    let mut notes = Vec::new();
    let mut beat = 0.0;

    for pattern in &patterns {
        for &interval in pattern {
            let measure = (beat / 4.0) as u32 + 1;
            notes.push(make_note(beat, 1.0, root_midi + interval, measure));
            beat += 1.0;
        }
        // Rest between patterns
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_rest(beat, 1.0, measure));
        beat += 1.0;
    }

    build_score(notes, tempo)
}

fn generate_intervals(root_midi: i32, tempo: f64) -> Score {
    // Practice intervals: 3rds, 4ths, 5ths, octaves ascending and descending
    let interval_sizes = [3, 4, 5, 7, 12]; // minor 3rd, major 3rd, 4th, 5th, octave
    let mut notes = Vec::new();
    let mut beat = 0.0;

    for &size in &interval_sizes {
        // Ascending
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_note(beat, 1.0, root_midi, measure));
        beat += 1.0;
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_note(beat, 1.0, root_midi + size, measure));
        beat += 1.0;

        // Descending
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_note(beat, 1.0, root_midi + size, measure));
        beat += 1.0;
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_note(beat, 1.0, root_midi, measure));
        beat += 1.0;
    }

    build_score(notes, tempo)
}

fn generate_arpeggios(root_midi: i32, tempo: f64) -> Score {
    // Major and minor arpeggios
    // Major: root, +4, +7, +12
    // Minor: root, +3, +7, +12
    let patterns: Vec<(&str, Vec<i32>)> = vec![
        ("major", vec![0, 4, 7, 12, 7, 4, 0]),
        ("minor", vec![0, 3, 7, 12, 7, 3, 0]),
    ];

    let mut notes = Vec::new();
    let mut beat = 0.0;

    for (_name, pattern) in &patterns {
        for &interval in pattern {
            let measure = (beat / 4.0) as u32 + 1;
            notes.push(make_note(beat, 1.0, root_midi + interval, measure));
            beat += 1.0;
        }
        // Rest between patterns
        let measure = (beat / 4.0) as u32 + 1;
        notes.push(make_rest(beat, 1.0, measure));
        beat += 1.0;
    }

    build_score(notes, tempo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_midi() {
        assert_eq!(key_to_midi("C4").unwrap(), 60);
        assert_eq!(key_to_midi("A4").unwrap(), 69);
        assert_eq!(key_to_midi("Bb3").unwrap(), 58);
        assert_eq!(key_to_midi("F#4").unwrap(), 66);
        assert_eq!(key_to_midi("C").unwrap(), 60); // default octave 4
    }

    #[test]
    fn test_generate_major_scale() {
        let score = generate("major_scale", "C4", 120.0).unwrap();
        assert!(!score.notes.is_empty());
        // First note should be C4
        assert_eq!(score.notes[0].midi, 60);
        // Should have 16 notes (8 up + 7 down + 1 final root)
        assert_eq!(score.notes.len(), 16);
        // 8th note should be C5 (top of scale)
        assert_eq!(score.notes[7].midi, 72);
    }

    #[test]
    fn test_generate_all_types() {
        for exercise_type in &[
            "long_tones",
            "major_scale",
            "chromatic",
            "lip_slurs",
            "intervals",
            "arpeggios",
        ] {
            let result = generate(exercise_type, "C4", 100.0);
            assert!(result.is_ok(), "Failed to generate {}", exercise_type);
            let score = result.unwrap();
            assert!(!score.notes.is_empty(), "{} has no notes", exercise_type);
            assert!(score.total_beats > 0.0, "{} has no beats", exercise_type);
        }
    }

    #[test]
    fn test_unknown_type() {
        let result = generate("nonexistent", "C4", 120.0);
        assert!(result.is_err());
    }
}
