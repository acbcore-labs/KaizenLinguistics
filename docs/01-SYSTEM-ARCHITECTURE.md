# 01 — System Architecture & Local-First Runtime Topology

**Project:** KaizenLinguistics (Personal Proprietary Dynamic Immersion Engine)  
**Target Host Architecture:** Apple Silicon (`arm64` / M1 MacBook Pro, 16 GB Unified Memory, 1 TB NVMe SSD)  
**Cost & Licensing Constraint:** 100% Zero-Cost, Offline-First, Pure Open-Source Models & Public/CC Corpora  

---

## 1. Executive Architectural Vision

**KaizenLinguistics** re-engineers and surpasses the classic [Rosetta Stone Dynamic Immersion Method](https://www.rosettastone.com/our-method) and its proprietary *TruAccent* speech engine using a deterministic, zero-cloud, local-first software stack. Whereas legacy commercial platforms rely on static, manually photographed scripts and coarse acoustic templates, KaizenLinguistics combines:

1. **Inductive Visual-Audio-Orthographic Immersion:** Zero native-language ($L_1$) translation during active instruction, forcing direct neural binding between visual concepts, spoken phonetics, and target-language ($L_2$) morphosyntax ([Rosetta Stone Blog: Dynamic Immersion](https://blog.rosettastone.com/dynamic-immersion-learning-with-rosetta-stone/)).
2. **Proposer-Verifier Curriculum Compilation:** An offline ingestion and synthesis pipeline proposes candidate minimal-pair visual/audio exercises from open corpora ([Kaikki.org Machine-Readable Wiktionary](https://kaikki.org/dictionary/rawdata.html), [Wikidata Lexicographical Data](https://www.wikidata.org/wiki/Wikidata:Lexicographical_data), [Tatoeba](https://en.wikipedia.org/wiki/Tatoeba), and [Universal Dependencies](https://universaldependencies.org/)), while a strict deterministic **Verifier** validates morphological contrast, visual disambiguation margin via [Apple MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip), and phonetic alignability before sealing lessons into an immutable Merkle-hashed content DAG.
3. **Sub-Phonemic Acoustic Telemetry ("OpenTruAccent"):** Frame-level forced alignment and Goodness of Pronunciation (GoP) scoring powered by [ggml-org/whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com), [facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft), and Dynamic Time Warping ([Halleck45/OpenPronounce](https://github.com/Halleck45/OpenPronounce)), paired with 24 kHz neural speech synthesis via [Kokoro-82M on MLX](https://github.com/gabrimatic/kokoro-mlx).
4. **Mathematically Optimal Memory Scheduling:** Native Rust integration of the Free Spaced Repetition Scheduler ([open-spaced-repetition/fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs)) extended across coupled multi-modal stability vectors (Listening, Reading, Speaking, Writing).

---

## 2. Three Engineering Pillars Applied to KaizenLinguistics

To guarantee zero-confabulation reliability, deterministic state transitions, and sub-50ms interactive latency on a 16 GB Apple M1 machine, the system enforces three foundational engineering pillars:

### Pillar I: Proposer-Verifier Duality (Zero Unverified Curriculum Artifacts)
* **The Problem:** Generative or scraped language content frequently suffers from ambiguous images (where multiple pictures plausibly match a sentence), broken morphological agreement, or unverified G2P transcriptions.
* **The Architectural Solution:** Curriculum generation is strictly decoupled from runtime execution into a two-stage **Proposer $\rightarrow$ Verifier** compiler:
  * **The Proposer (Offline Compiler):** Mines sentence-image-audio candidates from [Tatoeba](https://github.com/Helsinki-nlp/tatoeba-challenge), [Kaikki.org](https://kaikki.org/), and [Open Images V7](https://docs.ultralytics.com/datasets/detect/open-images-v7).
  * **The Verifier (Deterministic Gate):** Rejects any candidate quad $(I_1, I_2, I_3, I_4)$ unless:
    1. **Morphosyntactic Parse Verification:** The target sentence parses cleanly against [Universal Dependencies](https://universaldependencies.org/) CoNLL-U grammar constraints with exactly one targeted grammatical or lexical delta relative to its distractors.
    2. **Visual-Semantic Margin Verification:** Using [apple-aiml-research/ml-mobileclip](https://github.com/apple-aiml-research/ml-mobileclip) embeddings, the cosine similarity between the target caption $T_i$ and target image $I_i$ must exceed all distractor images $I_{j \neq i}$ by a strict margin $\Delta_{\text{clip}} \ge 0.18$.
    3. **Phonetic Alignment Verification:** The reference audio (from [Mozilla Common Voice](https://commonvoice.mozilla.org/) or [Kokoro-82M MLX](https://blaizzy.github.io/mlx-audio/models/tts/kokoro/)) must achieve $100\%$ phoneme coverage and mean posterior probability $\ge 0.85$ when force-aligned against the [espeak-ng](https://github.com/espeak-ng/espeak-ng) IPA transcription.

### Pillar II: Deterministic State & Auditable Merkle Content Addressing
* Every lexical concept, morphological paradigm, audio waveform, image tile, and 4-choice immersion quad is content-addressed by its `BLAKE3` / `SHA-256` digest.
* Learner progress (FSRS memory states $[D, S, R]$ and phoneme error confusion matrices) is recorded as an append-only event log in SQLite (`WAL` mode), allowing deterministic replay, exact parameter re-optimization via [fsrs-rs](https://docs.rs/fsrs), and zero risk of state corruption.

### Pillar III: Hardware-Sympathetic Zero-Allocation Execution on Apple Silicon
* **Strict Memory Ceiling (`< 6.0 GB` Combined Resident Footprint):** Designed specifically for an Apple MacBook Pro M1 (16 GB unified RAM), leaving $>10\text{ GB}$ free for macOS and developer workflows.
* **Unified Memory & Neural Engine Offload:**
  * Desktop shell built on [Tauri v2](https://github.com/tauri-apps/tauri) (`Rust` core + native macOS `WKWebView`), eliminating the $400\text{--}800\text{ MB}$ Chromium overhead of Electron ([Tauri Architecture](https://github.com/tauri-apps/tauri/blob/dev/ARCHITECTURE.md)).
  * Pre-allocated PCM audio ring buffers in Rust (`cpal`) pass 16 kHz mono `f32` audio frames directly to in-process C/C++ [whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com) (using Apple CoreML / Metal backends) and ONNX/MLX phonetic encoders without heap fragmentation or Python IPC bottlenecks during active speaking drills.

---

## 3. High-Level System Topology

```
+-----------------------------------------------------------------------------------+
|                        KAIZEN LINGUISTICS DESKTOP CLIENT                          |
|                   Tauri v2 Application Shell (macOS WKWebView)                    |
|  +-----------------------+  +------------------------+  +----------------------+  |
|  | 2x2 Visual Immersion  |  | Real-Time IPA & Pitch  |  | FSRS-5 Telemetry &   |  |
|  | Quad Canvas (Zero-L1) |  | Waveform Visualizer    |  | Phoneme Heatmap UI   |  |
|  +-----------+-----------+  +------------+-----------+  +----------+-----------+  |
+--------------|---------------------------|-------------------------|--------------+
               | Tauri IPC Commands        | Zero-Copy Audio Stream  |
+--------------v---------------------------v-------------------------v--------------+
|                         RUST CORE RUNTIME DAEMON (src-tauri)                      |
|  +-----------------------+  +------------------------+  +----------------------+  |
|  | Deterministic Lesson  |  | FSRS-rs Multi-Modal    |  | Acoustic Alignment & |  |
|  | State Machine Engine  |  | Scheduler & Optimizer  |  | GoP Scoring Engine   |  |
|  +-----------+-----------+  +------------+-----------+  +----------+-----------+  |
|              |                           |                         |              |
|  +-----------v---------------------------v-------------------------v-----------+  |
|  |              Embedded SQLite 3 (WAL Mode) + Content-Addressed CAS           |  |
|  |   [Lexemes | UD Paradigms | Visual Quads | FSRS Cards | Phoneme Telemetry]  |  |
|  +-----------------------------------------------------------------------------+  |
+-----------------------------------------|-----------------------------------------+
                                          | In-Process FFI / Local UNIX Socket
+-----------------------------------------v-----------------------------------------+
|                 LOCAL APPLE SILICON (M1 METAL / ANE / MLX) INFERENCE              |
|  +-------------------------+ +-------------------------+ +---------------------+  |
|  | ASR & Word Timestamps   | | Phoneme CTC Posteriors  | | Neural TTS Engine   |  |
|  | ggml-org/whisper.cpp    | | wav2vec2-xlsr-53-espeak | | Kokoro-82M (MLX)    |  |
|  | (CoreML ANE + Metal)    | | + DTW Forced Aligner    | | 24kHz Multi-Voice   |  |
|  +-------------------------+ +-------------------------+ +---------------------+  |
+-----------------------------------------------------------------------------------+
                                          ^
                                          | Immutable Sealed Course Packs (.klpack)
+-----------------------------------------|-----------------------------------------+
|               OFFLINE PROPOSER-VERIFIER CURRICULUM COMPILER (Python/Rust)         |
|  +-----------------------------------------------------------------------------+  |
|  | Open Data Sources: Kaikki.org (Wiktextract) | Wikidata Lexemes | Tatoeba    |  |
|  | Mozilla Common Voice | Universal Dependencies | Open Images V7 | Wikimedia  |  |
|  +--------------------------------------+--------------------------------------+  |
|                                         |                                         |
|  +--------------------------------------v--------------------------------------+  |
|  | Verifier Gate: MobileCLIP Semantic Margin + UD Syntax Diff + MFA Alignment  |  |
|  +-----------------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------------+
```

---

## 4. Component Stack & Memory Budget on Apple M1 (16 GB RAM)

Every subsystem is selected to maximize inference speed on Apple Silicon (`Metal` GPU + `Apple Neural Engine` + `ARM NEON`) while keeping resident memory minimal:

| Subsystem Layer | Selected Open-Source Technology | License | Execution Backend on M1 | Resident RAM Budget | Primary Role |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Desktop Shell & UI** | [tauri-apps/tauri](https://github.com/tauri-apps/tauri) (v2) + SolidJS / TypeScript | MIT / Apache-2.0 | Native `WKWebView` + Rust Core | `~95 MB` | Low-latency 60fps UI, audio visualization, keyboard/touch bindings |
| **Spaced Repetition Core** | [open-spaced-repetition/fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) (`fsrs` crate) | MIT | Native Rust (`aarch64-apple-darwin`) | `< 15 MB` | DSR memory state scheduling, review log optimization, simulation |
| **Orthographic & Word ASR** | [ggml-org/whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com) (`whisper-small` / `base` Q5_1) | MIT | CoreML (ANE) + Metal F16 | `~280 MB` | Real-time streaming transcription and word-level boundary detection |
| **Phoneme CTC Acoustic Model** | [facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft) (INT8/FP16 ONNX) | Apache-2.0 | ONNX Runtime (`CoreMLExecutionProvider`) | `~420 MB` | Language-agnostic IPA frame posteriors for Goodness of Pronunciation (GoP) |
| **G2P & Phonemic Dictionary** | [espeak-ng/espeak-ng](https://github.com/espeak-ng/espeak-ng) + [TigreGotico/espyak](https://github.com/TigreGotico/espyak) + [dmort27/epitran](https://github.com/dmort27/epitran) | GPLv3 / MIT | Native C FFI / Pure Python (Compiler) | `< 10 MB` | Grapheme-to-Phoneme conversion and articulatory feature lookup |
| **Local Neural TTS** | [gabrimatic/kokoro-mlx](https://github.com/gabrimatic/kokoro-mlx) / [Neywa-Labs/mlx-audio](https://github.com/Neywa-Labs/mlx-audio) (`Kokoro-82M`) | Apache-2.0 | Apple MLX (Metal GPU) | `~350 MB` | Instant 24 kHz natural speech generation across 54+ voice presets |
| **Visual Grounding Verifier** | [apple-aiml-research/ml-mobileclip](https://github.com/apple-aiml-research/ml-mobileclip) (`MobileCLIP-S2`) | Apple Sample Code / MIT | Apple MLX / CoreML (Offline Build) | `~310 MB` *(Compiler only)* | Verifying image-sentence correspondence and mining visual distractors |
| **Offline Forced Alignment** | [Montreal-Forced-Aligner](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner) / [lingjzhu/charsiu](https://github.com/lingjzhu/charsiu) | MIT | Kaldi / PyTorch MPS (Offline Build) | `~600 MB` *(Compiler only)* | Pre-computing millisecond phoneme boundaries on reference audio |
| **Embedded Storage** | SQLite 3 (`rusqlite`, WAL mode, `mmap_size=256MB`) | Public Domain | Native C/Rust in-process | `~60 MB` | Relational storage for lexemes, DAG exercises, FSRS state, and telemetry |
| **Total Active Runtime** | **All Runtime Processes Combined** | **100% Free / Open** | **M1 Unified Memory** | **`~1.23 GB` Active** | **Leaves `>14.5 GB` unified RAM free on 16 GB MacBook Pro M1** |

---

## 5. Runtime Process Separation: Compiler vs. Interactive Engine

A critical architectural mistake in open-source language apps is bundling heavy data-wrangling dependencies (Wiktionary XML dump parsers, Kaldi recipe trainers, multi-gigabyte image indices) into the interactive desktop runtime. KaizenLinguistics splits the system into two cleanly isolated binaries:

### 5.1 `kaizen-compiler` (Offline Course Pack Builder)
* Written in **Python 3.12+ (managed via `uv`) + Rust CLI helpers**.
* Ingests raw upstream dumps:
  * [Kaikki.org JSONL dictionaries](https://kaikki.org/dictionary/rawdata.html) generated by [tatuylonen/wiktextract](https://github.com/tatuylonen/wiktextract) (or pre-indexed [Kaikki-Wiktionary-Ultimate-SQLite](https://huggingface.co/datasets/wave101828228/Kaikki-Wiktionary-Ultimate-SQLite)).
  * [Wikidata Lexicographical Data](https://www.wikidata.org/wiki/Wikidata:Lexicographical_data) senses and Q-ID concept links.
  * [Universal Dependencies](https://universaldependencies.org/) treebanks for morphosyntactic templates.
  * [Tatoeba](https://en.wikipedia.org/wiki/Tatoeba) sentence pairs and [Mozilla Common Voice](https://commonvoice.mozilla.org/) recordings.
  * [Open Images V7](https://docs.ultralytics.com/datasets/detect/open-images-v7) and Wikimedia Commons visual assets.
* Runs the **Proposer-Verifier pipeline** (using [MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip), [Montreal Forced Aligner](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner), and [Kokoro-82M MLX](https://github.com/gabrimatic/kokoro-mlx)) to compile deterministic, content-addressed course bundles (`<lang_code>_v1.klpack`, backed by SQLite + Opus audio + WebP images).

### 5.2 `kaizen-app` (Interactive Desktop Immersion Client)
* Built with [Tauri v2](https://github.com/tauri-apps/tauri) (`Rust` backend + lightweight reactive frontend).
* Mounts `.klpack` course archives read-only and maintains a private user telemetry database (`user_profile.db`).
* Executes real-time audio capture, VAD segmentation, [whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com) + [wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft) phoneme evaluation, and [fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) scheduling with zero network calls and sub-50ms UI frame times.

---

## 6. Inter-Process & Hardware Data Flow (Real-Time Audio Loop)

During an interactive speaking or pronunciation exercise, latency between the user finishing an utterance and receiving phoneme-colored visual feedback must be **under 250 ms**:

1. **Audio Capture (`cpal` Rust Thread):** Captures microphone input at native sample rate, resamples via SIMD `rubato` to `16,000 Hz` mono `f32`, and writes into a lock-free ring buffer (`rtrb`).
2. **Streaming VAD & Endpointing:** Frame-level energy + Silero VAD detects voice onset ($t_{\text{onset}}$) and utterance completion ($t_{\text{offset}}$ after $350\text{ ms}$ trailing silence).
3. **Parallel Acoustic Dispatch:**
   * **Thread A (`whisper.cpp` CoreML/Metal):** Evaluates orthographic accuracy and word-level timestamps ([Whisper.cpp Review](https://www.promptquorum.com/power-local-llm/whisper-cpp-review)).
   * **Thread B (`wav2vec2-xlsr-53-espeak-cv-ft` + DTW Aligner):** Computes the $(T \times |\Sigma_{\text{IPA}}|)$ log-posterior matrix, runs constrained Dynamic Time Warping ([Halleck45/OpenPronounce](https://github.com/Halleck45/OpenPronounce)) against the expected reference phoneme sequence $\Phi_{\text{target}}$, and extracts pitch contour $F_0(t)$ via the YIN/pYIN algorithm.
4. **Deterministic Diagnostic Aggregation:** Rust aggregates per-phoneme Goodness of Pronunciation ($\text{GoP}(\phi_i)$), duration z-scores, and intonation correlation into a structured JSON telemetry payload rendered immediately in the UI as color-coded IPA tokens (`Green` $\ge -1.2$, `Amber` $\in [-2.5, -1.2)$, `Red` $< -2.5$).
