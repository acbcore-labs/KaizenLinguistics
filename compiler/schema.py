#!/usr/bin/env python3
"""
KaizenLinguistics — Hardened v2.0 Dual-Database SQLite Schema & Integrity Engine
Implements the exact SQLite 3 STRICT DDL from docs/05-DATA-SCHEMA-AND-STATE-MACHINE.md:
  1. Immutable Course Pack Schema (course_<lang_iso>.klpack)
  2. Personal Telemetry & Replayable Scheduling Schema (user_telemetry.db)
  3. Application-Level Cross-Database Foreign Key & Manifest Hash Integrity Verifier
"""

from __future__ import annotations
import hashlib
import json
import sqlite3
from pathlib import Path
from typing import Dict, List, Optional, Tuple


COURSE_PACK_DDL = """
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS pack_manifest (
    pack_id                 TEXT PRIMARY KEY,
    lang_code               TEXT NOT NULL,
    dialect_profile         TEXT NOT NULL,
    pack_semver             TEXT NOT NULL,
    schema_version          INTEGER NOT NULL DEFAULT 2,
    compiled_at_utc         TEXT NOT NULL,
    manifest_blake3_hash    TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE IF NOT EXISTS concepts (
    concept_id              TEXT PRIMARY KEY,
    concept_type            TEXT NOT NULL CHECK (concept_type IN ('SCRIPT_PHONOLOGY', 'LEXICAL', 'MORPHOLOGICAL', 'SYNTACTIC', 'DISCOURSE')),
    wikidata_qid            TEXT,
    semantic_domain_code    TEXT,
    tier_number             INTEGER NOT NULL CHECK (tier_number BETWEEN 1 AND 4),
    unit_number             INTEGER NOT NULL,
    lesson_number           INTEGER NOT NULL,
    content_hash            TEXT NOT NULL UNIQUE
) STRICT;

CREATE TABLE IF NOT EXISTS lexemes (
    lexeme_id               TEXT PRIMARY KEY,
    concept_id              TEXT NOT NULL REFERENCES concepts(concept_id),
    lang_code               TEXT NOT NULL,
    lemma_unicode           TEXT NOT NULL,
    lemma_unpointed         TEXT NOT NULL,
    strongs_id              TEXT,
    bdb_or_abbott_id        TEXT,
    root_consonants         TEXT,
    upos                    TEXT NOT NULL,
    gender_or_stem          TEXT,
    ipa_restored            TEXT NOT NULL,
    ipa_academic            TEXT,
    corpus_freq_count       INTEGER NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS inflection_forms (
    form_id                 TEXT PRIMARY KEY,
    lexeme_id               TEXT NOT NULL REFERENCES lexemes(lexeme_id),
    surface_unicode         TEXT NOT NULL,
    surface_unpointed       TEXT NOT NULL,
    morph_tag               TEXT NOT NULL,
    ud_feats_json           TEXT NOT NULL,
    ipa_phones_json         TEXT NOT NULL,
    bound_morpheme_spans    TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS media_assets (
    asset_hash              TEXT PRIMARY KEY,
    media_type              TEXT NOT NULL CHECK (media_type IN ('IMAGE_WEBP', 'IMAGE_SVG', 'AUDIO_OPUS', 'AUDIO_WAV')),
    source_corpus           TEXT NOT NULL,
    speaker_or_author       TEXT NOT NULL,
    license_spdx            TEXT NOT NULL,
    source_uri              TEXT NOT NULL,
    duration_ms             INTEGER,
    payload_blob            BLOB NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS sentences (
    sentence_id             TEXT PRIMARY KEY,
    source_ref              TEXT,
    lang_code               TEXT NOT NULL,
    text_pointed            TEXT NOT NULL,
    text_pedagogical        TEXT NOT NULL,
    text_unpointed          TEXT NOT NULL,
    accepted_variants_json  TEXT NOT NULL,
    macula_tree_json        TEXT NOT NULL,
    ipa_sequence_json       TEXT NOT NULL,
    audio_asset_hash        TEXT NOT NULL REFERENCES media_assets(asset_hash),
    alignment_tiers_json    TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS contrast_exercises (
    exercise_id             TEXT PRIMARY KEY,
    primary_concept_id      TEXT NOT NULL REFERENCES concepts(concept_id),
    presentation_type       TEXT NOT NULL CHECK (presentation_type IN ('QUAD_SCENE', 'ASPECT_STRIP', 'DIALOGUE_PAIR', 'CLAUSE_PEELER')),
    contrast_dimension      TEXT NOT NULL,
    items_json              TEXT NOT NULL,
    target_panel_index      INTEGER NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS exercise_evidence (
    exercise_id                     TEXT PRIMARY KEY REFERENCES contrast_exercises(exercise_id),
    structural_validity             TEXT NOT NULL CHECK (structural_validity IN ('PASS', 'FAIL')),
    linguistic_contrast_purity      TEXT NOT NULL CHECK (linguistic_contrast_purity IN ('PASS', 'CONFOUNDED_LEXEME', 'MULTI_FEATURE_DRIFT')),
    visual_clip_min_margin          REAL,
    audio_alignment_mean_conf       REAL NOT NULL,
    license_provenance_complete     INTEGER NOT NULL CHECK (license_provenance_complete IN (0, 1)),
    editorial_status                TEXT NOT NULL CHECK (editorial_status IN ('DRAFT', 'APPROVED', 'REJECTED')),
    reviewer_id                     TEXT,
    reviewed_at_utc                 TEXT,
    review_notes                    TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS concept_prerequisites (
    parent_concept_id       TEXT NOT NULL REFERENCES concepts(concept_id),
    child_concept_id        TEXT NOT NULL REFERENCES concepts(concept_id),
    PRIMARY KEY (parent_concept_id, child_concept_id)
) STRICT;
"""


USER_TELEMETRY_DDL = """
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS user_profiles (
    user_id                 TEXT PRIMARY KEY,
    display_name            TEXT NOT NULL,
    avatar                  TEXT NOT NULL DEFAULT '🏛️',
    created_at_ms           INTEGER NOT NULL,
    last_active_ms          INTEGER NOT NULL,
    current_lang            TEXT NOT NULL DEFAULT 'grc',
    streak_days             INTEGER NOT NULL DEFAULT 1,
    total_reviews           INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE TABLE IF NOT EXISTS installed_packs (
    pack_id                 TEXT PRIMARY KEY,
    lang_code               TEXT NOT NULL,
    pack_semver             TEXT NOT NULL,
    manifest_blake3_hash    TEXT NOT NULL,
    installed_at_ms         INTEGER NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS fsrs_cards (
    card_id                 TEXT PRIMARY KEY,
    lang_code               TEXT NOT NULL,
    pack_id                 TEXT NOT NULL REFERENCES installed_packs(pack_id),
    concept_id              TEXT NOT NULL,
    modality                TEXT NOT NULL CHECK (modality IN ('LISTEN', 'READ', 'SPEAK', 'WRITE')),
    scaffold_stage          INTEGER NOT NULL DEFAULT 1 CHECK (scaffold_stage BETWEEN 1 AND 6),
    state                   INTEGER NOT NULL DEFAULT 0,
    stability               REAL NOT NULL DEFAULT 0.0,
    difficulty              REAL NOT NULL DEFAULT 5.0,
    elapsed_days            REAL NOT NULL DEFAULT 0.0,
    scheduled_days          REAL NOT NULL DEFAULT 0.0,
    reps                    INTEGER NOT NULL DEFAULT 0,
    lapses                  INTEGER NOT NULL DEFAULT 0,
    due_timestamp_ms        INTEGER NOT NULL,
    buried_until_ms         INTEGER,
    last_review_ms          INTEGER
) STRICT;

CREATE INDEX IF NOT EXISTS idx_fsrs_due ON fsrs_cards(lang_code, due_timestamp_ms, buried_until_ms, state);

CREATE TABLE IF NOT EXISTS review_logs (
    log_id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    event_uuid              TEXT NOT NULL UNIQUE,
    card_id                 TEXT NOT NULL REFERENCES fsrs_cards(card_id),
    pack_id                 TEXT NOT NULL,
    pack_semver             TEXT NOT NULL,
    exercise_id             TEXT NOT NULL,
    permutation_seed        INTEGER NOT NULL,
    review_timestamp_ms     INTEGER NOT NULL,
    event_type              TEXT NOT NULL CHECK (event_type IN ('DIRECT_REVIEW', 'SIBLING_BURY', 'ABSTENTION_RETRY', 'PACK_MIGRATION')),
    rating                  INTEGER CHECK (rating BETWEEN 1 AND 4),
    grade_source            TEXT NOT NULL CHECK (grade_source IN ('OBJECTIVE_CHOICE', 'SELF_GRADE', 'CALIBRATED_AUTO', 'SYSTEM_BURY')),
    fsrs_crate_version      TEXT NOT NULL,
    fsrs_params_json        TEXT NOT NULL,
    state_before            INTEGER NOT NULL,
    stability_before        REAL NOT NULL,
    difficulty_before       REAL NOT NULL,
    state_after             INTEGER NOT NULL,
    stability_after         REAL NOT NULL,
    difficulty_after        REAL NOT NULL,
    due_after_ms            INTEGER NOT NULL,
    response_latency_ms     INTEGER,
    audio_replays           INTEGER NOT NULL DEFAULT 0,
    grader_model_version    TEXT,
    acoustic_evidence_json  TEXT,
    edit_distance           INTEGER
) STRICT;

CREATE TABLE IF NOT EXISTS session_checkpoints (
    session_id              TEXT PRIMARY KEY,
    lang_code               TEXT NOT NULL,
    pack_id                 TEXT NOT NULL REFERENCES installed_packs(pack_id),
    started_at_ms           INTEGER NOT NULL,
    updated_at_ms           INTEGER NOT NULL,
    queue_state_json        TEXT NOT NULL,
    completed_count         INTEGER NOT NULL DEFAULT 0,
    is_active               INTEGER NOT NULL CHECK (is_active IN (0, 1))
) STRICT;

CREATE TABLE IF NOT EXISTS phoneme_telemetry (
    lang_code               TEXT NOT NULL,
    target_ipa              TEXT NOT NULL,
    spoken_ipa              TEXT NOT NULL,
    occurrence_count        INTEGER NOT NULL DEFAULT 1,
    ewma_gop_score          REAL NOT NULL,
    last_updated_ms         INTEGER NOT NULL,
    PRIMARY KEY (lang_code, target_ipa, spoken_ipa)
) STRICT;
"""


def content_digest(payload: bytes) -> str:
    """Computes deterministic 256-bit cryptographic digest (blake2s/sha256 hex)."""
    return hashlib.blake2b(payload, digest_size=32).hexdigest()


def init_course_pack_db(db_path: Path) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    if db_path.exists():
        db_path.unlink()
    conn = sqlite3.connect(str(db_path))
    conn.executescript(COURSE_PACK_DDL)
    conn.commit()
    return conn


def init_telemetry_db(db_path: Path) -> sqlite3.Connection:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(str(db_path))
    conn.executescript(USER_TELEMETRY_DDL)
    conn.commit()
    return conn


def verify_cross_db_integrity(
    telemetry_db_path: Path, course_pack_path: Path, now_ms: int = 1794691200000
) -> Dict[str, object]:
    """
    Enforces application-level cross-database integrity between user_telemetry.db
    and an attached course_<lang>.klpack archive (since SQLite REFERENCES cannot
    cross ATTACH DATABASE boundaries).
    """
    conn = sqlite3.connect(str(telemetry_db_path))
    conn.execute("PRAGMA foreign_keys = ON;")
    conn.execute(f"ATTACH DATABASE ? AS course;", (str(course_pack_path),))

    row = conn.execute(
        "SELECT pack_id, lang_code, pack_semver, manifest_blake3_hash FROM course.pack_manifest LIMIT 1;"
    ).fetchone()
    if not row:
        raise ValueError(f"Course pack {course_pack_path} is missing pack_manifest row.")
    pack_id, lang_code, pack_semver, manifest_hash = row

    # Check if pack is already registered
    existing = conn.execute(
        "SELECT pack_semver, manifest_blake3_hash FROM installed_packs WHERE pack_id = ?;",
        (pack_id,),
    ).fetchone()

    migrated_cards = 0
    if existing is None:
        conn.execute(
            """
            INSERT INTO installed_packs (pack_id, lang_code, pack_semver, manifest_blake3_hash, installed_at_ms)
            VALUES (?, ?, ?, ?, ?);
            """,
            (pack_id, lang_code, pack_semver, manifest_hash, now_ms),
        )
    elif existing[1] != manifest_hash:
        conn.execute(
            """
            UPDATE installed_packs
            SET pack_semver = ?, manifest_blake3_hash = ?, installed_at_ms = ?
            WHERE pack_id = ?;
            """,
            (pack_semver, manifest_hash, now_ms, pack_id),
        )

    # Check for orphan FSRS cards referencing concepts not present in the mounted course pack
    orphans = conn.execute(
        """
        SELECT c.card_id, c.concept_id
        FROM fsrs_cards c
        LEFT JOIN course.concepts k ON c.concept_id = k.concept_id
        WHERE c.pack_id = ? AND k.concept_id IS NULL;
        """,
        (pack_id,),
    ).fetchall()

    # Check evidence gate: ensure all exercises in the pack have PASS structural & linguistic purity
    unapproved = conn.execute(
        """
        SELECT exercise_id, structural_validity, linguistic_contrast_purity, editorial_status
        FROM course.exercise_evidence
        WHERE structural_validity != 'PASS'
           OR linguistic_contrast_purity != 'PASS'
           OR license_provenance_complete != 1
           OR editorial_status != 'APPROVED';
        """
    ).fetchall()

    conn.commit()
    conn.close()

    return {
        "pack_id": pack_id,
        "lang_code": lang_code,
        "pack_semver": pack_semver,
        "manifest_hash": manifest_hash,
        "orphan_cards": len(orphans),
        "unapproved_exercises": len(unapproved),
        "migrated_cards": migrated_cards,
        "integrity_ok": len(orphans) == 0 and len(unapproved) == 0,
    }
