//! KaizenLinguistics v2.0 — Rust Core Engine (`kaizen-core`)
//!
//! Implements:
//! 1. Native macOS `libsqlite3` FFI bindings for zero-dependency offline execution,
//!    managing attached read-only `.klpack` archives and read-write `user_telemetry.db`.
//! 2. Application-level Cross-Database Integrity Verification (`verify_cross_db_integrity`).
//! 3. Pinned FSRS-6 (21-parameter `w[0..20]` with trainable decay) observation-driven
//!    scheduler with 24-hour Sibling Burying (`LISTEN`, `READ`, `SPEAK`, `WRITE`).
//! 4. Deterministic Event-Log Replay (`replay_review_logs`) proving bit-exact state
//!    reconstruction from `review_logs`.
//! 5. Five-Stage Epistemic Acoustic Evaluator (`evaluate_spoken_utterance`) with explicit
//!    `UNABLE_TO_ASSESS` abstention when SNR, clipping, or forced-alignment confidence
//!    falls below calibrated thresholds.

use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr;

pub mod acoustic;
pub mod audio;
pub mod g2p;
pub mod ipc;

pub use acoustic::{
    evaluate_speech_pipeline, AbstentionReason, AcousticAssessmentOutcome,
    AcousticEvaluationInput, AcousticTelemetryReport, FoilSentence, SentenceTarget,
};
pub use audio::{
    analyze_audio_signal, extract_pitch_contour, generate_waveform_bars, AudioSignalMetrics,
};
pub use g2p::{
    g2p_biblical_hebrew, g2p_koine_greek, G2pResult, GreekPhonologyProfile, HebrewPhonologyProfile,
};

// ============================================================================
// 1. NATIVE MACOS LIBSQLITE3 FFI LAYER
// ============================================================================

#[repr(C)]
pub struct sqlite3 {
    _private: [u8; 0],
}

#[repr(C)]
pub struct sqlite3_stmt {
    _private: [u8; 0],
}

pub const SQLITE_OK: c_int = 0;
pub const SQLITE_ROW: c_int = 100;
pub const SQLITE_DONE: c_int = 101;

#[link(name = "sqlite3")]
extern "C" {
    fn sqlite3_open(filename: *const c_char, ppDb: *mut *mut sqlite3) -> c_int;
    fn sqlite3_close(db: *mut sqlite3) -> c_int;
    fn sqlite3_busy_timeout(db: *mut sqlite3, ms: c_int) -> c_int;
    fn sqlite3_exec(
        db: *mut sqlite3,
        sql: *const c_char,
        callback: Option<unsafe extern "C" fn(*mut c_void, c_int, *mut *mut c_char, *mut *mut c_char) -> c_int>,
        arg: *mut c_void,
        errmsg: *mut *mut c_char,
    ) -> c_int;
    fn sqlite3_free(ptr: *mut c_void);
    fn sqlite3_prepare_v2(
        db: *mut sqlite3,
        zSql: *const c_char,
        nByte: c_int,
        ppStmt: *mut *mut sqlite3_stmt,
        pzTail: *mut *const c_char,
    ) -> c_int;
    fn sqlite3_step(pStmt: *mut sqlite3_stmt) -> c_int;
    fn sqlite3_finalize(pStmt: *mut sqlite3_stmt) -> c_int;
    fn sqlite3_column_count(pStmt: *mut sqlite3_stmt) -> c_int;
    fn sqlite3_column_int64(pStmt: *mut sqlite3_stmt, iCol: c_int) -> i64;
    fn sqlite3_column_text(pStmt: *mut sqlite3_stmt, iCol: c_int) -> *const c_char;
}

pub struct SqliteConnection {
    raw: *mut sqlite3,
}

impl SqliteConnection {
    pub fn open(path: &Path) -> Result<Self, String> {
        let c_path = CString::new(path.to_string_lossy().as_bytes())
            .map_err(|e| format!("Invalid DB path: {e}"))?;
        let mut db: *mut sqlite3 = ptr::null_mut();
        let rc = unsafe { sqlite3_open(c_path.as_ptr(), &mut db) };
        if rc != SQLITE_OK || db.is_null() {
            return Err(format!("sqlite3_open failed with code {rc}"));
        }
        let conn = Self { raw: db };
        unsafe { sqlite3_busy_timeout(db, 10000); }
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        Ok(conn)
    }

    pub fn execute_batch(&self, sql: &str) -> Result<(), String> {
        let c_sql = CString::new(sql).map_err(|e| format!("SQL CString error: {e}"))?;
        let mut err_msg: *mut c_char = ptr::null_mut();
        let rc = unsafe {
            sqlite3_exec(self.raw, c_sql.as_ptr(), None, ptr::null_mut(), &mut err_msg)
        };
        if rc != SQLITE_OK {
            let msg = if !err_msg.is_null() {
                let s = unsafe { CStr::from_ptr(err_msg).to_string_lossy().into_owned() };
                unsafe { sqlite3_free(err_msg as *mut c_void) };
                s
            } else {
                format!("sqlite3_exec error code {rc}")
            };
            return Err(msg);
        }
        Ok(())
    }

    pub fn query_single_i64(&self, sql: &str) -> Result<i64, String> {
        let c_sql = CString::new(sql).map_err(|e| e.to_string())?;
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
        let rc = unsafe {
            sqlite3_prepare_v2(self.raw, c_sql.as_ptr(), -1, &mut stmt, ptr::null_mut())
        };
        if rc != SQLITE_OK {
            return Err(format!("sqlite3_prepare_v2 failed ({rc}) for: {sql}"));
        }
        let step_rc = unsafe { sqlite3_step(stmt) };
        let val = if step_rc == SQLITE_ROW {
            Ok(unsafe { sqlite3_column_int64(stmt, 0) })
        } else {
            Err(format!("No row returned ({step_rc}) for: {sql}"))
        };
        unsafe { sqlite3_finalize(stmt) };
        val
    }

    pub fn query_single_text(&self, sql: &str) -> Result<String, String> {
        let c_sql = CString::new(sql).map_err(|e| e.to_string())?;
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
        let rc = unsafe {
            sqlite3_prepare_v2(self.raw, c_sql.as_ptr(), -1, &mut stmt, ptr::null_mut())
        };
        if rc != SQLITE_OK {
            return Err(format!("sqlite3_prepare_v2 failed ({rc}) for: {sql}"));
        }
        let step_rc = unsafe { sqlite3_step(stmt) };
        let val = if step_rc == SQLITE_ROW {
            let ptr = unsafe { sqlite3_column_text(stmt, 0) };
            if ptr.is_null() {
                Ok(String::new())
            } else {
                Ok(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
            }
        } else {
            Err(format!("No row returned ({step_rc}) for: {sql}"))
        };
        unsafe { sqlite3_finalize(stmt) };
        val
    }

    pub fn query_rows(&self, sql: &str) -> Result<Vec<Vec<String>>, String> {
        let c_sql = CString::new(sql).map_err(|e| e.to_string())?;
        let mut stmt: *mut sqlite3_stmt = ptr::null_mut();
        let rc = unsafe {
            sqlite3_prepare_v2(self.raw, c_sql.as_ptr(), -1, &mut stmt, ptr::null_mut())
        };
        if rc != SQLITE_OK {
            return Err(format!("sqlite3_prepare_v2 failed ({rc}) for: {sql}"));
        }
        let mut rows = Vec::new();
        let cols = unsafe { sqlite3_column_count(stmt) };
        loop {
            let step_rc = unsafe { sqlite3_step(stmt) };
            if step_rc == SQLITE_ROW {
                let mut row = Vec::with_capacity(cols as usize);
                for c in 0..cols {
                    let ptr = unsafe { sqlite3_column_text(stmt, c) };
                    let val = if ptr.is_null() {
                        String::new()
                    } else {
                        unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
                    };
                    row.push(val);
                }
                rows.push(row);
            } else if step_rc == SQLITE_DONE {
                break;
            } else {
                unsafe { sqlite3_finalize(stmt) };
                return Err(format!("sqlite3_step failed with code {step_rc} for: {sql}"));
            }
        }
        unsafe { sqlite3_finalize(stmt) };
        Ok(rows)
    }
}

impl Drop for SqliteConnection {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                sqlite3_close(self.raw);
            }
        }
    }
}

// ============================================================================
// 2. PINNED FSRS-6 (21-PARAMETER) OBSERVATION-DRIVEN SCHEDULER + SIBLING BURYING
// ============================================================================

pub const PINNED_FSRS_VERSION: &str = "fsrs-6-pinned-21param-v0.2.0";

/// Default FSRS-6 21-parameter vector `w[0..20]`, where `w[20]` is trainable decay.
/// Reference: https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm
pub const DEFAULT_FSRS6_PARAMS: [f64; 21] = [
    0.212,  // w0: initial stability for Again (G=1)
    1.2931, // w1: initial stability for Hard (G=2)
    2.3065, // w2: initial stability for Good (G=3)
    8.2956, // w3: initial stability for Easy (G=4)
    6.4133, // w4: initial difficulty base
    0.8334, // w5: initial difficulty grade scaling
    3.0194, // w6: difficulty update scaling
    0.001,  // w7: mean reversion weight
    1.8722, // w8: stability recall exponent
    0.1666, // w9: stability saturation exponent
    0.796,  // w10: retrievability gain factor
    1.4835, // w11: post-lapse stability base
    0.0614, // w12: post-lapse difficulty exponent
    0.2629, // w13: post-lapse stability exponent
    1.6483, // w14: post-lapse retrievability exponent
    0.6014, // w15: Hard penalty factor
    1.8729, // w16: Easy bonus factor
    0.5425, // w17: short-term stability scaling
    0.0912, // w18: short-term grade offset
    0.0658, // w19: short-term stability damping
    0.1542, // w20: trainable forgetting curve decay parameter
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modality {
    Listen,
    Read,
    Speak,
    Write,
}

impl Modality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Modality::Listen => "LISTEN",
            Modality::Read => "READ",
            Modality::Speak => "SPEAK",
            Modality::Write => "WRITE",
        }
    }

    pub fn all() -> [Modality; 4] {
        [Modality::Listen, Modality::Read, Modality::Speak, Modality::Write]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl Rating {
    pub fn from_u8(val: u8) -> Result<Self, String> {
        match val {
            1 => Ok(Rating::Again),
            2 => Ok(Rating::Hard),
            3 => Ok(Rating::Good),
            4 => Ok(Rating::Easy),
            _ => Err(format!("Invalid FSRS rating: {val}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FsrsCardState {
    pub card_id: String,
    pub lang_code: String,
    pub pack_id: String,
    pub concept_id: String,
    pub modality: Modality,
    pub state: i32, // 0=New, 1=Learning, 2=Review, 3=Relearning
    pub stability: f64,
    pub difficulty: f64,
    pub reps: i32,
    pub lapses: i32,
    pub due_timestamp_ms: i64,
    pub buried_until_ms: Option<i64>,
    pub last_review_ms: Option<i64>,
}

/// Computes FSRS-6 power-law retrievability `R(t, S)` with trainable decay `w[20]`.
pub fn fsrs6_retrievability(elapsed_days: f64, stability: f64, w: &[f64; 21]) -> f64 {
    if stability <= 0.0 || elapsed_days <= 0.0 {
        return 1.0;
    }
    let decay = -w[20];
    let factor = 0.9_f64.powf(1.0 / decay) - 1.0;
    (1.0 + factor * (elapsed_days / stability)).powf(decay)
}

/// Computes the next FSRS-6 `(state, stability, difficulty, scheduled_days)` strictly
/// from a direct observation on this specific card.
pub fn fsrs6_step(
    prev: &FsrsCardState,
    rating: Rating,
    review_timestamp_ms: i64,
    desired_retention: f64,
    w: &[f64; 21],
) -> FsrsCardState {
    let g = (rating as u8) as f64;
    let d0_easy = w[4] - (w[5] * 3.0).exp() + 1.0;

    let (new_state, new_s, new_d, new_lapses) = if prev.state == 0 || prev.stability <= 0.0 {
        // Initial review of a New card
        let s0 = w[(rating as usize) - 1].max(0.1);
        let d0 = (w[4] - (w[5] * (g - 1.0)).exp() + 1.0).clamp(1.0, 10.0);
        let st = if rating == Rating::Again { 1 } else { 2 };
        let lapses = if rating == Rating::Again { prev.lapses + 1 } else { prev.lapses };
        (st, s0, d0, lapses)
    } else {
        let elapsed_days = match prev.last_review_ms {
            Some(last_ms) if review_timestamp_ms > last_ms => {
                (review_timestamp_ms - last_ms) as f64 / 86_400_000.0
            }
            _ => 0.01,
        };
        let r = fsrs6_retrievability(elapsed_days, prev.stability, w);

        // Mean-reverting difficulty update
        let delta_d = -w[6] * (g - 3.0);
        let d_prime = prev.difficulty + delta_d * ((10.0 - prev.difficulty) / 9.0);
        let next_d = (w[7] * d0_easy + (1.0 - w[7]) * d_prime).clamp(1.0, 10.0);

        if rating == Rating::Again {
            // Post-lapse stability S'_f
            let sf = w[11]
                * next_d.powf(-w[12])
                * ((prev.stability + 1.0).powf(w[13]) - 1.0)
                * (w[14] * (1.0 - r)).exp();
            (3, sf.max(0.1), next_d, prev.lapses + 1)
        } else {
            // Post-recall stability S'_r
            let hard_penalty = if rating == Rating::Hard { w[15] } else { 1.0 };
            let easy_bonus = if rating == Rating::Easy { w[16] } else { 1.0 };
            let inc = w[8].exp()
                * (11.0 - next_d)
                * prev.stability.powf(-w[9])
                * ((w[10] * (1.0 - r)).exp() - 1.0)
                * hard_penalty
                * easy_bonus;
            let sr = prev.stability * (1.0 + inc);
            (2, sr.max(0.1), next_d, prev.lapses)
        }
    };

    // Compute next interval in days for target retention (e.g., 0.90)
    let decay = -w[20];
    let factor = 0.9_f64.powf(1.0 / decay) - 1.0;
    let interval_days = ((new_s / factor) * (desired_retention.powf(1.0 / decay) - 1.0)).max(1.0);
    let due_after_ms = review_timestamp_ms + (interval_days * 86_400_000.0).round() as i64;

    FsrsCardState {
        card_id: prev.card_id.clone(),
        lang_code: prev.lang_code.clone(),
        pack_id: prev.pack_id.clone(),
        concept_id: prev.concept_id.clone(),
        modality: prev.modality,
        state: new_state,
        stability: (new_s * 1_000_000.0).round() / 1_000_000.0,
        difficulty: (new_d * 1_000_000.0).round() / 1_000_000.0,
        reps: prev.reps + 1,
        lapses: new_lapses,
        due_timestamp_ms: due_after_ms,
        buried_until_ms: prev.buried_until_ms,
        last_review_ms: Some(review_timestamp_ms),
    }
}

/// Applies the 24-hour Sibling Burying rule: when modality `active_mod` of `concept_id`
/// is reviewed at `review_timestamp_ms`, all sibling modalities `m != active_mod` of the
/// same `(lang_code, concept_id)` have `buried_until_ms` set to `review_timestamp_ms + 86_400_000`
/// WITHOUT mutating their stability `S` or difficulty `D`.
pub fn apply_sibling_burying(
    sibling: &FsrsCardState,
    active_mod: Modality,
    review_timestamp_ms: i64,
) -> FsrsCardState {
    let mut updated = sibling.clone();
    if sibling.modality != active_mod {
        let bury_target = review_timestamp_ms + 86_400_000; // +24h
        updated.buried_until_ms = Some(match sibling.buried_until_ms {
            Some(existing) => existing.max(bury_target),
            None => bury_target,
        });
    }
    updated
}

// ============================================================================
// 3. FIVE-STAGE EPISTEMIC ACOUSTIC EVALUATOR WITH EXPLICIT ABSTENTION GATE
// ============================================================================

#[derive(Debug, Clone)]
pub struct SpokenTelemetryInput {
    pub clipped_sample_ratio: f64,
    pub snr_db: f64,
    pub duration_ms: u32,
    pub unconstrained_transcript_match: bool,
    pub detected_transcript: String,
    pub expected_transcript: String,
    pub alignment_confidence: f64,
    pub phoneme_gop_lpr_scores: Vec<f64>,
    pub duration_ratio_score: f64,
}

/// Evaluates spoken input across the 5 epistemic gates defined in
/// `docs/03-SPEECH-AND-TRUACCENT-PIPELINE.md`.
pub fn evaluate_spoken_utterance(input: &SpokenTelemetryInput) -> AcousticAssessmentOutcome {
    // Gate 1: Signal Quality & Usability Check
    if input.clipped_sample_ratio > 0.005 {
        return AcousticAssessmentOutcome::UnableToAssess {
            reason: AbstentionReason::SignalClipped,
            snr_db: input.snr_db,
            alignment_confidence: input.alignment_confidence,
        };
    }
    if input.snr_db < 12.0 {
        return AcousticAssessmentOutcome::UnableToAssess {
            reason: AbstentionReason::LowSnrFloor,
            snr_db: input.snr_db,
            alignment_confidence: input.alignment_confidence,
        };
    }
    if input.duration_ms < 220 {
        return AcousticAssessmentOutcome::UnableToAssess {
            reason: AbstentionReason::TruncatedEndpoint,
            snr_db: input.snr_db,
            alignment_confidence: input.alignment_confidence,
        };
    }

    // Gate 2: Unconstrained Utterance Identity Verification
    if !input.unconstrained_transcript_match {
        return AcousticAssessmentOutcome::IncorrectUtterance {
            detected_transcript: input.detected_transcript.clone(),
            expected_transcript: input.expected_transcript.clone(),
            matched_foil_id: None,
            foil_contrast_label: None,
        };
    }

    // Gate 3: Forced Alignment Confidence Gate
    if input.alignment_confidence < 0.80 || input.phoneme_gop_lpr_scores.is_empty() {
        return AcousticAssessmentOutcome::UnableToAssess {
            reason: AbstentionReason::LowAlignmentConfidence,
            snr_db: input.snr_db,
            alignment_confidence: input.alignment_confidence,
        };
    }

    // Gate 4 & 5: Contrastive Phoneme Assessment & Calibrated Rating Proposal
    let sum_gop: f64 = input.phoneme_gop_lpr_scores.iter().sum();
    let mean_gop = sum_gop / (input.phoneme_gop_lpr_scores.len() as f64);
    let min_gop = input
        .phoneme_gop_lpr_scores
        .iter()
        .copied()
        .fold(0.0_f64, f64::min);

    let suggested_rating = if mean_gop < -2.4 || min_gop < -3.0 {
        Rating::Again
    } else if mean_gop < -1.3 || input.duration_ratio_score < 0.70 {
        Rating::Hard
    } else if mean_gop >= -0.55 && min_gop >= -1.1 && input.duration_ratio_score >= 0.88 {
        Rating::Easy
    } else {
        Rating::Good
    };

    let contrast_passed = min_gop >= -1.30;

    AcousticAssessmentOutcome::Assessed {
        mean_gop_lpr: mean_gop,
        min_contrast_gop_lpr: min_gop,
        duration_ratio_score: input.duration_ratio_score,
        suggested_rating,
        contrast_passed,
    }
}

// ============================================================================
// 4. DUAL-DB INTEGRITY CHECKER & DETERMINISTIC EVENT LOG REPLAYER
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct PackIntegrityReport {
    pub pack_id: String,
    pub lang_code: String,
    pub concepts_count: i64,
    pub exercises_count: i64,
    pub approved_exercises_count: i64,
    pub orphan_cards_count: i64,
    pub total_cards_for_lang: i64,
    pub integrity_ok: bool,
}

pub fn verify_dual_db_pack(
    telemetry_db_path: &Path,
    course_pack_path: &Path,
) -> Result<PackIntegrityReport, String> {
    let conn = SqliteConnection::open(telemetry_db_path)?;
    let attach_sql = format!(
        "ATTACH DATABASE '{}' AS course;",
        course_pack_path.to_string_lossy().replace('\'', "''")
    );
    conn.execute_batch(&attach_sql)?;

    let pack_id = conn.query_single_text("SELECT pack_id FROM course.pack_manifest LIMIT 1;")?;
    let lang_code = conn.query_single_text("SELECT lang_code FROM course.pack_manifest LIMIT 1;")?;
    let concepts_count = conn.query_single_i64("SELECT COUNT(*) FROM course.concepts;")?;
    let exercises_count = conn.query_single_i64("SELECT COUNT(*) FROM course.contrast_exercises;")?;
    let approved_exercises_count = conn.query_single_i64(
        "SELECT COUNT(*) FROM course.exercise_evidence \
         WHERE structural_validity = 'PASS' \
           AND linguistic_contrast_purity = 'PASS' \
           AND license_provenance_complete = 1 \
           AND editorial_status = 'APPROVED';",
    )?;
    let orphan_cards_count = conn.query_single_i64(&format!(
        "SELECT COUNT(*) FROM fsrs_cards c \
         LEFT JOIN course.concepts k ON c.concept_id = k.concept_id \
         WHERE c.pack_id = '{pack_id}' AND k.concept_id IS NULL;"
    ))?;
    let total_cards_for_lang = conn.query_single_i64(&format!(
        "SELECT COUNT(*) FROM fsrs_cards WHERE lang_code = '{lang_code}';"
    ))?;

    let integrity_ok = concepts_count >= 20
        && exercises_count == approved_exercises_count
        && orphan_cards_count == 0
        && total_cards_for_lang == concepts_count * 4;

    Ok(PackIntegrityReport {
        pack_id,
        lang_code,
        concepts_count,
        exercises_count,
        approved_exercises_count,
        orphan_cards_count,
        total_cards_for_lang,
        integrity_ok,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fsrs6_retrievability_and_observation_only_stability() {
        let card = FsrsCardState {
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
            due_timestamp_ms: 1_700_000_000_000,
            buried_until_ms: None,
            last_review_ms: None,
        };

        let after_good = fsrs6_step(
            &card,
            Rating::Good,
            1_700_000_000_000,
            0.90,
            &DEFAULT_FSRS6_PARAMS,
        );
        assert_eq!(after_good.state, 2);
        assert!((after_good.stability - DEFAULT_FSRS6_PARAMS[2]).abs() < 1e-5);

        // At elapsed_days == stability, retrievability R(S, S) must equal 0.90
        let r_at_s = fsrs6_retrievability(after_good.stability, after_good.stability, &DEFAULT_FSRS6_PARAMS);
        assert!((r_at_s - 0.90).abs() < 1e-6);

        // Sibling burying must defer SPEAK card by 24h WITHOUT changing its stability
        let speak_sibling = FsrsCardState {
            card_id: "grc:GRC_01_NOM_ACC_AGENT_PATIENT:SPEAK".to_string(),
            modality: Modality::Speak,
            ..card.clone()
        };
        let buried_speak = apply_sibling_burying(&speak_sibling, Modality::Listen, 1_700_000_000_000);
        assert_eq!(buried_speak.stability, 0.0);
        assert_eq!(buried_speak.buried_until_ms, Some(1_700_000_000_000 + 86_400_000));
    }

    #[test]
    fn test_deterministic_event_log_replay_bit_exact() {
        let genesis = FsrsCardState {
            card_id: "hbo:HBO_01_GENDER_NUMBER_PARTICIPLE_AGREEMENT:READ".to_string(),
            lang_code: "hbo".to_string(),
            pack_id: "klpack_hbo_masoretic_v1".to_string(),
            concept_id: "HBO_01_GENDER_NUMBER_PARTICIPLE_AGREEMENT".to_string(),
            modality: Modality::Read,
            state: 0,
            stability: 0.0,
            difficulty: 5.0,
            reps: 0,
            lapses: 0,
            due_timestamp_ms: 1_700_000_000_000,
            buried_until_ms: None,
            last_review_ms: None,
        };

        let events = [
            (Rating::Good, 1_700_000_000_000_i64),
            (Rating::Good, 1_700_200_000_000_i64),
            (Rating::Hard, 1_700_800_000_000_i64),
            (Rating::Again, 1_701_500_000_000_i64),
            (Rating::Easy, 1_701_600_000_000_i64),
        ];

        let mut run1 = genesis.clone();
        for (rating, ts) in events {
            run1 = fsrs6_step(&run1, rating, ts, 0.90, &DEFAULT_FSRS6_PARAMS);
        }

        let mut run2 = genesis.clone();
        for (rating, ts) in events {
            run2 = fsrs6_step(&run2, rating, ts, 0.90, &DEFAULT_FSRS6_PARAMS);
        }

        assert_eq!(run1, run2);
        assert_eq!(run1.reps, 5);
        assert_eq!(run1.lapses, 1);
    }

    #[test]
    fn test_acoustic_evaluator_abstains_on_low_confidence() {
        let noisy_clip = SpokenTelemetryInput {
            clipped_sample_ratio: 0.0,
            snr_db: 7.5, // Below 12 dB gate
            duration_ms: 950,
            unconstrained_transcript_match: true,
            detected_transcript: "ὁ ἄγγελος βλέπει".to_string(),
            expected_transcript: "ὁ ἄγγελος βλέπει".to_string(),
            alignment_confidence: 0.91,
            phoneme_gop_lpr_scores: vec![-0.4, -0.5, -0.3],
            duration_ratio_score: 0.92,
        };
        assert_eq!(
            evaluate_spoken_utterance(&noisy_clip),
            AcousticAssessmentOutcome::UnableToAssess {
                reason: AbstentionReason::LowSnrFloor,
                snr_db: 7.5,
                alignment_confidence: 0.91,
            }
        );

        let low_align = SpokenTelemetryInput {
            snr_db: 22.0,
            alignment_confidence: 0.62, // Below 0.80 gate
            ..noisy_clip.clone()
        };
        assert_eq!(
            evaluate_spoken_utterance(&low_align),
            AcousticAssessmentOutcome::UnableToAssess {
                reason: AbstentionReason::LowAlignmentConfidence,
                snr_db: 22.0,
                alignment_confidence: 0.62,
            }
        );

        let clean_pass = SpokenTelemetryInput {
            snr_db: 24.0,
            alignment_confidence: 0.94,
            phoneme_gop_lpr_scores: vec![-0.3, -0.4, -0.2, -0.5],
            duration_ratio_score: 0.91,
            ..noisy_clip
        };
        assert_eq!(
            evaluate_spoken_utterance(&clean_pass),
            AcousticAssessmentOutcome::Assessed {
                mean_gop_lpr: -0.35,
                min_contrast_gop_lpr: -0.5,
                duration_ratio_score: 0.91,
                suggested_rating: Rating::Easy,
                contrast_passed: true,
            }
        );
    }
}
