# KaizenLinguistics

**A Personal, Zero-Cost, Local-First "Rosetta Stone" Dynamic Immersion & Acoustic Phonetics Engine**  
*Engineered for Apple Silicon (`M1` / `Metal` / `CoreML` / `MLX`) using 100% Open-Source Models and Public Corpora.*

---

## 1. Overview

**KaizenLinguistics** is a personal, proprietary language acquisition system designed to replicate and surpass the pedagogical strengths of the [Rosetta Stone Dynamic Immersion Method](https://www.rosettastone.com/our-method) and its *TruAccent* pronunciation engine ([Rosetta Stone Blog: Dynamic Immersion](https://blog.rosettastone.com/dynamic-immersion-learning-with-rosetta-stone/)) while operating **100% offline, at zero cost, with zero cloud API dependencies**.

Every aspect of the system is built on verifiable, deterministic systems engineering principles:
* **Zero-L1 Inductive Immersion:** Teaches vocabulary, morphology, and syntax exclusively in the target language ($L_2$) using $2 \times 2$ minimal-pair visual scenes, native/neural audio, and inductive bound-morpheme diff highlighting.
* **Proposer-Verifier Course Compilation:** An offline compiler (`kaizen-compiler`) synthesizes structured 20-unit courses from open linguistic corpora ([Kaikki.org Wiktionary](https://kaikki.org/dictionary/rawdata.html), [Wikidata Lexicographical Data](https://www.wikidata.org/wiki/Wikidata:Lexicographical_data), [Universal Dependencies](https://universaldependencies.org/), [Tatoeba](https://en.wikipedia.org/wiki/Tatoeba), [Mozilla Common Voice](https://commonvoice.mozilla.org/), and [Open Images V7](https://docs.ultralytics.com/datasets/detect/open-images-v7)), gated by a deterministic **Verifier** using [Apple MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip) semantic margin checks and [Montreal Forced Aligner](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner) phoneme boundaries.
* **Sub-Phonemic "OpenTruAccent" Speech Engine:** Combines [ggml-org/whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com) (CoreML/Metal) with [facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft) CTC phoneme posteriors and Dynamic Time Warping ([Halleck45/OpenPronounce](https://github.com/Halleck45/OpenPronounce)) to score pronunciation at the individual IPA phoneme, vowel duration, and pitch-contour level—paired with `24 kHz` local neural speech synthesis via [Kokoro-82M on MLX](https://github.com/gabrimatic/kokoro-mlx) ([Neywa-Labs/mlx-audio](https://github.com/Neywa-Labs/mlx-audio)).
* **Coupled Multi-Modal FSRS-5 Scheduling:** Integrates [open-spaced-repetition/fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) ([FSRS Algorithm Wiki](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm)) across coupled Listening, Reading, Speaking, and Writing stability vectors so every concept is reviewed at its mathematically optimal $90\%$ retrievability threshold.

---

## 2. Foundational Documentation Suite

| Document | Architectural Scope & Contents |
| :--- | :--- |
| **[`docs/01-SYSTEM-ARCHITECTURE.md`](./docs/01-SYSTEM-ARCHITECTURE.md)** | Hardware topology for Apple M1 (16 GB RAM, `~1.23 GB` active footprint), Proposer-Verifier duality, [Tauri v2](https://github.com/tauri-apps/tauri) desktop runtime vs. offline compiler separation, and sub-250ms real-time audio IPC flow. |
| **[`docs/02-PEDAGOGY-AND-FSRS-ENGINE.md`](./docs/02-PEDAGOGY-AND-FSRS-ENGINE.md)** | Mathematical specification of $2 \times 2$ Visual-Semantic Minimal-Pair Quads, 6-stage scaffolded modality progression, coupled multi-modal [FSRS](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm) equations, collateral transfer matrix $\mathbf{\Gamma}$, and deterministic telemetry-to-grade mapping. |
| **[`docs/03-SPEECH-AND-TRUACCENT-PIPELINE.md`](./docs/03-SPEECH-AND-TRUACCENT-PIPELINE.md)** | Complete acoustic engineering blueprint for **OpenTruAccent**: [whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com) + [wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft), Log-Posterior Ratio Goodness of Pronunciation ($\text{GoP}_{\text{LPR}}$), `pYIN` $F_0$ tone scoring, [espeak-ng](https://github.com/espeak-ng/espeak-ng) / [epitran](https://github.com/dmort27/epitran) G2P alignment, and [Kokoro-82M MLX](https://github.com/gabrimatic/kokoro-mlx) TTS. |
| **[`docs/04-OPEN-CORPUS-AND-ASSET-PIPELINE.md`](./docs/04-OPEN-CORPUS-AND-ASSET-PIPELINE.md)** | Zero-cost ETL data pipeline joining [Wikidata Lexemes](https://www.wikidata.org/wiki/Wikidata:Lexicographical_data), [Kaikki.org](https://kaikki.org/dictionary/rawdata.html) ([wiktextract](https://github.com/tatuylonen/wiktextract)), [Universal Dependencies](https://universaldependencies.org/), [Tatoeba](https://en.wikipedia.org/wiki/Tatoeba), [Mozilla Common Voice](https://commonvoice.mozilla.org/), and [Open Images V7](https://docs.ultralytics.com/datasets/detect/open-images-v7) with [Apple MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip) verification. |
| **[`docs/05-DATA-SCHEMA-AND-STATE-MACHINE.md`](./docs/05-DATA-SCHEMA-AND-STATE-MACHINE.md)** | Complete SQLite `STRICT` DDL schema for immutable `.klpack` course archives and personal telemetry ledgers, plus the Rust deterministic session state machine and adaptive remediation loops. |
| **[`ROADMAP.md`](./ROADMAP.md)** | Phased implementation blueprint, repository layout, verification benchmarks, and Tranche 0–4 build milestones. |

---

## 3. Planned Repository Directory Structure

```
KaizenLinguistics/
├── README.md                                  # Master project overview & architecture index
├── ROADMAP.md                                 # Phased engineering roadmap & verification gates
├── docs/
│   ├── 01-SYSTEM-ARCHITECTURE.md              # Hardware budget, runtime topology & IPC design
│   ├── 02-PEDAGOGY-AND-FSRS-ENGINE.md         # Dynamic Immersion math & coupled FSRS-5 engine
│   ├── 03-SPEECH-AND-TRUACCENT-PIPELINE.md    # OpenTruAccent GoP, forced alignment & MLX TTS
│   ├── 04-OPEN-CORPUS-AND-ASSET-PIPELINE.md   # Open-data ETL & MobileCLIP Verifier specification
│   └── 05-DATA-SCHEMA-AND-STATE-MACHINE.md    # SQLite STRICT schemas & Rust lesson state machine
├── compiler/                                  # Offline Proposer-Verifier Course Compiler (Python uv + Rust)
│   ├── pyproject.toml
│   ├── src/
│   │   ├── ingest/                            # Kaikki, Wikidata, UD, Tatoeba, OpenImages fetchers
│   │   ├── proposer/                          # i+1 topological sorter & 2x2 quad synthesizer
│   │   ├── verifier/                          # MobileCLIP margin gate, UD diff gate & MFA aligner
│   │   ├── tts/                               # Kokoro-82M MLX & Piper TTS batch synthesis
│   │   └── packager/                          # BLAKE3 Merkle sealer -> .klpack SQLite builder
├── app/                                       # Interactive Desktop Client (Tauri v2 + SolidJS/TS)
│   ├── src-tauri/                             # Rust core daemon
│   │   ├── Cargo.toml                         # fsrs, rusqlite, cpal, whisper-rs, ort (ONNX CoreML)
│   │   └── src/
│   │       ├── audio/                         # Lock-free ring buffer, Silero VAD, pYIN F0
│   │       ├── truaccent/                     # Wav2Vec2 CTC forced aligner & GoP_LPR scorer
│   │       ├── scheduler/                     # Coupled multi-modal FSRS-rs engine
│   │       ├── session/                       # Deterministic lesson & remediation state machine
│   │       └── storage/                       # Dual-DB SQLite WAL manager (.klpack + telemetry)
│   └── src-ui/                                # Zero-L1 2x2 immersion canvas & IPA waveform UI
└── packs/                                     # Compiled, content-addressed .klpack language courses
```
# KaizenLinguistics
