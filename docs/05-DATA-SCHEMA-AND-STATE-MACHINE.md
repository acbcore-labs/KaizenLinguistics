# 05 — Deterministic Database Schema, Replayable Event Log & Lesson State Machine (v2.0 — Hardened)

**Project:** KaizenLinguistics  
**Primary Languages:** Koine Greek (`grc`) & Classical / Biblical Hebrew (`hbo`)  
**Storage Engine:** Dual SQLite 3 (`WAL` Mode, `STRICT` Tables, Application-Enforced Cross-DB Integrity)  

---

## 1. Resolving Dual-Database Boundary & Deterministic Replay Requirements

Architectural review identified two critical database and state-reproducibility gaps in v1.0:
1. **Cross-Database Foreign Keys in SQLite:** Per the official [SQLite Foreign Key Specification](https://www.sqlite.org/foreignkeys.html), SQLite `REFERENCES` constraints **cannot cross attached database boundaries** (`course_<lang>.klpack` $\leftrightarrow$ `user_telemetry.db`). Relying on SQLite DDL alone across `ATTACH DATABASE` would fail or leave dangling references.
2. **Incomplete Event Log for Deterministic Replay:** Replaying `review_logs` to deterministically reconstruct `fsrs_cards` requires logging the exact **scheduler version**, **FSRS parameter vector `w`**, **course pack ID & version**, **exercise permutation seed**, **acoustic/grader model version**, and **explicit sibling-burying events**. Furthermore, `fsrs_cards` must be scoped by `lang_code` and `pack_id` so shared cross-lingual `concept_id` keys (`Q726`) never collide between Koine Greek (`grc`) and Biblical Hebrew (`hbo`).

In v2.0, both issues are resolved explicitly in the schema and Rust storage layer.

---

## 2. Hardened SQLite DDL Schema Specification

### 2.1 Immutable Course Pack Schema (`course_<lang_iso>.klpack`)

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

-- 0. Course Pack Manifest & Version Identity
CREATE TABLE pack_manifest (
    pack_id                 TEXT PRIMARY KEY,           -- e.g., 'klpack_grc_koine_v1' or 'klpack_hbo_masoretic_v1'
    lang_code               TEXT NOT NULL,              -- ISO 639-3: 'grc' or 'hbo'
    dialect_profile         TEXT NOT NULL,              -- e.g., 'RESTORED_KOINE_1C', 'ERASMIAN', 'TIBERIAN_MASORETIC'
    pack_semver             TEXT NOT NULL,              -- e.g., '0.1.0'
    schema_version          INTEGER NOT NULL DEFAULT 2,
    compiled_at_utc         TEXT NOT NULL,
    manifest_blake3_hash    TEXT NOT NULL UNIQUE
) STRICT;

-- 1. Contrast-Graph Concept & Grammatical Nodes
CREATE TABLE concepts (
    concept_id              TEXT PRIMARY KEY,           -- e.g., 'GRC_NOM_ACC_CASE' or 'HBO_QAL_PARTICIPLE_SG_PL'
    concept_type            TEXT NOT NULL CHECK (concept_type IN ('SCRIPT_PHONOLOGY', 'LEXICAL', 'MORPHOLOGICAL', 'SYNTACTIC', 'DISCOURSE')),
    wikidata_qid            TEXT,                       -- Optional cross-lingual Q-ID anchor
    semantic_domain_code    TEXT,                       -- Louw-Nida (Greek) or UBS SDBH (Hebrew) domain code
    tier_number             INTEGER NOT NULL CHECK (tier_number BETWEEN 1 AND 4),
    unit_number             INTEGER NOT NULL,
    lesson_number           INTEGER NOT NULL,
    content_hash            TEXT NOT NULL UNIQUE
) STRICT;

-- 2. Target-Language Lexemes (MorphGNT / Abbott-Smith / OpenScriptures BDB / MACULA)
CREATE TABLE lexemes (
    lexeme_id               TEXT PRIMARY KEY,
    concept_id              TEXT NOT NULL REFERENCES concepts(concept_id),
    lang_code               TEXT NOT NULL,              -- 'grc' or 'hbo'
    lemma_unicode           TEXT NOT NULL,              -- Polytonic Greek or Pointed Hebrew headword (NFC normalized)
    lemma_unpointed         TEXT NOT NULL,              -- Consonantal Hebrew or stripped Greek for search
    strongs_id              TEXT,                       -- e.g., 'G3056' (logos) or 'H1697' (davar)
    bdb_or_abbott_id        TEXT,                       -- Canonical lexicon XML ID
    root_consonants         TEXT,                       -- Triliteral root for Hebrew (e.g., 'דבר', 'שׁמר')
    upos                    TEXT NOT NULL,              -- 'NOUN', 'VERB', 'ADJ', 'ADP', 'PART', etc.
    gender_or_stem          TEXT,                       -- Greek gender or Hebrew Binyan ('Qal', 'Piel', 'Hiphil', etc.)
    ipa_restored            TEXT NOT NULL,              -- Primary reconstructed IPA transcription
    ipa_academic            TEXT,                       -- Optional Erasmian / Academic IPA variant
    corpus_freq_count       INTEGER NOT NULL            -- Exact occurrence count in SBLGNT / WLC
) STRICT;

-- 3. Inflected Surface Forms & Morpheme Span Annotations
CREATE TABLE inflection_forms (
    form_id                 TEXT PRIMARY KEY,
    lexeme_id               TEXT NOT NULL REFERENCES lexemes(lexeme_id),
    surface_unicode         TEXT NOT NULL,              -- Exact pointed/accented token
    surface_unpointed       TEXT NOT NULL,
    morph_tag               TEXT NOT NULL,              -- MorphGNT parsing code or OSHB/MACULA morph tag
    ud_feats_json           TEXT NOT NULL,              -- JSON map of grammatical features
    ipa_phones_json         TEXT NOT NULL,              -- JSON array of expected IPA phones
    bound_morpheme_spans    TEXT NOT NULL               -- JSON character offsets of prefix/suffix/case ending for UI highlighting
) STRICT;

-- 4. Content-Addressed Media Assets with Mandatory License Provenance
CREATE TABLE media_assets (
    asset_hash              TEXT PRIMARY KEY,           -- BLAKE3 hex digest of binary payload
    media_type              TEXT NOT NULL CHECK (media_type IN ('IMAGE_WEBP', 'IMAGE_SVG', 'AUDIO_OPUS')),
    source_corpus           TEXT NOT NULL,              -- 'ALEPH_WITH_BETH', 'ALPHA_WITH_ANGELA', 'MACULA', 'OPEN_IMAGES_V7', 'TTS_SYNTH'
    speaker_or_author       TEXT NOT NULL,
    license_spdx            TEXT NOT NULL,              -- 'CC0-1.0', 'CC-BY-4.0', 'CC-BY-SA-4.0', 'Public-Domain'
    source_uri              TEXT NOT NULL,
    duration_ms             INTEGER,
    payload_blob            BLOB NOT NULL
) STRICT;

-- 5. Target Clauses / Sentences with Syntactic Trees & Accepted Variants
CREATE TABLE sentences (
    sentence_id             TEXT PRIMARY KEY,
    source_ref              TEXT,                       -- e.g., 'SBLGNT:John.1.1' or 'WLC:Gen.1.1' or 'CURATED:GRC_001'
    lang_code               TEXT NOT NULL,
    text_pointed            TEXT NOT NULL,              -- Full polytonic or Masoretic pointed/cantillated text
    text_pedagogical        TEXT NOT NULL,              -- Pointed text without cantillation marks
    text_unpointed          TEXT NOT NULL,              -- Consonantal / unpointed text
    accepted_variants_json  TEXT NOT NULL,              -- JSON list of acceptable word-order / orthographic / allophonic variants
    macula_tree_json        TEXT NOT NULL,              -- Clause hierarchy, semantic roles (Agent/Patient), and referent links
    ipa_sequence_json       TEXT NOT NULL,              -- Canonical IPA sequence for active phonological profile
    audio_asset_hash        TEXT NOT NULL REFERENCES media_assets(asset_hash),
    alignment_tiers_json    TEXT NOT NULL               -- Word and phone [t_start_ms, t_end_ms] intervals + confidence
) STRICT;

-- 6. Multi-Modal Contrast Exercises (Supporting 2x2 Quads, Aspect Strips & Dialogue Pairs)
CREATE TABLE contrast_exercises (
    exercise_id             TEXT PRIMARY KEY,
    primary_concept_id      TEXT NOT NULL REFERENCES concepts(concept_id),
    presentation_type       TEXT NOT NULL CHECK (presentation_type IN ('QUAD_SCENE', 'ASPECT_STRIP', 'DIALOGUE_PAIR', 'CLAUSE_PEELER')),
    contrast_dimension      TEXT NOT NULL,              -- e.g., 'CASE_NOM_VS_ACC', 'BINYAN_QAL_VS_HIPHIL', 'ASPECT_PRES_VS_AOR'
    items_json              TEXT NOT NULL,              -- JSON array of panel objects [{sentence_id, image_hash, role, highlight_spans}]
    target_panel_index      INTEGER NOT NULL
) STRICT;

-- 7. Six-Dimensional Evidence & Editorial Sign-Off Table (Per Exercise)
CREATE TABLE exercise_evidence (
    exercise_id                     TEXT PRIMARY KEY REFERENCES contrast_exercises(exercise_id),
    structural_validity             TEXT NOT NULL CHECK (structural_validity IN ('PASS', 'FAIL')),
    linguistic_contrast_purity      TEXT NOT NULL CHECK (linguistic_contrast_purity IN ('PASS', 'CONFOUNDED_LEXEME', 'MULTI_FEATURE_DRIFT')),
    visual_clip_min_margin          REAL,               -- Heuristic MobileCLIP cosine margin (nullable for non-image exercises)
    audio_alignment_mean_conf       REAL NOT NULL,      -- MFA / Wav2Vec2 alignment confidence (0.0..1.0)
    license_provenance_complete     INTEGER NOT NULL CHECK (license_provenance_complete IN (0, 1)),
    editorial_status                TEXT NOT NULL CHECK (editorial_status IN ('DRAFT', 'APPROVED', 'REJECTED')),
    reviewer_id                     TEXT,
    reviewed_at_utc                 TEXT,
    review_notes                    TEXT
) STRICT;

-- 8. Concept Prerequisite DAG
CREATE TABLE concept_prerequisites (
    parent_concept_id       TEXT NOT NULL REFERENCES concepts(concept_id),
    child_concept_id        TEXT NOT NULL REFERENCES concepts(concept_id),
    PRIMARY KEY (parent_concept_id, child_concept_id)
) STRICT;
```

---

### 2.2 Personal Telemetry & Replayable Scheduling Schema (`user_telemetry.db`)

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

-- 1. Installed Course Pack Registry (Used for App-Level Cross-DB Integrity Checks)
CREATE TABLE installed_packs (
    pack_id                 TEXT PRIMARY KEY,
    lang_code               TEXT NOT NULL,
    pack_semver             TEXT NOT NULL,
    manifest_blake3_hash    TEXT NOT NULL,
    installed_at_ms         INTEGER NOT NULL
) STRICT;

-- 2. Independent Per-Modality FSRS Cards (Scoped by lang_code and concept_id)
CREATE TABLE fsrs_cards (
    card_id                 TEXT PRIMARY KEY,           -- lang_code || ':' || concept_id || ':' || modality
    lang_code               TEXT NOT NULL,              -- 'grc' or 'hbo'
    pack_id                 TEXT NOT NULL REFERENCES installed_packs(pack_id),
    concept_id              TEXT NOT NULL,              -- Verified against course DB on session boot
    modality                TEXT NOT NULL CHECK (modality IN ('LISTEN', 'READ', 'SPEAK', 'WRITE')),
    scaffold_stage          INTEGER NOT NULL DEFAULT 1 CHECK (scaffold_stage BETWEEN 1 AND 6),
    state                   INTEGER NOT NULL DEFAULT 0, -- 0=New, 1=Learning, 2=Review, 3=Relearning
    stability               REAL NOT NULL DEFAULT 0.0,  -- Updated ONLY from direct observations of this modality
    difficulty              REAL NOT NULL DEFAULT 5.0,
    elapsed_days            REAL NOT NULL DEFAULT 0.0,
    scheduled_days          REAL NOT NULL DEFAULT 0.0,
    reps                    INTEGER NOT NULL DEFAULT 0,
    lapses                  INTEGER NOT NULL DEFAULT 0,
    due_timestamp_ms        INTEGER NOT NULL,
    buried_until_ms         INTEGER,                    -- Set by Sibling Burying rule without altering stability
    last_review_ms          INTEGER
) STRICT;

CREATE INDEX idx_fsrs_due ON fsrs_cards(lang_code, due_timestamp_ms, buried_until_ms, state);

-- 3. Fully Replayable Append-Only Event Log
CREATE TABLE review_logs (
    log_id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    event_uuid              TEXT NOT NULL UNIQUE,
    card_id                 TEXT NOT NULL REFERENCES fsrs_cards(card_id),
    pack_id                 TEXT NOT NULL,
    pack_semver             TEXT NOT NULL,
    exercise_id             TEXT NOT NULL,
    permutation_seed        INTEGER NOT NULL,           -- Deterministic RNG seed for panel order
    review_timestamp_ms     INTEGER NOT NULL,
    event_type              TEXT NOT NULL CHECK (event_type IN ('DIRECT_REVIEW', 'SIBLING_BURY', 'ABSTENTION_RETRY', 'PACK_MIGRATION')),
    rating                  INTEGER CHECK (rating BETWEEN 1 AND 4), -- NULL if event_type == 'ABSTENTION_RETRY' or 'SIBLING_BURY'
    grade_source            TEXT NOT NULL CHECK (grade_source IN ('OBJECTIVE_CHOICE', 'SELF_GRADE', 'CALIBRATED_AUTO', 'SYSTEM_BURY')),
    fsrs_crate_version      TEXT NOT NULL,              -- e.g., 'fsrs-rs-1.3.5'
    fsrs_params_json        TEXT NOT NULL,              -- Exact 19- or 21-float parameter array used
    state_before            INTEGER NOT NULL,
    stability_before        REAL NOT NULL,
    difficulty_before       REAL NOT NULL,
    state_after             INTEGER NOT NULL,
    stability_after         REAL NOT NULL,
    difficulty_after        REAL NOT NULL,
    due_after_ms            INTEGER NOT NULL,
    response_latency_ms     INTEGER,
    audio_replays           INTEGER NOT NULL DEFAULT 0,
    grader_model_version    TEXT,                       -- e.g., 'wav2vec2-xlsr-53-onnx-v1'
    acoustic_evidence_json  TEXT,                       -- JSON: {snr_db, alignment_conf, gop_scores, abstention_reason}
    edit_distance           INTEGER
) STRICT;

-- 4. Restart-Safe Active Session Checkpoint
CREATE TABLE session_checkpoints (
    session_id              TEXT PRIMARY KEY,
    lang_code               TEXT NOT NULL,
    pack_id                 TEXT NOT NULL REFERENCES installed_packs(pack_id),
    started_at_ms           INTEGER NOT NULL,
    updated_at_ms           INTEGER NOT NULL,
    queue_state_json        TEXT NOT NULL,              -- Remaining ordered queue of (card_id, exercise_id, permutation_seed)
    completed_count         INTEGER NOT NULL DEFAULT 0,
    is_active               INTEGER NOT NULL CHECK (is_active IN (0, 1))
) STRICT;
```

---

## 3. Application-Level Cross-Database Integrity Protocol

Because SQLite cannot enforce `FOREIGN KEY` constraints across attached databases (`course_<lang>.klpack` and `user_telemetry.db`), the Rust `StorageManager` runs a mandatory **Cross-DB Integrity & Migration Handshake** inside a transaction whenever a course pack is mounted:

1. **Pack Hash Verification:** Computes or verifies `manifest_blake3_hash` of the mounted `.klpack` against `installed_packs`.
2. **Orphan Card Check:** Runs a single join query across the attached databases:
   ```sql
   SELECT c.card_id, c.concept_id
   FROM telemetry.fsrs_cards c
   LEFT JOIN course.concepts k ON c.concept_id = k.concept_id
   WHERE c.lang_code = ?1 AND k.concept_id IS NULL;
   ```
3. **Safe Course-Pack Upgrade Migration:** If a `.klpack` is upgraded (e.g., `0.1.0` $\rightarrow$ `0.2.0`), any card whose `concept_id` remains valid preserves its exact FSRS history; any card whose `concept_id` was deprecated or split is marked in `review_logs` with `event_type = 'PACK_MIGRATION'` rather than silently breaking at runtime.
