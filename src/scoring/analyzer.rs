use crate::scoring::types::*;

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

fn midi_to_name(midi: i32) -> String {
    let name = NOTE_NAMES[(midi.rem_euclid(12)) as usize];
    let octave = midi / 12 - 1;
    format!("{}{}", name, octave)
}

fn cents_between(played_midi: f64, target_midi: i32) -> f64 {
    (played_midi - target_midi as f64) * 100.0
}

pub fn analyze_performance(
    score: &Score,
    played_notes: &[PlayedNote],
    tolerance_cents: f64,
    timing_tolerance_beats: f64,
) -> PerformanceAnalysis {
    analyze_performance_with_trail(score, played_notes, tolerance_cents, timing_tolerance_beats, None)
}

pub fn analyze_performance_with_trail(
    score: &Score,
    played_notes: &[PlayedNote],
    tolerance_cents: f64,
    timing_tolerance_beats: f64,
    pitch_trail: Option<&[PitchTrailPoint]>,
) -> PerformanceAnalysis {
    let target_notes: Vec<&NoteEvent> = score.notes.iter().filter(|n| !n.is_rest).collect();
    let total_notes = target_notes.len() as u32;

    if total_notes == 0 {
        return PerformanceAnalysis {
            total_notes: 0,
            notes_correct: 0,
            notes_wrong_pitch: 0,
            notes_missed: 0,
            avg_pitch_error_cents: 0.0,
            avg_timing_error_beats: 0.0,
            pitch_tendency: "accurate".to_string(),
            timing_tendency: "on_time".to_string(),
            problem_intervals: Vec::new(),
            feedback: vec!["No notes in score to analyze.".to_string()],
            overall_score: 0.0,
            note_results: Vec::new(),
            pitch_stability: None,
            attack_quality: None,
            breath_support: None,
            endurance_delta: None,
            technique_feedback: Vec::new(),
        };
    }

    let mut note_results: Vec<NoteResult> = Vec::new();
    let mut pitch_errors: Vec<f64> = Vec::new();
    let mut timing_errors: Vec<f64> = Vec::new();
    let mut used_played: Vec<bool> = vec![false; played_notes.len()];

    // For each target note, find the best matching played note
    for target in &target_notes {
        let mut best_idx: Option<usize> = None;
        let mut best_timing_dist = f64::MAX;

        for (i, played) in played_notes.iter().enumerate() {
            if used_played[i] {
                continue;
            }
            let timing_dist = (played.onset_beat - target.start_beat).abs();
            if timing_dist <= timing_tolerance_beats && timing_dist < best_timing_dist {
                best_timing_dist = timing_dist;
                best_idx = Some(i);
            }
        }

        match best_idx {
            Some(idx) => {
                used_played[idx] = true;
                let played = &played_notes[idx];
                let cent_error = cents_between(played.midi_float, target.midi);
                let timing_error = played.onset_beat - target.start_beat;

                if cent_error.abs() <= tolerance_cents {
                    note_results.push(NoteResult {
                        target_midi: target.midi,
                        target_beat: target.start_beat,
                        status: "correct".to_string(),
                        played_midi: Some(played.midi_float),
                        pitch_error_cents: Some(cent_error),
                        timing_error_beats: Some(timing_error),
                    });
                    pitch_errors.push(cent_error);
                    timing_errors.push(timing_error);
                } else {
                    note_results.push(NoteResult {
                        target_midi: target.midi,
                        target_beat: target.start_beat,
                        status: "wrong_pitch".to_string(),
                        played_midi: Some(played.midi_float),
                        pitch_error_cents: Some(cent_error),
                        timing_error_beats: Some(timing_error),
                    });
                    pitch_errors.push(cent_error);
                    timing_errors.push(timing_error);
                }
            }
            None => {
                note_results.push(NoteResult {
                    target_midi: target.midi,
                    target_beat: target.start_beat,
                    status: "missed".to_string(),
                    played_midi: None,
                    pitch_error_cents: None,
                    timing_error_beats: None,
                });
            }
        }
    }

    let notes_correct = note_results.iter().filter(|r| r.status == "correct").count() as u32;
    let notes_wrong_pitch = note_results
        .iter()
        .filter(|r| r.status == "wrong_pitch")
        .count() as u32;
    let notes_missed = note_results.iter().filter(|r| r.status == "missed").count() as u32;

    let avg_pitch_error_cents = if !pitch_errors.is_empty() {
        pitch_errors.iter().sum::<f64>() / pitch_errors.len() as f64
    } else {
        0.0
    };

    let avg_timing_error_beats = if !timing_errors.is_empty() {
        timing_errors.iter().sum::<f64>() / timing_errors.len() as f64
    } else {
        0.0
    };

    let pitch_tendency = if avg_pitch_error_cents > 10.0 {
        "sharp"
    } else if avg_pitch_error_cents < -10.0 {
        "flat"
    } else {
        "accurate"
    }
    .to_string();

    let timing_tendency = if avg_timing_error_beats > 0.1 {
        "late"
    } else if avg_timing_error_beats < -0.1 {
        "early"
    } else {
        "on_time"
    }
    .to_string();

    // Analyze interval problems
    let problem_intervals = analyze_intervals(&target_notes, &note_results, tolerance_cents);

    // Generate feedback messages
    let mut feedback: Vec<String> = Vec::new();

    if total_notes > 0 {
        let pct = (notes_correct as f64 / total_notes as f64) * 100.0;
        if pct >= 90.0 {
            feedback.push(format!("Excellent! You nailed {:.0}% of the notes.", pct));
        } else if pct >= 70.0 {
            feedback.push(format!("Good job! You got {:.0}% of the notes right.", pct));
        } else if pct >= 50.0 {
            feedback.push(format!(
                "Keep practicing! You hit {:.0}% of the notes correctly.",
                pct
            ));
        } else {
            feedback.push(format!(
                "This one's tough! You got {:.0}% correct. Try slowing down the tempo.",
                pct
            ));
        }
    }

    if notes_missed > 0 {
        feedback.push(format!(
            "You missed {} note{}. Make sure to play through the whole piece.",
            notes_missed,
            if notes_missed == 1 { "" } else { "s" }
        ));
    }

    if !pitch_errors.is_empty() {
        let abs_avg = pitch_errors.iter().map(|e| e.abs()).sum::<f64>() / pitch_errors.len() as f64;
        if abs_avg > 30.0 {
            if avg_pitch_error_cents > 10.0 {
                feedback.push(format!(
                    "Your pitch is consistently {:.0} cents sharp. Try relaxing your embouchure slightly.",
                    avg_pitch_error_cents
                ));
            } else if avg_pitch_error_cents < -10.0 {
                feedback.push(format!(
                    "Your pitch is consistently {:.0} cents flat. Try firming up your embouchure and using more air support.",
                    avg_pitch_error_cents.abs()
                ));
            }
        }
    }

    if !timing_errors.is_empty() {
        let abs_avg =
            timing_errors.iter().map(|e| e.abs()).sum::<f64>() / timing_errors.len() as f64;
        if abs_avg > 0.15 {
            if avg_timing_error_beats > 0.1 {
                feedback.push(
                    "You tend to come in late. Try anticipating the beat and starting your air a bit earlier.".to_string(),
                );
            } else if avg_timing_error_beats < -0.1 {
                feedback.push(
                    "You tend to rush ahead. Try listening to the beat and holding back slightly."
                        .to_string(),
                );
            }
        }
    }

    for problem in &problem_intervals {
        let dir_word = if problem.direction == "up" {
            "ascending"
        } else {
            "descending"
        };
        if problem.avg_error_cents > 0.0 {
            feedback.push(format!(
                "You overshoot when going {} from {} to {} (avg +{:.0} cents). Try less pressure on the jump.",
                dir_word, problem.from_note, problem.to_note, problem.avg_error_cents
            ));
        } else {
            feedback.push(format!(
                "You undershoot when going {} from {} to {} (avg {:.0} cents). Use more air support on the jump.",
                dir_word, problem.from_note, problem.to_note, problem.avg_error_cents
            ));
        }
    }

    if feedback.is_empty() {
        feedback.push("Play with the mic active to get feedback!".to_string());
    }

    // Overall score: weighted combination of pitch accuracy and note hit rate
    let hit_rate = if total_notes > 0 {
        (notes_correct + notes_wrong_pitch) as f64 / total_notes as f64
    } else {
        0.0
    };
    let pitch_score = if !pitch_errors.is_empty() {
        let abs_avg = pitch_errors.iter().map(|e| e.abs()).sum::<f64>() / pitch_errors.len() as f64;
        (1.0 - (abs_avg / 100.0).min(1.0)) * 100.0
    } else {
        0.0
    };
    let correct_rate = if total_notes > 0 {
        notes_correct as f64 / total_notes as f64
    } else {
        0.0
    };
    let overall_score = (correct_rate * 60.0 + hit_rate * 20.0 + pitch_score * 0.2).min(100.0);

    // Technique analysis
    let (pitch_stability, attack_quality, breath_support, endurance_delta, technique_feedback) =
        if let Some(trail) = pitch_trail {
            analyze_technique(&target_notes, &note_results, trail)
        } else {
            (None, None, None, None, Vec::new())
        };

    PerformanceAnalysis {
        total_notes,
        notes_correct,
        notes_wrong_pitch,
        notes_missed,
        avg_pitch_error_cents,
        avg_timing_error_beats,
        pitch_tendency,
        timing_tendency,
        problem_intervals,
        feedback,
        overall_score,
        note_results,
        pitch_stability,
        attack_quality,
        breath_support,
        endurance_delta,
        technique_feedback,
    }
}

fn analyze_technique(
    target_notes: &[&NoteEvent],
    note_results: &[NoteResult],
    pitch_trail: &[PitchTrailPoint],
) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>, Vec<String>) {
    if pitch_trail.is_empty() || target_notes.is_empty() {
        return (None, None, None, None, Vec::new());
    }

    let mut stability_values: Vec<f64> = Vec::new();
    let mut attack_times: Vec<f64> = Vec::new();
    let mut sustain_drifts: Vec<f64> = Vec::new();
    let mut technique_feedback = Vec::new();

    for target in target_notes {
        let note_end = target.start_beat + target.duration_beats;
        let trail_points: Vec<&PitchTrailPoint> = pitch_trail
            .iter()
            .filter(|p| p.beat >= target.start_beat && p.beat < note_end)
            .collect();

        if trail_points.len() < 3 {
            continue;
        }

        let target_midi = target.midi as f64;

        // Pitch stability: std dev of cents within held notes
        let cents: Vec<f64> = trail_points
            .iter()
            .map(|p| (p.midi_float - target_midi) * 100.0)
            .collect();
        let mean_cents = cents.iter().sum::<f64>() / cents.len() as f64;
        let variance = cents.iter().map(|c| (c - mean_cents).powi(2)).sum::<f64>() / cents.len() as f64;
        stability_values.push(variance.sqrt());

        // Attack quality: how many trail points until within 20 cents of target
        let mut attack_count = 0;
        for c in &cents {
            if c.abs() <= 20.0 {
                break;
            }
            attack_count += 1;
        }
        let attack_ratio = attack_count as f64 / trail_points.len() as f64;
        attack_times.push(attack_ratio);

        // Breath support: for notes >= 2 beats, compare first half avg vs second half avg
        if target.duration_beats >= 2.0 {
            let mid = trail_points.len() / 2;
            if mid > 0 {
                let first_avg: f64 =
                    trail_points[..mid].iter().map(|p| p.midi_float).sum::<f64>() / mid as f64;
                let second_avg: f64 = trail_points[mid..]
                    .iter()
                    .map(|p| p.midi_float)
                    .sum::<f64>()
                    / (trail_points.len() - mid) as f64;
                let drift_cents = (second_avg - first_avg).abs() * 100.0;
                sustain_drifts.push(drift_cents);
            }
        }
    }

    // Aggregate pitch stability
    let pitch_stability = if !stability_values.is_empty() {
        Some(stability_values.iter().sum::<f64>() / stability_values.len() as f64)
    } else {
        None
    };

    // Aggregate attack quality (0 = instant, 1 = never stabilizes)
    let attack_quality = if !attack_times.is_empty() {
        let avg_attack = attack_times.iter().sum::<f64>() / attack_times.len() as f64;
        Some((1.0 - avg_attack).max(0.0))
    } else {
        None
    };

    // Aggregate breath support (lower drift = better)
    let breath_support = if !sustain_drifts.is_empty() {
        let avg_drift = sustain_drifts.iter().sum::<f64>() / sustain_drifts.len() as f64;
        Some((1.0 - (avg_drift / 50.0).min(1.0)).max(0.0))
    } else {
        None
    };

    // Endurance delta: compare accuracy in first half vs second half of note_results
    let endurance_delta = if note_results.len() >= 4 {
        let mid = note_results.len() / 2;
        let first_correct = note_results[..mid]
            .iter()
            .filter(|r| r.status == "correct")
            .count() as f64
            / mid as f64;
        let second_correct = note_results[mid..]
            .iter()
            .filter(|r| r.status == "correct")
            .count() as f64
            / (note_results.len() - mid) as f64;
        Some((first_correct - second_correct) * 100.0)
    } else {
        None
    };

    // Generate technique feedback
    if let Some(stability) = pitch_stability {
        if stability > 15.0 {
            technique_feedback.push(
                "Your pitch wobbles on sustained notes. Focus on steady airflow.".to_string(),
            );
        }
    }
    if let Some(attack) = attack_quality {
        if attack < 0.7 {
            technique_feedback.push(
                "Your note attacks are slow to center. Try a firmer tongue stroke.".to_string(),
            );
        }
    }
    if let Some(breath) = breath_support {
        if breath < 0.7 {
            technique_feedback.push(
                "Your pitch drops through long notes. Practice deep breathing.".to_string(),
            );
        }
    }
    if let Some(delta) = endurance_delta {
        if delta > 15.0 {
            technique_feedback.push(
                "Your accuracy drops later in the piece. Build endurance with long tones."
                    .to_string(),
            );
        }
    }

    (
        pitch_stability,
        attack_quality,
        breath_support,
        endurance_delta,
        technique_feedback,
    )
}

fn analyze_intervals(
    _target_notes: &[&NoteEvent],
    results: &[NoteResult],
    tolerance_cents: f64,
) -> Vec<IntervalProblem> {
    use std::collections::HashMap;

    // Track errors per interval (from_midi, to_midi)
    let mut interval_errors: HashMap<(i32, i32), Vec<f64>> = HashMap::new();

    for i in 1..results.len() {
        let prev = &results[i - 1];
        let curr = &results[i];

        // Only analyze intervals where both notes were played
        if let (Some(_prev_cents), Some(curr_cents)) =
            (prev.pitch_error_cents, curr.pitch_error_cents)
        {
            if curr_cents.abs() > tolerance_cents * 0.5 {
                let key = (prev.target_midi, curr.target_midi);
                interval_errors.entry(key).or_default().push(curr_cents);
            }
        }
    }

    let mut problems: Vec<IntervalProblem> = Vec::new();
    for ((from_midi, to_midi), errors) in &interval_errors {
        if errors.len() < 2 {
            continue; // Need at least 2 occurrences to call it a pattern
        }
        let avg = errors.iter().sum::<f64>() / errors.len() as f64;
        if avg.abs() > 20.0 {
            let direction = if to_midi > from_midi { "up" } else { "down" };
            problems.push(IntervalProblem {
                from_note: midi_to_name(*from_midi),
                to_note: midi_to_name(*to_midi),
                direction: direction.to_string(),
                avg_error_cents: avg,
                count: errors.len() as u32,
            });
        }
    }

    // Sort by severity
    problems.sort_by(|a, b| {
        b.avg_error_cents
            .abs()
            .partial_cmp(&a.avg_error_cents.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    problems.truncate(3); // Top 3 problem intervals
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_score(notes: Vec<(f64, f64, i32)>) -> Score {
        Score {
            tempo: 120.0,
            notes: notes
                .into_iter()
                .map(|(beat, dur, midi)| NoteEvent {
                    start_beat: beat,
                    duration_beats: dur,
                    midi,
                    is_rest: false,
                    measure_number: 1,
                    note_type: "quarter".to_string(),
                })
                .collect(),
            measures: vec![],
            key_fifths: 0,
            transpose: None,
            title: None,
            total_beats: 4.0,
        }
    }

    #[test]
    fn test_perfect_performance() {
        let score = make_score(vec![(0.0, 1.0, 60), (1.0, 1.0, 62), (2.0, 1.0, 64)]);
        let played = vec![
            PlayedNote {
                onset_beat: 0.0,
                midi_float: 60.0,
                midi_rounded: 60,
                confidence: 0.9,
            },
            PlayedNote {
                onset_beat: 1.0,
                midi_float: 62.0,
                midi_rounded: 62,
                confidence: 0.9,
            },
            PlayedNote {
                onset_beat: 2.0,
                midi_float: 64.0,
                midi_rounded: 64,
                confidence: 0.9,
            },
        ];

        let result = analyze_performance(&score, &played, 50.0, 0.25);
        assert_eq!(result.total_notes, 3);
        assert_eq!(result.notes_correct, 3);
        assert_eq!(result.notes_missed, 0);
        assert_eq!(result.notes_wrong_pitch, 0);
        assert!(result.overall_score > 70.0);
        assert_eq!(result.pitch_tendency, "accurate");
        assert_eq!(result.timing_tendency, "on_time");
    }

    #[test]
    fn test_missed_notes() {
        let score = make_score(vec![(0.0, 1.0, 60), (1.0, 1.0, 62), (2.0, 1.0, 64)]);
        let played = vec![PlayedNote {
            onset_beat: 0.0,
            midi_float: 60.0,
            midi_rounded: 60,
            confidence: 0.9,
        }];

        let result = analyze_performance(&score, &played, 50.0, 0.25);
        assert_eq!(result.notes_correct, 1);
        assert_eq!(result.notes_missed, 2);
    }

    #[test]
    fn test_sharp_tendency() {
        let score = make_score(vec![(0.0, 1.0, 60), (1.0, 1.0, 62)]);
        let played = vec![
            PlayedNote {
                onset_beat: 0.0,
                midi_float: 60.2,
                midi_rounded: 60,
                confidence: 0.9,
            },
            PlayedNote {
                onset_beat: 1.0,
                midi_float: 62.3,
                midi_rounded: 62,
                confidence: 0.9,
            },
        ];

        let result = analyze_performance(&score, &played, 50.0, 0.25);
        assert_eq!(result.notes_correct, 2);
        assert_eq!(result.pitch_tendency, "sharp");
    }

    #[test]
    fn test_wrong_pitch() {
        let score = make_score(vec![(0.0, 1.0, 60)]);
        let played = vec![PlayedNote {
            onset_beat: 0.0,
            midi_float: 62.0,            // 200 cents off
            midi_rounded: 62,
            confidence: 0.9,
        }];

        let result = analyze_performance(&score, &played, 50.0, 0.25);
        assert_eq!(result.notes_wrong_pitch, 1);
        assert_eq!(result.notes_correct, 0);
    }

    #[test]
    fn test_empty_score() {
        let score = Score {
            tempo: 120.0,
            notes: vec![],
            measures: vec![],
            key_fifths: 0,
            transpose: None,
            title: None,
            total_beats: 0.0,
        };
        let result = analyze_performance(&score, &[], 50.0, 0.25);
        assert_eq!(result.total_notes, 0);
    }

    #[test]
    fn test_technique_analysis_with_trail() {
        let score = make_score(vec![(0.0, 4.0, 60), (4.0, 4.0, 62)]);
        let played = vec![
            PlayedNote {
                onset_beat: 0.0,
                midi_float: 60.0,
                midi_rounded: 60,
                confidence: 0.9,
            },
            PlayedNote {
                onset_beat: 4.0,
                midi_float: 62.0,
                midi_rounded: 62,
                confidence: 0.9,
            },
        ];
        // Simulate a stable pitch trail for the first note, wobbling on second
        let mut trail = Vec::new();
        for i in 0..20 {
            trail.push(PitchTrailPoint {
                beat: i as f64 * 0.2,
                midi_float: 60.0 + 0.01, // very stable
            });
        }
        for i in 0..20 {
            let wobble = if i % 2 == 0 { 0.3 } else { -0.3 };
            trail.push(PitchTrailPoint {
                beat: 4.0 + i as f64 * 0.2,
                midi_float: 62.0 + wobble, // wobbling
            });
        }

        let result =
            analyze_performance_with_trail(&score, &played, 50.0, 0.5, Some(&trail));
        assert_eq!(result.notes_correct, 2);
        assert!(result.pitch_stability.is_some());
        assert!(result.attack_quality.is_some());
        assert!(result.breath_support.is_some());
    }

    #[test]
    fn test_endurance_delta() {
        // 8 notes, first 4 perfect, last 4 missed
        let score = make_score(vec![
            (0.0, 1.0, 60),
            (1.0, 1.0, 62),
            (2.0, 1.0, 64),
            (3.0, 1.0, 65),
            (4.0, 1.0, 67),
            (5.0, 1.0, 69),
            (6.0, 1.0, 71),
            (7.0, 1.0, 72),
        ]);
        let played = vec![
            PlayedNote { onset_beat: 0.0, midi_float: 60.0, midi_rounded: 60, confidence: 0.9 },
            PlayedNote { onset_beat: 1.0, midi_float: 62.0, midi_rounded: 62, confidence: 0.9 },
            PlayedNote { onset_beat: 2.0, midi_float: 64.0, midi_rounded: 64, confidence: 0.9 },
            PlayedNote { onset_beat: 3.0, midi_float: 65.0, midi_rounded: 65, confidence: 0.9 },
            // last 4 missed
        ];
        let trail: Vec<PitchTrailPoint> = (0..40)
            .map(|i| PitchTrailPoint { beat: i as f64 * 0.2, midi_float: 60.0 })
            .collect();
        let result = analyze_performance_with_trail(&score, &played, 50.0, 0.5, Some(&trail));
        // First half: 4/4 correct, second half: 0/4 correct => delta = 100
        assert!(result.endurance_delta.is_some());
        let delta = result.endurance_delta.unwrap();
        assert!(delta > 50.0, "Expected large endurance delta, got {}", delta);
    }
}
