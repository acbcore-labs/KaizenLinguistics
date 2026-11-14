# 05 — Deterministic Database Schema, Content DAG & Lesson State Machine

**Project:** KaizenLinguistics  
**Storage Engine:** Embedded SQLite 3 (`WAL` Mode, Foreign Keys Enforced, `BLAKE3` Content-Addressed Blobs)  
**Runtime State Engine:** Native Rust (`src-tauri`) Deterministic Finite State Machine  

---

## 1. Dual-Database Architecture

To keep immutable course content cleanly separated from personal learning telemetry (allowing seamless course updates or multi-language installation without touching user progress), KaizenLinguistics uses two SQLite databases mounted via `ATTACH DATABASE`:

1. **`course_<lang_iso>.klpack` (Read-Only Immutable Course Pack):**
   Contains the Merkle-sealed Concept DAG, Lexemes, Inflection Forms, Sentences, Phoneme Alignments, Visual Quads, and Content-Addressed Media Blobs (`WebP` images and `Opus` 48 kbps audio).
2. **`user_telemetry.db` (Read-Write Personal Ledger):**
   Contains the append-only `review_logs` ledger, active `fsrs_cards` memory vectors ($D, S, R$), per-phoneme confusion matrices (`phoneme_telemetry`), and active lesson session checkpoints.

---

## 2. Complete SQLite DDL Schema Specification

### 2.1 Immutable Course Pack Schema (`course_<lang_iso>.klpack`)

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

-- 1. Cross-Lingual Semantic Concept Nodes (Anchored to Wikidata Q-IDs)
CREATE TABLE concepts (
    concept_id          TEXT PRIMARY KEY,               -- e.g., 'Q726' (Horse) or 'GRAM_PLURAL_NOUN'
    concept_type        TEXT NOT NULL CHECK (concept_type IN ('LEXICAL', 'MORPHOLOGICAL', 'SYNTACTIC', 'PHONOLOGICAL')),
    wikidata_qid        TEXT,                           -- Canonical Wikidata Q-ID if applicable
    cefr_tier           TEXT NOT NULL CHECK (cefr_tier IN ('A1', 'A2', 'B1', 'B2', 'C1')),
    unit_number         INTEGER NOT NULL,               -- 1..20
    lesson_number       INTEGER NOT NULL,               -- 1..80
    content_hash        TEXT NOT NULL UNIQUE            -- BLAKE3 hex digest of canonical concept definition
) STRICT;

-- 2. Target-Language Lexemes & Morphological Paradigms (Kaikki.org / Wiktextract + Wikidata L-IDs)
CREATE TABLE lexemes (
    lexeme_id           TEXT PRIMARY KEY,               -- BLAKE3(lang_code || lemma || upos || sense_idx)
    concept_id          TEXT NOT NULL REFERENCES concepts(concept_id),
    wikidata_lid        TEXT,                           -- e.g., 'L3456'
    lang_code           TEXT NOT NULL,                  -- ISO 639-3 / BCP-47 (e.g., 'es', 'ja', 'de', 'ru')
    lemma               TEXT NOT NULL,                  -- Canonical dictionary headword
    upos                TEXT NOT NULL,                  -- Universal Dependencies POS ('NOUN', 'VERB', 'ADJ', etc.)
    canonical_ipa       TEXT NOT NULL,                  -- Primary IPA transcription from Kaikki / espeak-ng
    gender_class        TEXT,                           -- e.g., 'Masc', 'Fem', 'Neut'
    frequency_rank      INTEGER NOT NULL                -- Corpus zipf frequency rank
) STRICT;

-- 3. Inflected Surface Forms (Universal Dependencies FEATS mapping)
CREATE TABLE inflection_forms (
    form_id             TEXT PRIMARY KEY,
    lexeme_id           TEXT NOT NULL REFERENCES lexemes(lexeme_id),
    surface_form        TEXT NOT NULL,                  -- e.g., 'caballos'
    ud_feats_json       TEXT NOT NULL,                  -- JSON: {"Number": "Plur", "Gender": "Masc"}
    ipa_phonemes_json   TEXT NOT NULL,                  -- JSON array of IPA phones: ["k", "a", "β", "a", "ʎ", "o", "s"]
    g2p_spans_json      TEXT NOT NULL                   -- Grapheme-to-phoneme index map for UI letter highlighting
) STRICT;

-- 4. Content-Addressed Media Assets (WebP 512x512 Images & 24kHz Opus Audio)
CREATE TABLE media_assets (
    asset_hash          TEXT PRIMARY KEY,               -- BLAKE3 digest of raw binary payload
    media_type          TEXT NOT NULL CHECK (media_type IN ('IMAGE_WEBP', 'AUDIO_OPUS')),
    source_corpus       TEXT NOT NULL,                  -- 'OPEN_IMAGES_V7', 'WIKIMEDIA', 'COMMON_VOICE', 'KOKORO_MLX'
    license_spdx        TEXT NOT NULL,                  -- 'CC0-1.0', 'CC-BY-4.0', 'CC-BY-SA-4.0', 'Apache-2.0'
    attribution_url     TEXT NOT NULL,                  -- Verified canonical source URL
    duration_ms         INTEGER,                        -- Populated for AUDIO_OPUS
    payload_blob        BLOB NOT NULL                   -- Compressed media bytes
) STRICT;

-- 5. Target Sentences with MFA Phoneme Timestamps & UD Dependency Trees
CREATE TABLE sentences (
    sentence_id         TEXT PRIMARY KEY,               -- BLAKE3 digest
    tatoeba_id          INTEGER,                        -- Source Tatoeba ID if derived from Tatoeba
    lang_code           TEXT NOT NULL,
    orthographic_text   TEXT NOT NULL,                  -- Full L2 sentence
    ud_conllu_json      TEXT NOT NULL,                  -- Tokenized UD dependency tree & morphological diff tags
    ipa_sequence_json   TEXT NOT NULL,                  -- Canonical sequence of target IPA phones
    audio_asset_hash    TEXT NOT NULL REFERENCES media_assets(asset_hash),
    mfa_alignment_json  TEXT NOT NULL,                  -- Millisecond [t_start, t_end] per word & IPA phoneme
    f0_contour_json     TEXT NOT NULL                   -- Reference speaker normalized semitone pitch contour
) STRICT;

-- 6. Verified 2x2 Minimal-Pair Immersion Quads
CREATE TABLE quad_exercises (
    quad_id             TEXT PRIMARY KEY,
    primary_concept_id  TEXT NOT NULL REFERENCES concepts(concept_id),
    contrast_type       TEXT NOT NULL,                  -- 'LEXICAL_NOUN', 'LEXICAL_VERB', 'MORPH_NUMBER', 'MORPH_TENSE', etc.
    target_slot         INTEGER NOT NULL CHECK (target_slot BETWEEN 0 AND 3),
    sentence_id_0       TEXT NOT NULL REFERENCES sentences(sentence_id),
    image_hash_0        TEXT NOT NULL REFERENCES media_assets(asset_hash),
    sentence_id_1       TEXT NOT NULL REFERENCES sentences(sentence_id),
    image_hash_1        TEXT NOT NULL REFERENCES media_assets(asset_hash),
    sentence_id_2       TEXT NOT NULL REFERENCES sentences(sentence_id),
    image_hash_2        TEXT NOT NULL REFERENCES media_assets(asset_hash),
    sentence_id_3       TEXT NOT NULL REFERENCES sentences(sentence_id),
    image_hash_3        TEXT NOT NULL REFERENCES media_assets(asset_hash),
    clip_min_margin     REAL NOT NULL CHECK (clip_min_margin >= 0.18), -- Verified by MobileCLIP gate
    morpheme_highlight  TEXT NOT NULL                   -- JSON character spans of bound morphemes to highlight post-answer
) STRICT;

-- 7. Prerequisite DAG Edges
CREATE TABLE concept_prerequisites (
    parent_concept_id   TEXT NOT NULL REFERENCES concepts(concept_id),
    child_concept_id    TEXT NOT NULL REFERENCES concepts(concept_id),
    PRIMARY KEY (parent_concept_id, child_concept_id)
) STRICT;
```

### 2.2 Personal Telemetry & FSRS Scheduling Schema (`user_telemetry.db`)

```sql
-- 1. Coupled Multi-Modal FSRS-5 Memory State Cards
CREATE TABLE fsrs_cards (
    card_id             TEXT PRIMARY KEY,               -- concept_id || ':' || modality
    concept_id          TEXT NOT NULL,
    modality            TEXT NOT NULL CHECK (modality IN ('LISTEN', 'READ', 'SPEAK', 'WRITE')),
    scaffold_stage      INTEGER NOT NULL DEFAULT 1 CHECK (scaffold_stage BETWEEN 1 AND 6),
    state               INTEGER NOT NULL DEFAULT 0,     -- 0=New, 1=Learning, 2=Review, 3=Relearning
    stability           REAL NOT NULL DEFAULT 0.0,      -- FSRS Stability S (days)
    difficulty          REAL NOT NULL DEFAULT 5.0,      -- FSRS Difficulty D (1.0..10.0)
    elapsed_days        REAL NOT NULL DEFAULT 0.0,
    scheduled_days      REAL NOT NULL DEFAULT 0.0,
    reps                INTEGER NOT NULL DEFAULT 0,
    lapses              INTEGER NOT NULL DEFAULT 0,
    due_timestamp_ms    INTEGER NOT NULL,
    last_review_ms      INTEGER
) STRICT;

CREATE INDEX idx_fsrs_due ON fsrs_cards(due_timestamp_ms, state);

-- 2. Append-Only Immutable Review Log (For fsrs-rs Parameter Optimization)
CREATE TABLE review_logs (
    log_id              INTEGER PRIMARY KEY AUTOINCREMENT,
    card_id             TEXT NOT NULL REFERENCES fsrs_cards(card_id),
    quad_id             TEXT NOT NULL,
    review_timestamp_ms INTEGER NOT NULL,
    rating              INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 4), -- 1=Again, 2=Hard, 3=Good, 4=Easy
    state_before        INTEGER NOT NULL,
    stability_before    REAL NOT NULL,
    difficulty_before   REAL NOT NULL,
    response_latency_ms INTEGER NOT NULL,
    audio_replays       INTEGER NOT NULL DEFAULT 0,
    gop_mean_score      REAL,                           -- Populated on SPEAK modality
    dl_edit_distance    INTEGER                         -- Populated on WRITE modality
) STRICT;

-- 3. Longitudinal Phoneme & Articulatory Confusion Matrix
CREATE TABLE phoneme_telemetry (
    lang_code           TEXT NOT NULL,
    target_ipa          TEXT NOT NULL,                  -- Expected phoneme φ_m
    spoken_ipa          TEXT NOT NULL,                  -- Argmax phoneme φ_hat detected by wav2vec2-xlsr-53
    occurrence_count    INTEGER NOT NULL DEFAULT 1,
    ewma_gop_score      REAL NOT NULL,                  -- Exponentially weighted moving average GoP_LPR
    last_updated_ms     INTEGER NOT NULL,
    PRIMARY KEY (lang_code, target_ipa, spoken_ipa)
) STRICT;
```

---

## 3. Deterministic Lesson Session State Machine (Rust Core)

During an active study session, the Tauri Rust backend drives a deterministic state machine (`SessionEngine`) that dynamically interleaves **New Concept Scaffolding (Stages 1–6)**, **Due FSRS Reviews**, and **Targeted Phoneme/Minimal-Pair Remediation**:

```
                     +----------------------------+
                     |      SESSION_INIT          |
                     | Load FSRS Due Queue +      |
                     | Next i+1 Unseen Concept    |
                     +-------------+--------------+
                                   │
                                   ▼
                     +----------------------------+
          ┌─────────>|    PRESENT_STIMULUS        |<────────┐
          │          | Render 2x2 Quad or Prompt  |         │
          │          | Play Reference Audio (L2)  |         │
          │          +-------------+--------------+         │
          │                        │                        │
          │                        ▼                        │
          │          +----------------------------+         │
          │          |   AWAIT_USER_RESPONSE      |         │
          │          | - Click (Listen / Read)    |         │
          │          | - Mic Stream (OpenTruAccent)|        │
          │          | - Keystroke (Cloze Write)  |         │
          │          +-------------+--------------+         │
          │                        │                        │
          │                        ▼                        │
          │          +----------------------------+         │
          │          |  EVALUATE_AND_GRADE (G=1..4)|        │
          │          | Compute GoP / Latency / DL |         │
          │          +------+--------------+------+         │
          │                 │              │                │
          │     G >= 3      │              │ G == 1 or 2    │
          │   (Pass/Master) │              │ (Lapse/Error)  │
          │                 ▼              ▼                │
+---------+----------+             +-------+----------------+-----+
| ADVANCE_FSRS_STATE |             |   INJECT_REMEDIATION_DRILL   |
| - Update [D, S, R] |             | - Highlight Morpheme Diff OR |
| - Apply Γ Transfer |             | - Show Articulatory IPA Diff |
| - Pop Queue        |             | - Re-queue Contrast at pos+3 |
+---------+----------+             +------------------------------+
          │
          ▼
+--------------------+
| SESSION_COMPLETE   |
| Trigger fsrs-rs    |
| Batch Optimizer    |
+--------------------+
```

### 3.1 Real-Time Adaptive Remediation Rules
1. **Visual Distractor Foil Trap:** If the learner clicks Panel $j \neq i^*$, the UI immediately plays the audio $A_j$ corresponding to the panel they clicked alongside its text $S_j$, visually contrasts the morphological diff between $S_j$ and $S_{i^*}$, and re-inserts a rotated permutation of the quad $3$ items later in the session queue.
2. **Phonetic Substitution Remediation:** If OpenTruAccent detects that the learner substituted phoneme $\hat{\phi}$ for $\phi^*$ ($\text{GoP}_{\text{LPR}}(\phi^*) < -2.2$) twice in a row, the engine automatically queries `sentences` for a **Phonetic Minimal Pair Drill** contrasting $\phi^*$ vs. $\hat{\phi}$ (e.g., Spanish *pero* `/peɾo/` vs. *perro* `/pero/`) and injects an immediate 30-second auditory discrimination + imitation micro-loop before returning to the main queue.
