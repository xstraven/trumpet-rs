use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct CurriculumExercise {
    pub exercise_type: String,
    pub name: String,
    pub description: String,
    pub difficulty: u8,
    pub keys: Vec<String>,
    pub tempo_range: [f64; 2],
    pub midi_range: [i32; 2],
}

#[derive(Serialize, Clone, Debug)]
pub struct CurriculumStage {
    pub stage: u8,
    pub name: String,
    pub description: String,
    pub exercises: Vec<CurriculumExercise>,
}

pub fn get_curriculum() -> Vec<CurriculumStage> {
    vec![
        CurriculumStage {
            stage: 1,
            name: "Beginner".to_string(),
            description: "Build fundamentals: tone production and simple melodies (C4-G4)"
                .to_string(),
            exercises: vec![
                CurriculumExercise {
                    exercise_type: "long_tones".to_string(),
                    name: "Long Tones".to_string(),
                    description: "Sustain each note with steady tone".to_string(),
                    difficulty: 1,
                    keys: vec!["C4".to_string()],
                    tempo_range: [60.0, 80.0],
                    midi_range: [60, 67], // C4-G4
                },
                CurriculumExercise {
                    exercise_type: "major_scale".to_string(),
                    name: "C Major Scale".to_string(),
                    description: "Play the C major scale slowly and evenly".to_string(),
                    difficulty: 1,
                    keys: vec!["C4".to_string()],
                    tempo_range: [60.0, 80.0],
                    midi_range: [60, 67],
                },
            ],
        },
        CurriculumStage {
            stage: 2,
            name: "Early Beginner".to_string(),
            description: "Expand range and flexibility (C4-C5)".to_string(),
            exercises: vec![
                CurriculumExercise {
                    exercise_type: "major_scale".to_string(),
                    name: "Scales in C, F, G".to_string(),
                    description: "Practice major scales in three keys".to_string(),
                    difficulty: 2,
                    keys: vec!["C4".to_string(), "F4".to_string(), "G4".to_string()],
                    tempo_range: [70.0, 90.0],
                    midi_range: [60, 72], // C4-C5
                },
                CurriculumExercise {
                    exercise_type: "lip_slurs".to_string(),
                    name: "Simple Lip Slurs".to_string(),
                    description: "Smooth transitions between harmonics".to_string(),
                    difficulty: 2,
                    keys: vec!["C4".to_string()],
                    tempo_range: [70.0, 90.0],
                    midi_range: [60, 72],
                },
                CurriculumExercise {
                    exercise_type: "chromatic".to_string(),
                    name: "Chromatic Scale".to_string(),
                    description: "Half steps through one octave".to_string(),
                    difficulty: 2,
                    keys: vec!["C4".to_string()],
                    tempo_range: [70.0, 90.0],
                    midi_range: [60, 72],
                },
                CurriculumExercise {
                    exercise_type: "long_tones".to_string(),
                    name: "Extended Long Tones".to_string(),
                    description: "Sustain notes across the full octave".to_string(),
                    difficulty: 2,
                    keys: vec!["C4".to_string()],
                    tempo_range: [60.0, 80.0],
                    midi_range: [60, 72],
                },
            ],
        },
        CurriculumStage {
            stage: 3,
            name: "Intermediate".to_string(),
            description: "All keys, intervals, and arpeggios (C4-G5)".to_string(),
            exercises: vec![
                CurriculumExercise {
                    exercise_type: "major_scale".to_string(),
                    name: "Scales in All Keys".to_string(),
                    description: "Major scales in all 12 keys".to_string(),
                    difficulty: 3,
                    keys: vec![
                        "C4".to_string(),
                        "Db4".to_string(),
                        "D4".to_string(),
                        "Eb4".to_string(),
                        "E4".to_string(),
                        "F4".to_string(),
                        "F#4".to_string(),
                        "G4".to_string(),
                        "Ab4".to_string(),
                        "A4".to_string(),
                        "Bb4".to_string(),
                        "B4".to_string(),
                    ],
                    tempo_range: [80.0, 120.0],
                    midi_range: [60, 79], // C4-G5
                },
                CurriculumExercise {
                    exercise_type: "intervals".to_string(),
                    name: "Interval Training".to_string(),
                    description: "Practice 3rds, 4ths, 5ths, and octaves".to_string(),
                    difficulty: 3,
                    keys: vec!["C4".to_string(), "F4".to_string(), "G4".to_string()],
                    tempo_range: [80.0, 120.0],
                    midi_range: [60, 79],
                },
                CurriculumExercise {
                    exercise_type: "arpeggios".to_string(),
                    name: "Arpeggios".to_string(),
                    description: "Major and minor arpeggios".to_string(),
                    difficulty: 3,
                    keys: vec!["C4".to_string(), "F4".to_string(), "G4".to_string()],
                    tempo_range: [80.0, 120.0],
                    midi_range: [60, 79],
                },
                CurriculumExercise {
                    exercise_type: "lip_slurs".to_string(),
                    name: "Advanced Lip Slurs".to_string(),
                    description: "Extended harmonic patterns".to_string(),
                    difficulty: 3,
                    keys: vec!["C4".to_string(), "F4".to_string()],
                    tempo_range: [80.0, 110.0],
                    midi_range: [60, 79],
                },
                CurriculumExercise {
                    exercise_type: "broken_thirds".to_string(),
                    name: "Broken Thirds".to_string(),
                    description: "Scale in thirds: C-E, D-F, E-G...".to_string(),
                    difficulty: 3,
                    keys: vec!["C4".to_string(), "F4".to_string(), "G4".to_string()],
                    tempo_range: [80.0, 110.0],
                    midi_range: [60, 79],
                },
            ],
        },
        CurriculumStage {
            stage: 4,
            name: "Advanced".to_string(),
            description: "Full range, complex patterns, speed (C4-C6)".to_string(),
            exercises: vec![
                CurriculumExercise {
                    exercise_type: "tonguing".to_string(),
                    name: "Tonguing Patterns".to_string(),
                    description: "Repeated notes with varying rhythms for articulation"
                        .to_string(),
                    difficulty: 4,
                    keys: vec!["C4".to_string(), "G4".to_string(), "C5".to_string()],
                    tempo_range: [100.0, 160.0],
                    midi_range: [60, 84], // C4-C6
                },
                CurriculumExercise {
                    exercise_type: "octave_studies".to_string(),
                    name: "Octave Studies".to_string(),
                    description: "Octave jumps on the same pitch class".to_string(),
                    difficulty: 4,
                    keys: vec!["C4".to_string(), "F4".to_string(), "G4".to_string()],
                    tempo_range: [100.0, 140.0],
                    midi_range: [60, 84],
                },
                CurriculumExercise {
                    exercise_type: "broken_thirds".to_string(),
                    name: "Fast Broken Thirds".to_string(),
                    description: "Broken thirds at speed across full range".to_string(),
                    difficulty: 4,
                    keys: vec![
                        "C4".to_string(),
                        "D4".to_string(),
                        "Eb4".to_string(),
                        "F4".to_string(),
                        "G4".to_string(),
                        "A4".to_string(),
                        "Bb4".to_string(),
                    ],
                    tempo_range: [110.0, 160.0],
                    midi_range: [60, 84],
                },
                CurriculumExercise {
                    exercise_type: "chromatic".to_string(),
                    name: "Extended Chromatic".to_string(),
                    description: "Chromatic runs across two octaves".to_string(),
                    difficulty: 4,
                    keys: vec!["C4".to_string()],
                    tempo_range: [100.0, 150.0],
                    midi_range: [60, 84],
                },
                CurriculumExercise {
                    exercise_type: "arpeggios".to_string(),
                    name: "Extended Arpeggios".to_string(),
                    description: "Arpeggios across full range in all keys".to_string(),
                    difficulty: 4,
                    keys: vec![
                        "C4".to_string(),
                        "D4".to_string(),
                        "Eb4".to_string(),
                        "F4".to_string(),
                        "G4".to_string(),
                        "Ab4".to_string(),
                        "Bb4".to_string(),
                    ],
                    tempo_range: [100.0, 140.0],
                    midi_range: [60, 84],
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curriculum_structure() {
        let curriculum = get_curriculum();
        assert_eq!(curriculum.len(), 4);
        assert_eq!(curriculum[0].stage, 1);
        assert_eq!(curriculum[3].stage, 4);

        // Each stage has exercises
        for stage in &curriculum {
            assert!(!stage.exercises.is_empty());
            for ex in &stage.exercises {
                assert!(!ex.keys.is_empty());
                assert!(ex.tempo_range[0] <= ex.tempo_range[1]);
                assert!(ex.midi_range[0] <= ex.midi_range[1]);
            }
        }
    }

    #[test]
    fn test_stage_difficulty_progression() {
        let curriculum = get_curriculum();
        for (i, stage) in curriculum.iter().enumerate() {
            let expected_difficulty = (i + 1) as u8;
            for ex in &stage.exercises {
                assert_eq!(
                    ex.difficulty, expected_difficulty,
                    "Exercise '{}' in stage {} has wrong difficulty",
                    ex.name, stage.stage
                );
            }
        }
    }
}
