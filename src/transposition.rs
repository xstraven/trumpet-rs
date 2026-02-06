use crate::scoring::types::TransposeInfo;

/// Convert a concert-pitch MIDI note to written pitch for the instrument.
/// For Bb trumpet: chromatic = -2, so written C4 (60) sounds as concert Bb3 (58).
/// concert_to_written: midi - chromatic (e.g., 58 - (-2) = 60)
pub fn concert_to_written(midi_concert: i32, transpose: &TransposeInfo) -> i32 {
    midi_concert - transpose.chromatic
}

/// Convert a written-pitch MIDI note to concert pitch.
/// For Bb trumpet: written C4 (60) + chromatic(-2) = concert Bb3 (58)
pub fn written_to_concert(midi_written: i32, transpose: &TransposeInfo) -> i32 {
    midi_written + transpose.chromatic
}

/// Convert a detected frequency (concert pitch from mic) to written-pitch
/// fractional MIDI value for display purposes.
pub fn freq_to_written_midi(freq_hz: f64, transpose: &TransposeInfo) -> f64 {
    let concert_midi = 69.0 + 12.0 * (freq_hz / 440.0).log2();
    concert_midi - transpose.chromatic as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bb_trumpet() -> TransposeInfo {
        TransposeInfo {
            chromatic: -2,
            diatonic: -1,
        }
    }

    #[test]
    fn test_concert_to_written_bb_trumpet() {
        let t = bb_trumpet();
        // Concert Bb3 (58) -> Written C4 (60)
        assert_eq!(concert_to_written(58, &t), 60);
        // Concert A4 (69) -> Written B4 (71)
        assert_eq!(concert_to_written(69, &t), 71);
    }

    #[test]
    fn test_written_to_concert_bb_trumpet() {
        let t = bb_trumpet();
        // Written C4 (60) -> Concert Bb3 (58)
        assert_eq!(written_to_concert(60, &t), 58);
        // Written B4 (71) -> Concert A4 (69)
        assert_eq!(written_to_concert(71, &t), 69);
    }

    #[test]
    fn test_roundtrip() {
        let t = bb_trumpet();
        for midi in 48..=84 {
            assert_eq!(concert_to_written(written_to_concert(midi, &t), &t), midi);
        }
    }

    #[test]
    fn test_no_transposition() {
        let t = TransposeInfo {
            chromatic: 0,
            diatonic: 0,
        };
        assert_eq!(concert_to_written(60, &t), 60);
        assert_eq!(written_to_concert(60, &t), 60);
    }

    #[test]
    fn test_freq_to_written_midi() {
        let t = bb_trumpet();
        // A4 = 440 Hz, concert MIDI 69, written MIDI 71
        let written = freq_to_written_midi(440.0, &t);
        assert!((written - 71.0).abs() < 0.01);
    }
}
