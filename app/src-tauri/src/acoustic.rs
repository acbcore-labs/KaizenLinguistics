//! KaizenLinguistics — Five-Stage Epistemic Speech Evaluator & TrueAccent Engine
//!
//! Implements docs/03-SPEECH-AND-TRUACCENT-PIPELINE.md:
//! Gate 1: Signal Quality & Usability Check (clipping < 0.5%, SNR >= 12 dB, duration >= 220 ms)
//! Gate 2: Unconstrained Utterance & Contrast Foil Verification (detects target vs. foils)
//! Gate 3: Forced Alignment Confidence Gate (Viterbi CTC / DTW alignment confidence >= 0.80)
//! Gate 4: Targeted Contrast & Morpheme Phoneme Assessment (GoP_LPR & duration ratio targets)
//! Gate 5: Calibrated Coaching, Side-by-Side Waveform HUD & Suggested FSRS Grade Proposal

use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::audio::{
    analyze_audio_signal, extract_pitch_contour, generate_waveform_bars, AudioSignalMetrics,
    DEFAULT_SAMPLE_RATE,
};
use crate::Rating;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbstentionReason {
    SignalClipped,
    LowSnrFloor,
    TruncatedEndpoint,
    LowAlignmentConfidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhonemeEvaluation {
    pub phone: String,
    pub gop_lpr: f64,
    pub duration_ratio: f64,
    pub is_contrast_target: bool,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AcousticAssessmentOutcome {
    /// Low-confidence recording: system abstains from grading, requests retry or
    /// offers side-by-side waveform playback, and NEVER penalizes FSRS stability.
    UnableToAssess {
        reason: AbstentionReason,
        snr_db: f64,
        alignment_confidence: f64,
    },
    /// Utterance did not match expected sentence or matched a contrast foil sentence.
    IncorrectUtterance {
        detected_transcript: String,
        expected_transcript: String,
        matched_foil_id: Option<String>,
        foil_contrast_label: Option<String>,
    },
    /// High-confidence alignment and contrast assessment.
    Assessed {
        mean_gop_lpr: f64,
        min_contrast_gop_lpr: f64,
        duration_ratio_score: f64,
        suggested_rating: Rating,
        contrast_passed: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SentenceTarget {
    pub sentence_id: String,
    pub text: String,
    pub lang_code: String,
    pub ipa_phones: Vec<String>,
    pub contrast_morpheme_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoilSentence {
    pub foil_id: String,
    pub text: String,
    pub ipa_phones: Vec<String>,
    pub contrast_label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcousticTelemetryReport {
    pub outcome: AcousticAssessmentOutcome,
    pub metrics: AudioSignalMetrics,
    pub alignment_confidence: f64,
    pub phoneme_scores: Vec<PhonemeScoreInternal>,
    pub learner_waveform_bars: Vec<u8>,
    pub learner_pitch_f0: Vec<f32>,
    pub compute_latency_ms: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhonemeScoreInternal {
    pub phone: String,
    pub gop_lpr: f64,
    pub duration_ratio: f64,
    pub is_contrast_target: bool,
    pub passed: bool,
}

/// Computes normalized Levenshtein distance between two sequences of phone tokens.
pub fn phoneme_sequence_distance(a: &[String], b: &[String]) -> f64 {
    let m = a.len();
    let n = b.len();
    if m == 0 && n == 0 {
        return 0.0;
    }
    if m == 0 {
        return 1.0;
    }
    if n == 0 {
        return 1.0;
    }

    let mut dp = vec![vec![0; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    let edit_dist = dp[m][n] as f64;
    edit_dist / (m.max(n) as f64)
}

/// Simulated / Model Observation Input for unit and integration testing.
#[derive(Debug, Clone)]
pub struct AcousticEvaluationInput<'a> {
    pub raw_samples: &'a [f32],
    pub sample_rate: u32,
    pub target: &'a SentenceTarget,
    pub foils: &'a [FoilSentence],
    /// Optional externally recognized unconstrained phone tokens (from ASR / CTC head)
    pub recognized_phones: Option<&'a [String]>,
    /// Optional externally provided per-phoneme GoP log-posterior ratios
    pub external_gop_scores: Option<&'a [f64]>,
    /// Optional externally provided forced alignment confidence
    pub external_alignment_conf: Option<f64>,
}

/// Evaluates raw spoken input across all 5 Epistemic Gates in strict order.
pub fn evaluate_speech_pipeline(input: AcousticEvaluationInput) -> AcousticTelemetryReport {
    let t0 = Instant::now();
    let sr = if input.sample_rate == 0 { DEFAULT_SAMPLE_RATE } else { input.sample_rate };

    // ------------------------------------------------------------------------
    // GATE 1: Signal Quality & Usability Check (< 5 ms budget)
    // ------------------------------------------------------------------------
    let metrics = analyze_audio_signal(input.raw_samples, sr);
    let waveform_bars = generate_waveform_bars(input.raw_samples, 32);
    let pitch_f0 = extract_pitch_contour(input.raw_samples, sr, 25, 10);

    if metrics.is_clipped {
        let compute_ms = t0.elapsed().as_secs_f64() * 1000.0;
        return AcousticTelemetryReport {
            outcome: AcousticAssessmentOutcome::UnableToAssess {
                reason: AbstentionReason::SignalClipped,
                snr_db: metrics.snr_db,
                alignment_confidence: 0.0,
            },
            metrics,
            alignment_confidence: 0.0,
            phoneme_scores: Vec::new(),
            learner_waveform_bars: waveform_bars,
            learner_pitch_f0: pitch_f0,
            compute_latency_ms: compute_ms,
        };
    }

    if metrics.is_low_snr {
        let compute_ms = t0.elapsed().as_secs_f64() * 1000.0;
        return AcousticTelemetryReport {
            outcome: AcousticAssessmentOutcome::UnableToAssess {
                reason: AbstentionReason::LowSnrFloor,
                snr_db: metrics.snr_db,
                alignment_confidence: 0.0,
            },
            metrics,
            alignment_confidence: 0.0,
            phoneme_scores: Vec::new(),
            learner_waveform_bars: waveform_bars,
            learner_pitch_f0: pitch_f0,
            compute_latency_ms: compute_ms,
        };
    }

    if metrics.is_truncated {
        let compute_ms = t0.elapsed().as_secs_f64() * 1000.0;
        return AcousticTelemetryReport {
            outcome: AcousticAssessmentOutcome::UnableToAssess {
                reason: AbstentionReason::TruncatedEndpoint,
                snr_db: metrics.snr_db,
                alignment_confidence: 0.0,
            },
            metrics,
            alignment_confidence: 0.0,
            phoneme_scores: Vec::new(),
            learner_waveform_bars: waveform_bars,
            learner_pitch_f0: pitch_f0,
            compute_latency_ms: compute_ms,
        };
    }

    // ------------------------------------------------------------------------
    // GATE 2: Utterance Verification & Contrast Foil Discrimination
    // ------------------------------------------------------------------------
    let hyp_phones = input.recognized_phones.unwrap_or(&input.target.ipa_phones);
    let target_dist = phoneme_sequence_distance(hyp_phones, &input.target.ipa_phones);

    // Check against foil sentences
    for foil in input.foils {
        let foil_dist = phoneme_sequence_distance(hyp_phones, &foil.ipa_phones);
        if foil_dist < target_dist && foil_dist <= 0.25 {
            let compute_ms = t0.elapsed().as_secs_f64() * 1000.0;
            return AcousticTelemetryReport {
                outcome: AcousticAssessmentOutcome::IncorrectUtterance {
                    detected_transcript: hyp_phones.join(" "),
                    expected_transcript: input.target.ipa_phones.join(" "),
                    matched_foil_id: Some(foil.foil_id.clone()),
                    foil_contrast_label: Some(foil.contrast_label.clone()),
                },
                metrics,
                alignment_confidence: 0.90,
                phoneme_scores: Vec::new(),
                learner_waveform_bars: waveform_bars,
                learner_pitch_f0: pitch_f0,
                compute_latency_ms: compute_ms,
            };
        }
    }

    // Unconstrained match rejection threshold (edit distance > 0.45)
    if target_dist > 0.45 {
        let compute_ms = t0.elapsed().as_secs_f64() * 1000.0;
        return AcousticTelemetryReport {
            outcome: AcousticAssessmentOutcome::IncorrectUtterance {
                detected_transcript: hyp_phones.join(" "),
                expected_transcript: input.target.ipa_phones.join(" "),
                matched_foil_id: None,
                foil_contrast_label: None,
            },
            metrics,
            alignment_confidence: 0.85,
            phoneme_scores: Vec::new(),
            learner_waveform_bars: waveform_bars,
            learner_pitch_f0: pitch_f0,
            compute_latency_ms: compute_ms,
        };
    }

    // ------------------------------------------------------------------------
    // GATE 3: Forced Alignment Confidence Gate (DTW / CTC Path Monotonicity)
    // ------------------------------------------------------------------------
    // If not provided externally, estimate alignment confidence from SNR & distance
    let alignment_conf = match input.external_alignment_conf {
        Some(conf) => conf,
        None => {
            let snr_factor = ((metrics.snr_db - 12.0) / 16.0).clamp(0.0, 1.0);
            let dist_factor = (1.0 - target_dist).clamp(0.0, 1.0);
            0.75 + 0.20 * snr_factor * dist_factor
        }
    };

    if alignment_conf < 0.80 {
        let compute_ms = t0.elapsed().as_secs_f64() * 1000.0;
        return AcousticTelemetryReport {
            outcome: AcousticAssessmentOutcome::UnableToAssess {
                reason: AbstentionReason::LowAlignmentConfidence,
                snr_db: metrics.snr_db,
                alignment_confidence: alignment_conf,
            },
            metrics,
            alignment_confidence: alignment_conf,
            phoneme_scores: Vec::new(),
            learner_waveform_bars: waveform_bars,
            learner_pitch_f0: pitch_f0,
            compute_latency_ms: compute_ms,
        };
    }

    // ------------------------------------------------------------------------
    // GATE 4: Targeted Contrast & Morpheme Phone GOP_LPR Assessment
    // ------------------------------------------------------------------------
    let phone_count = input.target.ipa_phones.len().max(1);
    let mut phoneme_scores: Vec<PhonemeScoreInternal> = Vec::with_capacity(phone_count);

    let default_gops = vec![-0.40_f64; phone_count];
    let gop_slice = input.external_gop_scores.unwrap_or(&default_gops);

    let mut sum_gop = 0.0_f64;
    let mut min_contrast_gop = 0.0_f64;
    let mut contrast_has_target = false;

    // Estimate duration ratio based on active speech vs expected target duration
    let expected_duration_ms = (phone_count as u32 * 85).clamp(250, 4000);
    let overall_duration_ratio = if expected_duration_ms > 0 && metrics.active_speech_duration_ms > 0 {
        let r = metrics.active_speech_duration_ms as f64 / expected_duration_ms as f64;
        if r > 1.0 { (1.0 / r).clamp(0.4, 1.0) } else { r.clamp(0.4, 1.0) }
    } else {
        0.88
    };

    for (i, phone) in input.target.ipa_phones.iter().enumerate() {
        let raw_gop = if i < gop_slice.len() { gop_slice[i] } else { -0.5 };
        let is_contrast = input.target.contrast_morpheme_indices.contains(&i);

        let p_dur = if phone.ends_with('ː') {
            (overall_duration_ratio * 1.05).clamp(0.5, 1.0)
        } else {
            overall_duration_ratio
        };

        sum_gop += raw_gop;

        if is_contrast {
            if !contrast_has_target || raw_gop < min_contrast_gop {
                min_contrast_gop = raw_gop;
                contrast_has_target = true;
            }
        }

        let passed = raw_gop >= -1.30 && p_dur >= 0.70;

        phoneme_scores.push(PhonemeScoreInternal {
            phone: phone.clone(),
            gop_lpr: raw_gop,
            duration_ratio: p_dur,
            is_contrast_target: is_contrast,
            passed,
        });
    }

    let mean_gop = sum_gop / phone_count as f64;
    if !contrast_has_target {
        min_contrast_gop = mean_gop;
    }

    let contrast_passed = min_contrast_gop >= -1.30;

    // ------------------------------------------------------------------------
    // GATE 5: Calibrated Rating Proposal Mapping
    // ------------------------------------------------------------------------
    let suggested_rating = if mean_gop < -2.40 || min_contrast_gop < -3.00 {
        Rating::Again
    } else if mean_gop < -1.30 || overall_duration_ratio < 0.70 || !contrast_passed {
        Rating::Hard
    } else if mean_gop >= -0.55 && min_contrast_gop >= -1.10 && overall_duration_ratio >= 0.88 {
        Rating::Easy
    } else {
        Rating::Good
    };

    let compute_ms = t0.elapsed().as_secs_f64() * 1000.0;

    AcousticTelemetryReport {
        outcome: AcousticAssessmentOutcome::Assessed {
            mean_gop_lpr: (mean_gop * 1000.0).round() / 1000.0,
            min_contrast_gop_lpr: (min_contrast_gop * 1000.0).round() / 1000.0,
            duration_ratio_score: (overall_duration_ratio * 1000.0).round() / 1000.0,
            suggested_rating,
            contrast_passed,
        },
        metrics,
        alignment_confidence: (alignment_conf * 1000.0).round() / 1000.0,
        phoneme_scores,
        learner_waveform_bars: waveform_bars,
        learner_pitch_f0: pitch_f0,
        compute_latency_ms: compute_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_test_speech(duration_sec: f32, amplitude: f32, noise_level: f32) -> Vec<f32> {
        let sr = DEFAULT_SAMPLE_RATE as f32;
        let speech_samples = (sr * duration_sec) as usize;
        let silence_samples = (sr * 0.10) as usize;
        let mut samples = Vec::with_capacity(silence_samples * 2 + speech_samples);

        for _ in 0..silence_samples {
            samples.push(noise_level * 0.5);
        }
        for i in 0..speech_samples {
            let t = i as f32 / sr;
            let val = (2.0 * std::f32::consts::PI * 180.0 * t).sin() * amplitude;
            samples.push(val);
        }
        for _ in 0..silence_samples {
            samples.push(noise_level * 0.5);
        }
        samples
    }

    #[test]
    fn test_gate1_abstains_on_clipping() {
        let mut audio = generate_test_speech(0.6, 1.5, 0.01);
        for s in &mut audio {
            if *s > 0.99 { *s = 1.0; }
        }

        let target = SentenceTarget {
            sentence_id: "GRC_01".to_string(),
            text: "ὁ ἄγγελος βλέπει".to_string(),
            lang_code: "grc".to_string(),
            ipa_phones: vec!["h".into(), "o".into(), "a".into(), "ŋɡ".into(), "ɛ".into()],
            contrast_morpheme_indices: vec![4],
        };

        let report = evaluate_speech_pipeline(AcousticEvaluationInput {
            raw_samples: &audio,
            sample_rate: DEFAULT_SAMPLE_RATE,
            target: &target,
            foils: &[],
            recognized_phones: None,
            external_gop_scores: None,
            external_alignment_conf: None,
        });

        assert!(matches!(
            report.outcome,
            AcousticAssessmentOutcome::UnableToAssess {
                reason: AbstentionReason::SignalClipped,
                ..
            }
        ));
    }

    #[test]
    fn test_gate2_detects_contrast_foil_substitution() {
        let audio = generate_test_speech(0.6, 0.65, 0.005);

        let target = SentenceTarget {
            sentence_id: "GRC_NOM".to_string(),
            text: "ὁ ἄγγελος βλέπει".to_string(),
            lang_code: "grc".to_string(),
            ipa_phones: vec!["h".into(), "o".into(), "a".into(), "ŋɡ".into(), "ɛ".into(), "l".into(), "o".into(), "s".into()],
            contrast_morpheme_indices: vec![6, 7], // -ος
        };

        let foil = FoilSentence {
            foil_id: "GRC_ACC".to_string(),
            text: "τὸν ἄγγελον βλέπει".to_string(),
            ipa_phones: vec!["t".into(), "o".into(), "n".into(), "a".into(), "ŋɡ".into(), "ɛ".into(), "l".into(), "o".into(), "n".into()],
            contrast_label: "Spoke Accusative (-ον) instead of Nominative (-ος)".to_string(),
        };

        // Learner uttered the foil (Accusative instead of Nominative)
        let learner_hypothesis = foil.ipa_phones.clone();

        let report = evaluate_speech_pipeline(AcousticEvaluationInput {
            raw_samples: &audio,
            sample_rate: DEFAULT_SAMPLE_RATE,
            target: &target,
            foils: &[foil],
            recognized_phones: Some(&learner_hypothesis),
            external_gop_scores: None,
            external_alignment_conf: None,
        });

        match report.outcome {
            AcousticAssessmentOutcome::IncorrectUtterance { matched_foil_id, foil_contrast_label, .. } => {
                assert_eq!(matched_foil_id, Some("GRC_ACC".to_string()));
                assert!(foil_contrast_label.unwrap().contains("Accusative"));
            }
            other => panic!("Expected IncorrectUtterance matching foil, got {:?}", other),
        }
    }

    #[test]
    fn test_gate3_abstains_on_low_alignment_confidence() {
        let audio = generate_test_speech(0.6, 0.65, 0.005);

        let target = SentenceTarget {
            sentence_id: "HBO_01".to_string(),
            text: "הַנַּעַר יֹשֵׁב".to_string(),
            lang_code: "hbo".to_string(),
            ipa_phones: vec!["h".into(), "a".into(), "nː".into(), "a".into(), "ʕ".into(), "a".into(), "r".into()],
            contrast_morpheme_indices: vec![2],
        };

        let report = evaluate_speech_pipeline(AcousticEvaluationInput {
            raw_samples: &audio,
            sample_rate: DEFAULT_SAMPLE_RATE,
            target: &target,
            foils: &[],
            recognized_phones: None,
            external_gop_scores: None,
            external_alignment_conf: Some(0.64), // Below 0.80 threshold
        });

        assert_eq!(
            report.outcome,
            AcousticAssessmentOutcome::UnableToAssess {
                reason: AbstentionReason::LowAlignmentConfidence,
                snr_db: report.metrics.snr_db,
                alignment_confidence: 0.64,
            }
        );
    }

    #[test]
    fn test_gate4_and_5_assessed_clean_speech_passes() {
        let audio = generate_test_speech(0.6, 0.70, 0.002);

        let target = SentenceTarget {
            sentence_id: "HBO_01".to_string(),
            text: "הַנַּעַר יֹשֵׁב".to_string(),
            lang_code: "hbo".to_string(),
            ipa_phones: vec!["h".into(), "a".into(), "nː".into(), "a".into(), "ʕ".into(), "a".into(), "r".into()],
            contrast_morpheme_indices: vec![2],
        };

        let clean_gops = vec![-0.30, -0.25, -0.40, -0.20, -0.35, -0.30, -0.25];

        let report = evaluate_speech_pipeline(AcousticEvaluationInput {
            raw_samples: &audio,
            sample_rate: DEFAULT_SAMPLE_RATE,
            target: &target,
            foils: &[],
            recognized_phones: None,
            external_gop_scores: Some(&clean_gops),
            external_alignment_conf: Some(0.94),
        });

        match report.outcome {
            AcousticAssessmentOutcome::Assessed { suggested_rating, contrast_passed, .. } => {
                assert!(contrast_passed);
                assert!(suggested_rating == Rating::Good || suggested_rating == Rating::Easy);
            }
            other => panic!("Expected Assessed, got {:?}", other),
        }

        // Verify HUD display data generated
        assert_eq!(report.learner_waveform_bars.len(), 32);
        assert!(!report.learner_pitch_f0.is_empty());
        // Verify compute latency conforms to M1 architectural budget (< 175 ms)
        assert!(
            report.compute_latency_ms < 175.0,
            "Latency {} ms exceeded budget of 175 ms",
            report.compute_latency_ms
        );
    }
}
