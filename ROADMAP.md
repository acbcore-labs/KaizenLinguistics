# ROADMAP — Phased Engineering & Verification Blueprint

**Project:** KaizenLinguistics  
**Target Platform:** macOS (`aarch64-apple-darwin`, MacBook Pro M1, 16 GB Unified Memory)  
**Engineering Methodology:** Tranche-Gated Proposer-Verifier Architecture  

---

## 1. Execution Strategy & Tranche Gates

Development of **KaizenLinguistics** is structured into five deterministic engineering tranches (**Tranche 0 through Tranche 4**). No tranche advances until its quantitative acceptance criteria and automated verification suite pass at $100\%$.

```
Tranche 0: Storage, Schema & FSRS-rs Core Kernel (Weeks 1–2)
    │
    ▼
Tranche 1: Offline Proposer-Verifier Course Compiler (Weeks 3–5)
    │
    ▼
Tranche 2: OpenTruAccent Acoustic & Phoneme Scoring Engine (Weeks 6–8)
    │
    ▼
Tranche 3: Tauri v2 Zero-L1 Immersion UI & State Machine (Weeks 9–11)
    │
    ▼
Tranche 4: Continuous Telemetry Optimization & Multi-Language Scaling (Week 12+)
```

---

## 2. Detailed Tranche Breakdown

### Tranche 0: Storage, Schema & Coupled `fsrs-rs` Kernel
**Objective:** Establish the Rust workspace (`app/src-tauri`), SQLite `STRICT` dual-database layer, `BLAKE3` content-addressable asset store, and the coupled 4-modality FSRS-5 scheduling engine.

* **Core Deliverables:**
  * `storage/`: Rust `rusqlite` database manager implementing `course_<lang>.klpack` (read-only) and `user_telemetry.db` (`WAL` mode) per [`docs/05-DATA-SCHEMA-AND-STATE-MACHINE.md`](./docs/05-DATA-SCHEMA-AND-STATE-MACHINE.md).
  * `scheduler/`: Integration of [open-spaced-repetition/fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) ([`fsrs` crate](https://docs.rs/fsrs)) implementing the collateral transfer matrix $\mathbf{\Gamma}$ across `LISTEN`, `READ`, `SPEAK`, and `WRITE` modalities ([FSRS Algorithm Wiki](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm)).
  * `session/`: Deterministic `SessionEngine` state machine handling Stages 1–6 scaffolding and immediate lapse remediation queues.
* **Verification Gate (Tranche 0 Seal):**
  * Property-based unit tests (`proptest`) verifying 10,000 simulated review cycles with zero foreign-key violations, deterministic replay from `review_logs`, and `< 1 ms` scheduling overhead per card transition.

---

### Tranche 1: Offline Proposer-Verifier Course Compiler (`kaizen-compiler`)
**Objective:** Build the zero-cost Python (`uv`) + Rust offline ETL compiler that transforms public linguistic and visual dumps into verified `.klpack` archives.

* **Core Deliverables:**
  * `ingest/lexicon.py`: Stream parser for [Kaikki.org JSONL dictionaries](https://kaikki.org/dictionary/rawdata.html) ([tatuylonen/wiktextract](https://github.com/tatuylonen/wiktextract) / [Kaikki-Wiktionary-Ultimate-SQLite](https://huggingface.co/datasets/wave101828228/Kaikki-Wiktionary-Ultimate-SQLite)) and [Wikidata Lexicographical Data](https://www.wikidata.org/wiki/Wikidata:Lexicographical_data).
  * `ingest/syntax.py`: [Universal Dependencies](https://universaldependencies.org/) `CoNLL-U` parser and [Tatoeba](https://en.wikipedia.org/wiki/Tatoeba) ([Helsinki-NLP/tatoeba-challenge](https://github.com/Helsinki-nlp/tatoeba-challenge)) sentence indexer with topological $i+1$ concept ordering.
  * `verifier/visual_gate.py`: Scene retriever indexing [Open Images Dataset V7](https://docs.ultralytics.com/datasets/detect/open-images-v7) visual relationship triplets and gating candidate $2 \times 2$ quads with [apple-aiml-research/ml-mobileclip](https://github.com/apple-aiml-research/ml-mobileclip) (`MobileCLIP-S2` on [mlfoundations/open_clip](https://github.com/mlfoundations/open_clip)) to enforce diagonal cosine margin $\Delta_{\text{clip}} \ge 0.18$.
  * `tts/synth.py`: Batch reference audio generator prioritizing [Mozilla Common Voice](https://commonvoice.mozilla.org/) CC0 clips and synthesizing missing minimal pairs at `24 kHz` via [gabrimatic/kokoro-mlx](https://github.com/gabrimatic/kokoro-mlx) ([Neywa-Labs/mlx-audio](https://github.com/Neywa-Labs/mlx-audio)), followed by millisecond phoneme alignment via [Montreal Forced Aligner](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner) or [lingjzhu/charsiu](https://github.com/lingjzhu/charsiu).
* **Verification Gate (Tranche 1 Seal):**
  * Compile complete **Pilot Language Level 1 Course Pack (Units 1–4, 200 Concepts, 800 Quads)** with $100\%$ of quads passing the $\Delta_{\text{clip}} \ge 0.18$ disambiguation gate and $100\%$ of sentences carrying verified IPA phoneme timestamps.

---

### Tranche 2: "OpenTruAccent" Acoustic & Phoneme Scoring Pipeline
**Objective:** Implement the real-time, on-device speech recognition, forced alignment, and Goodness of Pronunciation ($\text{GoP}_{\text{LPR}}$) engine optimized for Apple M1 (`Metal` + `CoreML` ANE).

* **Core Deliverables:**
  * `audio/capture.rs`: Lock-free `cpal` + `rubato` 16 kHz mono PCM ring buffer with Silero VAD endpointing.
  * `truaccent/whisper_head.rs`: In-process C/C++ FFI bindings to [ggml-org/whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com) (using CoreML ANE encoder + Metal F16 decoder) for grammar-constrained word recognition ([Whisper.cpp Review](https://www.promptquorum.com/power-local-llm/whisper-cpp-review)).
  * `truaccent/phoneme_head.rs`: ONNX Runtime (`CoreMLExecutionProvider`) inference of [facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft) (and [facebook/wav2vec2-lv-60-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-lv-60-espeak-cv-ft)) paired with constrained DTW / Viterbi phoneme alignment ([Halleck45/OpenPronounce](https://github.com/Halleck45/OpenPronounce)) and [espeak-ng](https://github.com/espeak-ng/espeak-ng) / [TigreGotico/espyak](https://github.com/TigreGotico/espyak) / [dmort27/epitran](https://github.com/dmort27/epitran) grapheme-to-phoneme span mapping.
  * `truaccent/prosody.rs`: `pYIN` fundamental frequency ($F_0$) contour extractor and vowel/geminate duration ratio evaluator.
* **Verification Gate (Tranche 2 Seal):**
  * End-to-end latency from utterance completion ($t_{\text{offset}}$) to UI diagnostic JSON payload **$< 200\text{ ms}$** on M1 MacBook Pro.
  * Combined resident memory of `kaizen-app` + acoustic models **$< 1.3\text{ GB}$**.
  * Synthetic phonetic perturbation benchmark (deliberate phoneme substitutions and vowel-length errors) detected with **$> 92\%$ precision/recall**.

---

### Tranche 3: Tauri v2 Zero-L1 Immersion UI & Interactive Remediation
**Objective:** Build the high-craft [Tauri v2](https://github.com/tauri-apps/tauri) desktop interface delivering a distraction-free, zero-translation immersion experience.

* **Core Deliverables:**
  * **2x2 Minimal-Pair Immersion Grid:** Hardware-accelerated image quad canvas with keyboard (`1`–`4`, `Space` to replay, `R` to record) and click navigation, plus post-selection bound-morpheme diff highlighting.
  * **OpenTruAccent Acoustic HUD:** Real-time waveform & $F_0$ pitch contour overlay (Reference speaker vs. Learner), color-coded orthographic/IPA spans (`Green` / `Amber` / `Red`), and interactive click-to-compare phoneme slice playback.
  * **Articulatory & Phoneme Heatmap Dashboard:** Longitudinal confusion matrix visualizing mastered vs. substituted IPA phones across places and manners of articulation.
* **Verification Gate (Tranche 3 Seal):**
  * 60 fps UI rendering on macOS `WKWebView`, zero $L_1$ string leakage across all lesson screens, and complete offline execution with network interfaces disabled.

---

### Tranche 4: Personal FSRS Optimization & Multi-Language Scaling
**Objective:** Activate longitudinal machine-learning parameter tuning and expand `.klpack` compilation templates across language families.

* **Core Deliverables:**
  * Background invocation of the [fsrs-rs Optimizer](https://github.com/open-spaced-repetition/fsrs-rs) every 250 logged reviews to fit personalized weights $\mathbf{w}_{0..20}$ to actual retention telemetry.
  * Language-family specialized plugins for `kaizen-compiler`:
    * **Romance & Germanic (Spanish, French, Italian, German):** Gender/case morphological highlighters.
    * **East Asian (Japanese, Mandarin, Korean):** Pitch-accent / 4-tone $F_0$ DTW weighting and script-layer scaffolding (Kanji/Hanzi $\leftrightarrow$ Kana/Pinyin $\leftrightarrow$ IPA).
    * **Semitic & Slavic (Arabic, Russian, Hebrew):** Root-and-pattern (`triconsonantal root`) and 6/7-case paradigm matrix views.

---

## 3. Immediate Next Engineering Actions (Bootstrap Sequence)

1. **Initialize Compiler Environment:** Create `compiler/pyproject.toml` using `uv` with `wiktextract`, `espyak`, `epitran`, `onnxruntime`, `mlx`, and `kokoro-mlx`.
2. **Initialize Tauri v2 + Rust Workspace:** Scaffold `app/src-tauri` with `rusqlite` (`bundled`), `fsrs`, `blake3`, `cpal`, `rubato`, and `whisper-rs`.
3. **Select Pilot Target Language ($L_2$):** Configure `compiler/config/pilot_lang.toml` for the initial target language and run Stage 1–5 compilation on Unit 1.
