//! KaizenLinguistics — High-Performance Audio Signal Processing & Feature Extraction
//!
//! Implements:
//! 1. 16 kHz Mono PCM Signal Quality & Usability Analysis:
//!    - Peak amplitude and clipping ratio (|sample| >= 0.99)
//!    - Ambient noise floor estimation and Signal-to-Noise Ratio (SNR in dB)
//!    - Active speech duration and VAD endpoint validation
//! 2. Pitch ($F_0$) Contour Extraction via Normalized Autocorrelation (ACF)
//!    over 25 ms Hamming-windowed frames with 10 ms hop (70 Hz to 400 Hz range)
//! 3. Normalized Waveform Display Envelope (0..100 integer height) for HUD visualization
//! 4. Push-to-Talk (PTT) stream buffer management

use serde::{Deserialize, Serialize};

pub const DEFAULT_SAMPLE_RATE: u32 = 16_000;
pub const CLIPPING_AMPLITUDE_THRESHOLD: f32 = 0.99;
pub const MIN_SPEECH_DURATION_MS: u32 = 220;
pub const MIN_USABLE_SNR_DB: f64 = 12.0;
pub const MAX_ACCEPTABLE_CLIPPED_RATIO: f64 = 0.005;

pub const DEFAULT_FRAME_SIZE_MS: u32 = 25;
pub const DEFAULT_HOP_SIZE_MS: u32 = 10;
pub const MIN_HUMAN_F0_HZ: f32 = 70.0;
pub const MAX_HUMAN_F0_HZ: f32 = 400.0;

/// Extracted acoustic signal metrics evaluated during Gate 1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioSignalMetrics {
    pub duration_ms: u32,
    pub sample_count: usize,
    pub peak_amplitude: f32,
    pub clipped_samples: usize,
    pub clipped_sample_ratio: f64,
    pub ambient_noise_floor_rms: f32,
    pub signal_rms: f32,
    pub snr_db: f64,
    pub active_speech_duration_ms: u32,
    pub is_clipped: bool,
    pub is_low_snr: bool,
    pub is_truncated: bool,
}

/// Analyzes a raw 16 kHz mono 32-bit float PCM audio buffer.
pub fn analyze_audio_signal(samples: &[f32], sample_rate: u32) -> AudioSignalMetrics {
    if samples.is_empty() {
        return AudioSignalMetrics {
            duration_ms: 0,
            sample_count: 0,
            peak_amplitude: 0.0,
            clipped_samples: 0,
            clipped_sample_ratio: 0.0,
            ambient_noise_floor_rms: 0.0,
            signal_rms: 0.0,
            snr_db: 0.0,
            active_speech_duration_ms: 0,
            is_clipped: false,
            is_low_snr: true,
            is_truncated: true,
        };
    }

    let sr = if sample_rate == 0 { DEFAULT_SAMPLE_RATE } else { sample_rate };
    let duration_ms = ((samples.len() as f64 / sr as f64) * 1000.0).round() as u32;

    let mut peak_amplitude: f32 = 0.0;
    let mut clipped_samples: usize = 0;

    for &s in samples {
        let abs_val = s.abs();
        if abs_val > peak_amplitude {
            peak_amplitude = abs_val;
        }
        if abs_val >= CLIPPING_AMPLITUDE_THRESHOLD {
            clipped_samples += 1;
        }
    }

    let clipped_sample_ratio = clipped_samples as f64 / samples.len() as f64;

    // Frame-based energy analysis: 20 ms frames (320 samples @ 16kHz) with 10 ms hop (160 samples)
    let frame_len = (sr as usize * 20) / 1000;
    let hop_len = (sr as usize * 10) / 1000;

    let mut frame_rms_values: Vec<f32> = Vec::new();
    let mut pos = 0;
    while pos + frame_len <= samples.len() {
        let window = &samples[pos..pos + frame_len];
        let sum_sq: f32 = window.iter().map(|&x| x * x).sum();
        let rms = (sum_sq / frame_len as f32).sqrt();
        frame_rms_values.push(rms);
        pos += hop_len;
    }

    if frame_rms_values.is_empty() {
        let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();
        frame_rms_values.push(rms);
    }

    // Noise floor: estimated from 10th percentile energy frame or first 100ms
    let mut sorted_rms = frame_rms_values.clone();
    sorted_rms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let noise_idx = (sorted_rms.len() as f64 * 0.10).floor() as usize;
    let ambient_noise_floor_rms = sorted_rms[noise_idx.min(sorted_rms.len() - 1)].max(1e-5);

    // Speech threshold: frames with RMS >= 2.2x ambient noise floor
    let speech_threshold = (ambient_noise_floor_rms * 2.2).max(0.012);
    let mut speech_frames_rms: Vec<f32> = Vec::new();

    for &rms in &frame_rms_values {
        if rms >= speech_threshold {
            speech_frames_rms.push(rms);
        }
    }

    let (signal_rms, active_speech_duration_ms) = if !speech_frames_rms.is_empty() {
        let sum: f32 = speech_frames_rms.iter().sum();
        let mean_rms = sum / speech_frames_rms.len() as f32;
        let act_ms = (speech_frames_rms.len() * hop_len * 1000 / sr as usize) as u32;
        (mean_rms, act_ms)
    } else {
        // Fallback to mean of upper half
        let half_idx = sorted_rms.len() / 2;
        let upper_sum: f32 = sorted_rms[half_idx..].iter().sum();
        let mean_rms = upper_sum / (sorted_rms.len() - half_idx) as f32;
        (mean_rms, duration_ms)
    };

    let snr_db = if ambient_noise_floor_rms > 0.0 && signal_rms > 0.0 {
        let ratio = (signal_rms / ambient_noise_floor_rms) as f64;
        20.0 * ratio.max(1e-3).log10()
    } else {
        0.0
    };

    let is_clipped = clipped_sample_ratio > MAX_ACCEPTABLE_CLIPPED_RATIO;
    let is_low_snr = snr_db < MIN_USABLE_SNR_DB;
    let is_truncated = duration_ms < MIN_SPEECH_DURATION_MS;

    AudioSignalMetrics {
        duration_ms,
        sample_count: samples.len(),
        peak_amplitude,
        clipped_samples,
        clipped_sample_ratio,
        ambient_noise_floor_rms,
        signal_rms,
        snr_db: (snr_db * 100.0).round() / 100.0,
        active_speech_duration_ms,
        is_clipped,
        is_low_snr,
        is_truncated,
    }
}

/// Extracts fundamental frequency ($F_0$ in Hz) contour across 25 ms frames with 10 ms hop
/// using Normalized Autocorrelation Function (ACF) with peak parabolic interpolation.
pub fn extract_pitch_contour(
    samples: &[f32],
    sample_rate: u32,
    frame_size_ms: u32,
    hop_size_ms: u32,
) -> Vec<f32> {
    let sr = if sample_rate == 0 { DEFAULT_SAMPLE_RATE } else { sample_rate };
    let f_ms = if frame_size_ms == 0 { DEFAULT_FRAME_SIZE_MS } else { frame_size_ms };
    let h_ms = if hop_size_ms == 0 { DEFAULT_HOP_SIZE_MS } else { hop_size_ms };

    let frame_len = (sr as usize * f_ms as usize) / 1000;
    let hop_len = (sr as usize * h_ms as usize) / 1000;

    let min_lag = (sr as f32 / MAX_HUMAN_F0_HZ).floor() as usize; // ~40 samples @ 16kHz
    let max_lag = (sr as f32 / MIN_HUMAN_F0_HZ).ceil() as usize;  // ~228 samples @ 16kHz

    if samples.len() < frame_len {
        return Vec::new();
    }

    // Precompute Hamming window
    let mut window = vec![0.0_f32; frame_len];
    for i in 0..frame_len {
        window[i] = 0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / (frame_len - 1) as f32).cos();
    }

    let mut f0_contour: Vec<f32> = Vec::new();
    let mut pos = 0;

    while pos + frame_len <= samples.len() {
        let frame_raw = &samples[pos..pos + frame_len];

        // Apply window and remove DC offset
        let sum_raw: f32 = frame_raw.iter().sum();
        let dc = sum_raw / frame_len as f32;
        let mut frame = vec![0.0_f32; frame_len];
        for i in 0..frame_len {
            frame[i] = (frame_raw[i] - dc) * window[i];
        }

        let energy_r0: f32 = frame.iter().map(|&x| x * x).sum();
        let frame_rms = (energy_r0 / frame_len as f32).sqrt();

        // Energy gate: silence or ambient noise frames are unvoiced
        if frame_rms < 0.015 {
            f0_contour.push(0.0);
            pos += hop_len;
            continue;
        }

        // Compute Normalized Cross-Correlation for lags in [min_lag, max_lag]
        let mut corr_values = vec![0.0_f32; max_lag + 2];
        let mut best_lag = 0;
        let mut best_corr = 0.0_f32;

        for lag in min_lag..=max_lag.min(frame_len - 2) {
            let mut sum_prod = 0.0_f32;
            let mut sum_orig_sq = 0.0_f32;
            let mut sum_delayed_sq = 0.0_f32;

            for i in 0..(frame_len - lag) {
                sum_prod += frame[i] * frame[i + lag];
                sum_orig_sq += frame[i] * frame[i];
                sum_delayed_sq += frame[i + lag] * frame[i + lag];
            }

            let denom = (sum_orig_sq * sum_delayed_sq).sqrt();
            let norm_corr = if denom > 1e-6 { (sum_prod / denom).clamp(-1.0, 1.0) } else { 0.0 };
            corr_values[lag] = norm_corr;

            if norm_corr > best_corr {
                best_corr = norm_corr;
                best_lag = lag;
            }
        }

        // Voicing threshold: normalized autocorrelation must exceed 0.38 for valid pitch
        if best_corr >= 0.38 && best_lag > min_lag && best_lag < max_lag {
            // Parabolic interpolation for sub-sample peak localization
            let y_prev = corr_values[best_lag - 1];
            let y_peak = corr_values[best_lag];
            let y_next = corr_values[best_lag + 1];
            let denom = y_prev - 2.0 * y_peak + y_next;
            let delta = if denom.abs() > 1e-5 {
                0.5 * (y_prev - y_next) / denom
            } else {
                0.0
            };
            let refined_lag = (best_lag as f32 + delta.clamp(-0.5, 0.5)).max(1.0);
            let f0 = sr as f32 / refined_lag;
            if (MIN_HUMAN_F0_HZ..=MAX_HUMAN_F0_HZ).contains(&f0) {
                f0_contour.push((f0 * 10.0).round() / 10.0);
            } else {
                f0_contour.push(0.0);
            }
        } else {
            f0_contour.push(0.0);
        }

        pos += hop_len;
    }

    f0_contour
}

/// Generates normalized waveform display bars (0..100) for UI HUD rendering.
pub fn generate_waveform_bars(samples: &[f32], target_bar_count: usize) -> Vec<u8> {
    let num_bars = target_bar_count.clamp(16, 128);
    if samples.is_empty() {
        return vec![0; num_bars];
    }

    let chunk_size = samples.len() as f64 / num_bars as f64;
    let mut raw_energies: Vec<f32> = Vec::with_capacity(num_bars);

    for bar_idx in 0..num_bars {
        let start_idx = (bar_idx as f64 * chunk_size).floor() as usize;
        let end_idx = (((bar_idx + 1) as f64 * chunk_size).ceil() as usize).min(samples.len());

        if start_idx < end_idx {
            let chunk = &samples[start_idx..end_idx];
            let sum_sq: f32 = chunk.iter().map(|&x| x * x).sum();
            let rms = (sum_sq / chunk.len() as f32).sqrt();
            raw_energies.push(rms);
        } else {
            raw_energies.push(0.0);
        }
    }

    let max_energy = raw_energies
        .iter()
        .copied()
        .fold(0.0_f32, f32::max)
        .max(1e-4);

    raw_energies
        .iter()
        .map(|&e| {
            let normalized = (e / max_energy).clamp(0.0, 1.0);
            // Non-linear power scale for visual perception
            let bar_val = (normalized.powf(0.65) * 100.0).round() as u8;
            bar_val.clamp(4, 100)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_sine_with_silence(
        freq_hz: f32,
        speech_sec: f32,
        leading_silence_sec: f32,
        trailing_silence_sec: f32,
        amplitude: f32,
        noise_level: f32,
    ) -> Vec<f32> {
        let sr = DEFAULT_SAMPLE_RATE as f32;
        let total_speech_samples = (sr * speech_sec) as usize;
        let leading_samples = (sr * leading_silence_sec) as usize;
        let trailing_samples = (sr * trailing_silence_sec) as usize;
        let mut samples = Vec::with_capacity(leading_samples + total_speech_samples + trailing_samples);

        // Leading ambient room noise
        for i in 0..leading_samples {
            let pseudo_noise = (((i * 1103515245 + 12345) % 65536) as f32 / 32768.0 - 1.0) * noise_level;
            samples.push(pseudo_noise);
        }

        // Active speech sine
        for i in 0..total_speech_samples {
            let t = i as f32 / sr;
            let sine = (2.0 * std::f32::consts::PI * freq_hz * t).sin() * amplitude;
            let pseudo_noise = ((((i + leading_samples) * 1103515245 + 12345) % 65536) as f32 / 32768.0 - 1.0) * noise_level;
            samples.push((sine + pseudo_noise).clamp(-1.0, 1.0));
        }

        // Trailing ambient room noise
        for i in 0..trailing_samples {
            let pseudo_noise = ((((i + leading_samples + total_speech_samples) * 1103515245 + 12345) % 65536) as f32 / 32768.0 - 1.0) * noise_level;
            samples.push(pseudo_noise);
        }

        samples
    }

    #[test]
    fn test_clean_audio_signal_metrics_pass() {
        let audio = generate_sine_with_silence(160.0, 0.8, 0.1, 0.1, 0.65, 0.005);
        let metrics = analyze_audio_signal(&audio, DEFAULT_SAMPLE_RATE);

        assert_eq!(metrics.duration_ms, 1000);
        assert!(!metrics.is_clipped);
        assert!(!metrics.is_low_snr);
        assert!(!metrics.is_truncated);
        assert!(metrics.snr_db >= 18.0);
        assert!(metrics.peak_amplitude <= 0.75);
    }

    #[test]
    fn test_clipped_signal_detection() {
        // Generate heavily clipped square/saturated wave
        let mut audio = generate_sine_with_silence(160.0, 0.5, 0.05, 0.05, 1.5, 0.0);
        for s in &mut audio {
            if *s > 0.99 {
                *s = 1.0;
            } else if *s < -0.99 {
                *s = -1.0;
            }
        }
        let metrics = analyze_audio_signal(&audio, DEFAULT_SAMPLE_RATE);
        assert!(metrics.is_clipped);
        assert!(metrics.clipped_sample_ratio > 0.05);
    }

    #[test]
    fn test_low_snr_and_truncation_detection() {
        // High noise, low signal
        let noisy = generate_sine_with_silence(160.0, 0.6, 0.1, 0.1, 0.02, 0.08);
        let metrics_noisy = analyze_audio_signal(&noisy, DEFAULT_SAMPLE_RATE);
        assert!(metrics_noisy.is_low_snr);

        // Truncated clip (total 140ms < 220ms)
        let short_audio = generate_sine_with_silence(160.0, 0.10, 0.02, 0.02, 0.5, 0.01);
        let metrics_short = analyze_audio_signal(&short_audio, DEFAULT_SAMPLE_RATE);
        assert!(metrics_short.is_truncated);
    }

    #[test]
    fn test_pitch_f0_extraction_accuracy() {
        let freq_target = 180.0_f32;
        let audio = generate_sine_with_silence(freq_target, 0.6, 0.05, 0.05, 0.7, 0.002);
        let f0_track = extract_pitch_contour(&audio, DEFAULT_SAMPLE_RATE, 25, 10);
        assert!(!f0_track.is_empty());
        let voiced_f0s: Vec<f32> = f0_track.into_iter().filter(|&f| f > 0.0).collect();
        assert!(!voiced_f0s.is_empty());

        let mean_f0: f32 = voiced_f0s.iter().sum::<f32>() / voiced_f0s.len() as f32;
        assert!(
            (mean_f0 - freq_target).abs() < 5.0,
            "Expected mean F0 near {}, got {}",
            freq_target,
            mean_f0
        );
    }

    #[test]
    fn test_waveform_bars_generation() {
        let audio = generate_sine_with_silence(200.0, 0.4, 0.05, 0.05, 0.5, 0.01);
        let bars = generate_waveform_bars(&audio, 32);

        assert_eq!(bars.len(), 32);
        for &b in &bars {
            assert!(b >= 4 && b <= 100);
        }
    }
}
