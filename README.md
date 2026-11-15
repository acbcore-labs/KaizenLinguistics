# KaizenLinguistics (v2.0 — Hardened Specification)

**A Personal, Local-First "Rosetta Stone" Contrast-Graph & Philological Immersion Engine**  
*Primary Target Languages: **Koine Greek (`grc`)** & **Classical / Biblical Hebrew (`hbo`)***  
*Engineered for Apple Silicon (`M1` / `Metal` / `CoreML` / `MLX`) using 100% Open-Source Models and Public Corpora.*

---

## 1. Overview & Architectural Identity

**KaizenLinguistics** is a personal, local-first language acquisition system built to take a learner from zero script knowledge to intermediate reading, listening, and spoken internalization in **Koine Greek (`grc`)** and **Ancient / Biblical Hebrew (`hbo`)**—using 100% free, open-source datasets and local Apple Silicon inference.

Following rigorous architectural review (v2.0), the platform is grounded in five engineering commitments:
1. **Contrast-Graph Pedagogy (Not Isolated Labels):** Lessons are structured around controlled minimal contrasts (crossing case roles with word order in Koine Greek; crossing gender/number or *binyan* verbal stems on invariant triliteral roots in Biblical Hebrew) presented via $2 \times 2$ visual grids, 3-frame aspectual timelines, short dialogues, and a progressive clause-peeling authentic text reader.
2. **Separate Offline Compilation & Multi-Evidence Editorial Gating:** An offline toolkit (`kaizen-compiler`) joins gold-standard biblical and classical corpora ([Clear-Bible/macula-greek](https://github.com/Clear-Bible/macula-greek), [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew), [morphgnt/sblgnt](https://github.com/morphgnt/sblgnt), [ETCBC/bhsa](https://github.com/ETCBC/bhsa), [openscriptures/morphhb](https://github.com/openscriptures/morphhb), [openscriptures/HebrewLexicon](https://github.com/openscriptures/HebrewLexicon), and [Abbott-Smith](https://github.com/translatable-exegetical-tools/Abbott-Smith)) with open visual/audio immersion progressions ([Aleph with Beth](https://dantes.io/resource/aleph-with-beth-free-biblical-hebrew-video-course-6abc0ecdf769b646f61723db) and [Alpha with Angela](https://freegreek.online/about/)). Every exercise records a **6-dimensional evidence manifest** (structural validity, single-delta contrast purity, [Apple MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip) screening score, forced-alignment confidence, license provenance, and explicit human editorial approval).
3. **Calibrated Acoustic Assessment with First-Class Abstention:** Separates signal usability, unconstrained utterance verification ([ggml-org/whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com)), forced alignment confidence, and phoneme posterior scoring ([facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft))—explicitly returning **`UNABLE_TO_ASSESS`** on noisy or low-confidence recordings rather than fabricating articulatory diagnoses, while always providing instant side-by-side learner-vs-native waveform playback.
4. **Pinned FSRS Scheduling with Observed-Only Stability Updates:** Integrates a pinned version of [open-spaced-repetition/fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) ([FSRS Algorithm Wiki](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm)) where Listening, Reading, Speaking, and Writing maintain independent review histories updated strictly from direct observations, using **24-hour sibling burying** to prevent same-day repetition.
5. **Vertical-Slice Execution Order:** Prioritizes a working, restart-safe desktop learning loop ([Tauri v2](https://github.com/tauri-apps/tauri)) with 20 curated concepts and 40 contrast exercises before scaling corpus automation or auto-grading speech.

---

## 2. Foundational Documentation Suite

| Document | Architectural Scope & Contents |
| :--- | :--- |
| **[`docs/01-SYSTEM-ARCHITECTURE.md`](./docs/01-SYSTEM-ARCHITECTURE.md)** | Forward-corrective axioms, runtime vs. compiler separation, reconciled M1 memory budget (`~1.23 GB` warm target, `< 2.0 GB` hard cap), and decomposed VAD vs. compute latency targets. |
| **[`docs/02-PEDAGOGY-AND-FSRS-ENGINE.md`](./docs/02-PEDAGOGY-AND-FSRS-ENGINE.md)** | Formal **Contrast Graph** specification, unconfounded minimal-pair designs for Koine Greek and Biblical Hebrew, 6-stage modality progression, pinned [FSRS](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm) equations, and sibling burying rules. |
| **[`docs/03-SPEECH-AND-TRUACCENT-PIPELINE.md`](./docs/03-SPEECH-AND-TRUACCENT-PIPELINE.md)** | 5-stage epistemic speech pipeline with **`UNABLE_TO_ASSESS`** abstention gate, deterministic Tiberian Hebrew (`hbo`) and Restored/Erasmian Koine Greek (`grc`) G2P rules, and Milestone 3 empirical calibration protocol. |
| **[`docs/04-OPEN-CORPUS-AND-ASSET-PIPELINE.md`](./docs/04-OPEN-CORPUS-AND-ASSET-PIPELINE.md)** | Template-first assisted compilation workflow, 6-dimensional `exercise_evidence` schema, editorial approval gate, and 4-tier progression from concrete scenes to abstract discourse. |
| **[`docs/05-DATA-SCHEMA-AND-STATE-MACHINE.md`](./docs/05-DATA-SCHEMA-AND-STATE-MACHINE.md)** | Complete SQLite `STRICT` DDL for `.klpack` and `user_telemetry.db`, deterministic replay event log (`review_logs`), restart-safe `session_checkpoints`, and Rust cross-database integrity checks. |
| **[`docs/06-KOINE-GREEK-AND-BIBLICAL-HEBREW-CORPUS-SPEC.md`](./docs/06-KOINE-GREEK-AND-BIBLICAL-HEBREW-CORPUS-SPEC.md)** | Structural comparison of Biblical (`hbo`) vs. Modern (`heb`) Hebrew, Restored vs. Erasmian Koine Greek (`grc`), complete catalog of downloadable open repositories, and the Tier 1–4 trajectory to intermediate fluency. |
| **[`docs/07-UI-UX-DESIGN-SYSTEM.md`](./docs/07-UI-UX-DESIGN-SYSTEM.md)** | "Atelier Aldus" Venetian polyglot print design system (Vellum/Basalt atmospheric modes, Golden F₀ Oscilloscope, Rubric Vermilion callouts, and 0% generic SaaS UI). |
| **[`ROADMAP.md`](./ROADMAP.md)** | Vertical-slice engineering blueprint and verified completion record across all four milestones. |

---

## 3. System Architecture & Verification Record

KaizenLinguistics combines five production subsystems engineered for Apple Silicon (`aarch64-apple-darwin`):

1. **Dual-Database Storage Model**:
   * **`packs/*.klpack`**: Read-only, cryptographically hashed (BLAKE3) SQLite STRICT archives packaging compiled concepts, unconfounded minimal-contrast exercises, and zlib-compressed SVG and WAV media assets.
   * **`packs/user_telemetry.db`**: Personal, mutable, append-only review event log (`review_logs`) maintaining independent stability histories across all 4 learning modalities with 24-hour sibling burying.
2. **Deterministic Historical G2P Transducers**:
   * **Koine Greek (`grc`)**: Restored 1st Century Imperial Koine, Erasmian, and Modern Athenian profiles with exact breathings, diphthong monophthongization, nasal assimilation, and pitch/stress contours.
   * **Biblical Hebrew (`hbo`)**: Tiberian Masoretic with *Begadkefat* spirantization, *dagesh forte* gemination, *qamatz gadol/qatan*, vocal vs. silent *shewa*, and furtive *pathach* transposition.
3. **TrueAccent Epistemic Speech Pipeline**:
   * Evaluates raw 16 kHz PCM across 5 validation gates: Signal Hygiene (clipping/SNR) $\rightarrow$ Contrast Foil Discrimination $\rightarrow$ Forced Alignment Confidence ($\ge 0.80$) $\rightarrow$ Phoneme Posterior Scoring (`GoP_LPR`) & Duration $\rightarrow$ Calibrated FSRS Proposal.
   * **Epistemic Invariant**: When audio confidence is low, the engine returns **`UNABLE_TO_ASSESS`** and guarantees **zero stability degradation** on the learner's FSRS card.
4. **Tauri v2 Desktop Shell & Native IPC Suite**:
   * Seamless macOS desktop application shell exposing 6 native IPC commands: `get_catalog`, `record_review`, `evaluate_speech`, `g2p_transcribe`, `check_integrity`, and `get_reader_pericopes`.
5. **Clause Onion-Peeler Authentic Reader**:
   * 3-Stratum syntactic parser progressively peeling biblical verses from core kernel ($V + S + O$) $\rightarrow$ adjuncts $\rightarrow$ complete textus integer with interactive morphological and lexical apparatus.

---

## 4. Quick Start & Verification Suite

### Run the Full 6-Gate End-to-End Verification Suite
```bash
bash scripts/verify_milestone1.sh
```
This executes:
1. Deterministic G2P Transducers (`compiler/g2p.py`).
2. 140-Concept Course Pack Compilers & Dual-DB Integrity (`compiler/expand_tier23_packs.py`).
3. Acoustic Calibration Benchmark on 120-utterance test corpus (`compiler/acoustic_calibration.py`).
4. Automated Open Biblical Corpus Ingestion & Syntactic Decomposition (`compiler/ingest_corpus.py`).
5. Rust Core & Native Tauri IPC Unit Tests (23/23 tests passing in 0.15s) (`cargo test`).
6. Native Rust Dual-Database Integrity & Runtime Telemetry Gate (`cargo run -- --cli`).

### Launch the Desktop Application
```bash
cargo run --manifest-path app/src-tauri/Cargo.toml
```
Or open the static web preview directly in any standard browser:
```bash
open app/src-ui/index.html
open app/src-ui/reader.html
```
