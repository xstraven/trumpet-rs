use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct PitchResult {
    pub hz: f32,
    pub confidence: f32,
    pub midi_float: f32,
}

impl PitchResult {
    pub fn silence() -> Self {
        PitchResult {
            hz: 0.0,
            confidence: 0.0,
            midi_float: 0.0,
        }
    }
}

const YIN_THRESHOLD: f32 = 0.15;

/// Detect pitch using the YIN algorithm.
/// Returns a PitchResult with frequency, confidence, and fractional MIDI number.
pub fn detect_pitch_yin(samples: &[f32], sample_rate: f32) -> PitchResult {
    if samples.len() < 2 || sample_rate <= 0.0 {
        return PitchResult::silence();
    }

    // Step 1: Compute RMS for silence detection
    let mean = samples.iter().sum::<f32>() / samples.len() as f32;
    let mut energy = 0.0f32;
    for &s in samples {
        let v = s - mean;
        energy += v * v;
    }
    let rms = (energy / samples.len() as f32).sqrt();
    if rms < 0.02 {
        return PitchResult::silence();
    }

    // Frequency range for trumpet (concert pitch): ~80 Hz to ~1200 Hz
    let min_freq = 80.0f32;
    let max_freq = 1200.0f32;
    let min_lag = (sample_rate / max_freq).ceil() as usize;
    let max_lag = (sample_rate / min_freq).floor() as usize;

    let half_len = samples.len() / 2;
    let max_lag = max_lag.min(half_len);

    if min_lag >= max_lag || max_lag < 2 {
        return PitchResult::silence();
    }

    // Step 2: Difference function
    let mut diff = vec![0.0f32; max_lag + 1];
    for tau in 1..=max_lag {
        let mut sum = 0.0f32;
        for j in 0..half_len {
            let d = samples[j] - samples[j + tau];
            sum += d * d;
        }
        diff[tau] = sum;
    }

    // Step 3: Cumulative mean normalized difference function
    let mut cmnd = vec![0.0f32; max_lag + 1];
    cmnd[0] = 1.0;
    let mut running_sum = 0.0f32;
    for tau in 1..=max_lag {
        running_sum += diff[tau];
        if running_sum > 0.0 {
            cmnd[tau] = diff[tau] * tau as f32 / running_sum;
        } else {
            cmnd[tau] = 1.0;
        }
    }

    // Step 4: Absolute threshold -- find the first dip below threshold
    // starting from min_lag (to ignore frequencies above max_freq)
    let mut best_tau = 0usize;
    for tau in min_lag..=max_lag {
        if cmnd[tau] < YIN_THRESHOLD {
            // Walk forward to the local minimum of this valley
            let mut t = tau;
            while t + 1 <= max_lag && cmnd[t + 1] < cmnd[t] {
                t += 1;
            }
            best_tau = t;
            break;
        }
    }

    // If no dip below threshold found, pick the global minimum
    if best_tau == 0 {
        let mut min_val = f32::MAX;
        for tau in min_lag..=max_lag {
            if cmnd[tau] < min_val {
                min_val = cmnd[tau];
                best_tau = tau;
            }
        }
        // If the minimum is still very high, probably not a pitched signal
        if min_val > 0.5 {
            return PitchResult::silence();
        }
    }

    // Step 5: Parabolic interpolation for sub-sample accuracy
    let tau_refined = if best_tau > 0 && best_tau < max_lag {
        let alpha = cmnd[best_tau - 1];
        let beta = cmnd[best_tau];
        let gamma = cmnd[best_tau + 1];
        let denom = 2.0 * (2.0 * beta - alpha - gamma);
        if denom.abs() > 1e-10 {
            best_tau as f32 + (alpha - gamma) / denom
        } else {
            best_tau as f32
        }
    } else {
        best_tau as f32
    };

    if tau_refined <= 0.0 {
        return PitchResult::silence();
    }

    let hz = sample_rate / tau_refined;
    let confidence = 1.0 - cmnd[best_tau].min(1.0);
    let midi_float = 69.0 + 12.0 * (hz / 440.0).log2();

    PitchResult {
        hz,
        confidence,
        midi_float,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn generate_sine(freq: f32, sample_rate: f32, duration: f32) -> Vec<f32> {
        let n = (sample_rate * duration) as usize;
        (0..n)
            .map(|i| 0.5 * (2.0 * PI * freq * i as f32 / sample_rate).sin())
            .collect()
    }

    #[test]
    fn test_yin_a440() {
        let samples = generate_sine(440.0, 44100.0, 0.1);
        let result = detect_pitch_yin(&samples, 44100.0);
        assert!(result.hz > 0.0, "Should detect pitch");
        let error = (result.hz - 440.0).abs();
        assert!(error < 2.0, "Expected ~440 Hz, got {} (error {})", result.hz, error);
        assert!(result.confidence > 0.8, "Should have high confidence: {}", result.confidence);
        let midi_error = (result.midi_float - 69.0).abs();
        assert!(midi_error < 0.1, "MIDI should be ~69, got {}", result.midi_float);
    }

    #[test]
    fn test_yin_bb3() {
        // Bb3 = 233.08 Hz (concert pitch, common trumpet note)
        let samples = generate_sine(233.08, 44100.0, 0.1);
        let result = detect_pitch_yin(&samples, 44100.0);
        let error = (result.hz - 233.08).abs();
        assert!(error < 2.0, "Expected ~233 Hz, got {} (error {})", result.hz, error);
    }

    #[test]
    fn test_yin_c6() {
        // C6 = 1046.5 Hz (high trumpet range)
        // At high frequencies, fewer samples per period so tolerance is wider
        let samples = generate_sine(1046.5, 44100.0, 0.1);
        let result = detect_pitch_yin(&samples, 44100.0);
        let error = (result.hz - 1046.5).abs();
        assert!(error < 10.0, "Expected ~1047 Hz, got {} (error {})", result.hz, error);
    }

    #[test]
    fn test_yin_silence() {
        let samples = vec![0.0; 4410];
        let result = detect_pitch_yin(&samples, 44100.0);
        assert_eq!(result.hz, 0.0);
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn test_yin_empty() {
        let result = detect_pitch_yin(&[], 44100.0);
        assert_eq!(result.hz, 0.0);
    }

    #[test]
    fn test_yin_octave_robustness() {
        // Generate a signal with harmonics (fundamental + octave)
        // YIN should still detect the fundamental
        let n = 4410;
        let sample_rate = 44100.0;
        let fundamental = 440.0;
        let samples: Vec<f32> = (0..n)
            .map(|i| {
                let t = i as f32 / sample_rate;
                0.5 * (2.0 * PI * fundamental * t).sin()
                    + 0.3 * (2.0 * PI * 2.0 * fundamental * t).sin() // octave harmonic
                    + 0.1 * (2.0 * PI * 3.0 * fundamental * t).sin() // third harmonic
            })
            .collect();
        let result = detect_pitch_yin(&samples, sample_rate);
        let error = (result.hz - fundamental).abs();
        assert!(
            error < 5.0,
            "Should detect fundamental 440 Hz despite harmonics, got {} (error {})",
            result.hz,
            error
        );
    }
}
