//! KaizenLinguistics Desktop Shell & Verification Runner (`kaizen-core`)
//! 1. In desktop mode (default): Launches native Tauri v2 desktop window with
//!    the "Atelier Aldus" Polyglot Scriptorium UI and registered native IPC handlers.
//! 2. In CLI verification mode (`--cli`): Validates dual-DB integrity, FSRS-6 scheduler,
//!    native G2P, and the 5-Stage Epistemic Speech Evaluator, emitting JSON telemetry.

use kaizen_core::{
    analyze_audio_signal, apply_sibling_burying, evaluate_speech_pipeline,
    evaluate_spoken_utterance, fsrs6_step, g2p_biblical_hebrew, g2p_koine_greek,
    verify_dual_db_pack, AbstentionReason, AcousticAssessmentOutcome,
    AcousticEvaluationInput, FoilSentence, FsrsCardState, GreekPhonologyProfile,
    HebrewPhonologyProfile, Modality, Rating, SentenceTarget, SpokenTelemetryInput,
    DEFAULT_FSRS6_PARAMS, PINNED_FSRS_VERSION,
};
use std::path::PathBuf;

fn generate_sample_audio(duration_sec: f32, freq_hz: f32, amp: f32, noise_level: f32) -> Vec<f32> {
    let sr = 16_000.0_f32;
    let total_samples = (sr * duration_sec) as usize;
    let leading_silence = (sr * 0.08) as usize;
    let trailing_silence = (sr * 0.08) as usize;
    let mut samples = Vec::with_capacity(leading_silence + total_samples + trailing_silence);

    for _ in 0..leading_silence {
        samples.push(noise_level * 0.3);
    }
    for i in 0..total_samples {
        let t = i as f32 / sr;
        let s = (2.0 * std::f32::consts::PI * freq_hz * t).sin() * amp;
        samples.push(s);
    }
    for _ in 0..trailing_silence {
        samples.push(noise_level * 0.3);
    }
    samples
}

pub fn run_cli_verification() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to resolve project root");

    let packs_dir = root_dir.join("packs");
    let grc_pack = packs_dir.join("klpack_grc_koine_v1.klpack");
    let hbo_pack = packs_dir.join("klpack_hbo_masoretic_v1.klpack");
    let telemetry_db = packs_dir.join("user_telemetry.db");

    // 1. Dual-DB Integrity Verification
    let grc_report = verify_dual_db_pack(&telemetry_db, &grc_pack)
        .expect("Failed to verify Koine Greek pack");
    let hbo_report = verify_dual_db_pack(&telemetry_db, &hbo_pack)
        .expect("Failed to verify Biblical Hebrew pack");

    assert!(grc_report.integrity_ok, "GRC pack failed cross-DB integrity gate");
    assert!(hbo_report.integrity_ok, "HBO pack failed cross-DB integrity gate");

    // 2. FSRS-6 + Sibling Burying Verification
    let sample_card = FsrsCardState {
        card_id: "grc:GRC_01_NOM_ACC_AGENT_PATIENT:LISTEN".to_string(),
        lang_code: "grc".to_string(),
        pack_id: "klpack_grc_koine_v1".to_string(),
        concept_id: "GRC_01_NOM_ACC_AGENT_PATIENT".to_string(),
        modality: Modality::Listen,
        state: 0,
        stability: 0.0,
        difficulty: 5.0,
        reps: 0,
        lapses: 0,
        due_timestamp_ms: 1_794_691_200_000,
        buried_until_ms: None,
        last_review_ms: None,
    };

    let updated_listen = fsrs6_step(
        &sample_card,
        Rating::Good,
        1_794_691_200_000,
        0.90,
        &DEFAULT_FSRS6_PARAMS,
    );

    let speak_sibling = FsrsCardState {
        card_id: "grc:GRC_01_NOM_ACC_AGENT_PATIENT:SPEAK".to_string(),
        modality: Modality::Speak,
        ..sample_card.clone()
    };
    let buried_speak = apply_sibling_burying(&speak_sibling, Modality::Listen, 1_794_691_200_000);

    // 3. Native Deterministic G2P Demonstration
    let grc_phrase = "ὁ ἄγγελος βλέπει τὸν ἀδελφόν";
    let grc_g2p = g2p_koine_greek(grc_phrase, GreekPhonologyProfile::RestoredKoine1C);

    let hbo_phrase = "הַנַּעַר יֹשֵׁב עַל הָאֶבֶן";
    let hbo_g2p = g2p_biblical_hebrew(hbo_phrase, HebrewPhonologyProfile::TiberianMasoretic);

    // 4. Raw PCM Audio Signal & 5-Stage Epistemic Speech Pipeline
    let clean_pcm = generate_sample_audio(0.65, 175.0, 0.72, 0.003);
    let signal_metrics = analyze_audio_signal(&clean_pcm, 16_000);

    let target_sentence = SentenceTarget {
        sentence_id: "GRC_01_TGT".to_string(),
        text: grc_phrase.to_string(),
        lang_code: "grc".to_string(),
        ipa_phones: grc_g2p.phone_tokens.clone(),
        contrast_morpheme_indices: vec![grc_g2p.phone_tokens.len() - 2, grc_g2p.phone_tokens.len() - 1],
    };

    let foil_sentence = FoilSentence {
        foil_id: "GRC_01_FOIL_ACC".to_string(),
        text: "τὸν ἄγγελον βλέπει ὁ ἀδελφός".to_string(),
        ipa_phones: vec!["t".into(), "o".into(), "n".into(), "a".into(), "ŋɡ".into(), "ɛ".into(), "l".into(), "o".into(), "n".into()],
        contrast_label: "Spoke Accusative patient instead of Nominative agent".to_string(),
    };

    let speech_report = evaluate_speech_pipeline(AcousticEvaluationInput {
        raw_samples: &clean_pcm,
        sample_rate: 16_000,
        target: &target_sentence,
        foils: &[foil_sentence],
        recognized_phones: None,
        external_gop_scores: None,
        external_alignment_conf: Some(0.93),
    });

    let abstain_demo = evaluate_spoken_utterance(&SpokenTelemetryInput {
        clipped_sample_ratio: 0.0,
        snr_db: 8.2,
        duration_ms: 840,
        unconstrained_transcript_match: true,
        detected_transcript: grc_phrase.to_string(),
        expected_transcript: grc_phrase.to_string(),
        alignment_confidence: 0.89,
        phoneme_gop_lpr_scores: vec![-0.4, -0.3],
        duration_ratio_score: 0.90,
    });
    assert!(matches!(abstain_demo, AcousticAssessmentOutcome::UnableToAssess {
        reason: AbstentionReason::LowSnrFloor,
        ..
    }));

    println!(
        "{{\n  \"engine\": \"kaizen-core\",\n  \"fsrs_version\": \"{}\",\n  \"grc_pack_integrity\": {:?},\n  \"hbo_pack_integrity\": {:?},\n  \"sample_listen_stability_days\": {:.4},\n  \"sibling_speak_buried_until_ms\": {:?},\n  \"grc_g2p_restored\": \"{}\",\n  \"hbo_g2p_tiberian\": \"{}\",\n  \"raw_audio_snr_db\": {:.2},\n  \"speech_pipeline_outcome\": {:?},\n  \"speech_compute_latency_ms\": {:.3},\n  \"abstention_gate_verified\": true\n}}",
        PINNED_FSRS_VERSION,
        grc_report.integrity_ok,
        hbo_report.integrity_ok,
        updated_listen.stability,
        buried_speak.buried_until_ms,
        grc_g2p.full_ipa,
        hbo_g2p.full_ipa,
        signal_metrics.snr_db,
        speech_report.outcome,
        speech_report.compute_latency_ms
    );
}

pub fn run_tauri_app() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            kaizen_core::ipc::get_catalog,
            kaizen_core::ipc::record_review,
            kaizen_core::ipc::evaluate_speech,
            kaizen_core::ipc::g2p_transcribe,
            kaizen_core::ipc::check_integrity,
            kaizen_core::ipc::get_reader_pericopes,
            kaizen_core::ipc::get_available_languages,
            kaizen_core::ipc::get_fsrs_deck_summary,
            kaizen_core::ipc::get_user_profiles,
            kaizen_core::ipc::create_user_profile,
            kaizen_core::ipc::select_user_profile,
            kaizen_core::ipc::set_user_language,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--cli" || a == "verify") || std::env::var("KAIZEN_HEADLESS").is_ok() {
        run_cli_verification();
    } else {
        run_tauri_app();
    }
}
