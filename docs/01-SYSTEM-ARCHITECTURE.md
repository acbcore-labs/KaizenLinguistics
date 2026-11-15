# 01 — System Architecture & Local-First Runtime Topology (v2.0 — Hardened)

**Project:** KaizenLinguistics (Personal Local-First Language Acquisition Engine)  
**Primary Languages:** Koine Greek (`grc`) & Classical / Biblical Hebrew (`hbo`)  
**Target Host Architecture:** Apple Silicon (`arm64` / M1 MacBook Pro, 16 GB Unified Memory, 1 TB NVMe SSD)  
**Cost & Licensing Constraint:** 100% Zero-Cost, Offline-First, Pure Open-Source Models & Public/CC Corpora  

---

## 1. Executive Architectural Vision & Epistemic Grounding

**KaizenLinguistics** is a personal, local-first language acquisition platform designed to combine the best of [Rosetta Stone's Dynamic Immersion](https://www.rosettastone.com/our-method) (teaching through structured perceptual contrasts rather than $L_1$ translation crutches) with the deeply annotated philological corpora of **Koine Greek (`grc`)** and **Biblical Hebrew (`hbo`)**.

Following rigorous architectural review, v2.0 establishes a strict distinction between **deterministic system guarantees** (content hashing, schema constraints, reproducible state transitions) and **probabilistic model evidence** (visual similarity scores, acoustic phoneme posteriors, memory retention estimates):

1. **The Native Primitive is a Contrast Graph (Not Isolated Flashcards):**
   Every teachable unit is modeled as a structured neighborhood of meanings, language-specific surface forms, realizations (scenes, audio, text), and **controlled minimal contrasts** derived from [Clear-Bible/macula-greek](https://github.com/Clear-Bible/macula-greek), [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew), [morphgnt/sblgnt](https://github.com/morphgnt/sblgnt), and [ETCBC/bhsa](https://github.com/ETCBC/bhsa).
2. **Forward-Corrective Pairing (Proposer + Multi-Evidence Gate + Editorial Approval):**
   Course compilation is separated from the interactive runtime. Candidate exercises generated or mined from open corpora are evaluated across **six independent evidence dimensions** (structural validity, linguistic contrast purity, visual discrimination evidence, audio/alignment confidence, license completeness, and human editorial review status). Automated model scores (such as [Apple MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip) cosine margins) serve as **candidate screening and ranking heuristics**, never as a substitute for editorial sign-off.
3. **Calibrated Acoustic Assessment with Explicit Abstention:**
   Rather than overclaiming that a neural phoneme recognizer recovers exact articulatory tongue movements, the acoustic pipeline ([ggml-org/whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com) + [facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft)) separates signal quality, unconstrained utterance verification, forced alignment confidence, and contrast assessment—explicitly returning **`ABSTAIN / UNABLE_TO_ASSESS`** whenever signal noise or alignment uncertainty exceeds calibrated thresholds.
4. **Pinned FSRS Scheduling with Observed-Only Stability Updates:**
   Integrates a single pinned version of [open-spaced-repetition/fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) ([FSRS Algorithm Wiki](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm)). Listening, Reading, Speaking, and Writing maintain independent review histories updated strictly from direct observations of that skill, using **sibling spacing/burying** to prevent repetitive same-day drills while logging cross-modality transfer hypotheses in shadow mode.

---

## 2. Forward-Corrective Architectural Axioms

Every generative, inferential, or scheduling operation in KaizenLinguistics is paired with an explicit **corrective counterpart** so unverified model outputs never silently corrupt curriculum quality or learner mastery records:

| System Layer | Forward Operation | Corrective Counterpart |
| :--- | :--- | :--- |
| **1. Curriculum Synthesis** | Propose a contrast exercise from authored templates + [MACULA](https://github.com/Clear-Bible/macula-hebrew) / [MorphGNT](https://github.com/morphgnt/sblgnt) corpora | Multi-evidence inspection + explicit rejection codes (`CONFOUNDED_CONTRAST`, `AMBIGUOUS_SCENE`, `LOW_ALIGNMENT_CONF`) + editorial approval gate |
| **2. Learner Comprehension** | Infer grammatical or lexical understanding from a correct choice | Probe understanding with an unprompted permutation or novel lexical instantiation of the same grammatical contrast |
| **3. Speech & Phonology** | Score spoken pronunciation via acoustic alignment & phoneme posteriors | **Abstain (`UNABLE_TO_ASSESS`)** and request a clean re-recording when SNR, VAD clipping, or alignment confidence is low |
| **4. Multi-Skill Scheduling** | Bury/space sibling modalities (`LISTEN`, `READ`, `SPEAK`, `WRITE`) to avoid collision | Require direct active testing of the destination modality (`SPEAK` / `WRITE`) before advancing its FSRS stability $S$ |
| **5. Course Pack Evolution** | Publish an updated `.klpack` course version | Migrate unaffected concept states via stable `concept_id` + `contrast_id` keys and safely invalidate modified exercises |

---

## 3. High-Level System Topology

```
+-----------------------------------------------------------------------------------+
|                        KAIZEN LINGUISTICS DESKTOP CLIENT                          |
|                   Tauri v2 Application Shell (macOS WKWebView)                    |
|  +-----------------------+  +------------------------+  +----------------------+  |
|  | Multi-Modal Contrast  |  | Polytonic Greek &      |  | Scaffolded Authentic |  |
|  | Canvas (2x2 / Aspect  |  | Tiberian Hebrew Script |  | Text Reader (MACULA  |  |
|  | Strip / Dialogue)     |  | + Audio Waveform HUD   |  | Clause Onion-Peeler) |  |
|  +-----------+-----------+  +------------+-----------+  +----------+-----------+  |
+--------------|---------------------------|-------------------------|--------------+
               | Tauri IPC Commands        | Zero-Copy Audio Stream  |
+--------------v---------------------------v-------------------------v--------------+
|                         RUST CORE RUNTIME DAEMON (src-tauri)                      |
|  +-----------------------+  +------------------------+  +----------------------+  |
|  | Deterministic Session |  | Pinned fsrs-rs Engine  |  | 5-Stage Acoustic     |  |
|  | & Remediation Engine  |  | + Sibling Burying      |  | Evaluator + Abstain  |  |
|  +-----------+-----------+  +------------+-----------+  +----------+-----------+  |
|              |                           |                         |              |
|  +-----------v---------------------------v-------------------------v-----------+  |
|  |        Dual SQLite 3 Storage (WAL Mode) + App-Level Integrity Verifier      |  |
|  |  Read-Only: course_<lang>.klpack   |  Read-Write: user_telemetry.db         |  |
|  +-----------------------------------------------------------------------------+  |
+-----------------------------------------|-----------------------------------------+
                                          | In-Process FFI / Local UNIX Socket
+-----------------------------------------v-----------------------------------------+
|                 LOCAL APPLE SILICON (M1 METAL / ANE / MLX) INFERENCE              |
|  +-------------------------+ +-------------------------+ +---------------------+  |
|  | Unconstrained ASR Check | | Phoneme Posterior &     | | Reference Audio     |  |
|  | ggml-org/whisper.cpp    | | Alignment Confidence    | | Curated Human Clips |  |
|  | (CoreML ANE + Metal)    | | wav2vec2-xlsr-53-espeak | | + Local TTS Backup  |  |
|  +-------------------------+ +-------------------------+ +---------------------+  |
+-----------------------------------------------------------------------------------+
                                          ^
                                          | Versioned & Editorially Approved (.klpack)
+-----------------------------------------|-----------------------------------------+
|               OFFLINE ASSISTED COURSE COMPILER (Python uv + Rust)                 |
|  +-----------------------------------------------------------------------------+  |
|  | Biblical & Classical Corpora: ETCBC BHSA | Clear-Bible MACULA (Greek/Heb)   |  |
|  | MorphGNT SBLGNT | OpenScriptures morphhb & BDB | Abbott-Smith | UD Treebanks|  |
|  | Aleph with Beth & Alpha with Angela (CC Scripts/Visuals) | Open Images V7   |  |
|  +--------------------------------------+--------------------------------------+  |
|                                         |                                         |
|  +--------------------------------------v--------------------------------------+  |
|  | 6-Part Evidence Gate: Schema + Single-Delta Contrast Check + MobileCLIP     |  |
|  | Screening + MFA Alignment Confidence + License Audit + Human Review Signoff |  |
|  +-----------------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------------+
```

---

## 4. Reconciled Resource & Latency Engineering Targets on Apple M1 (16 GB RAM)

To eliminate discrepancies between architectural estimates and benchmark gates, v2.0 defines **one unified resource table** and separates **VAD endpointing window** from **post-endpoint compute latency**:

### 4.1 Unified Resident Memory Budget (16 GB MacBook Pro M1)
* **Hard Architectural Ceiling:** `< 2.0 GB` total resident memory (`RSS` + `Metal` wired buffers) across all runtime processes, leaving `> 14.0 GB` unified memory free for macOS and developer workflows.
* **Component Breakdown (Target Working Set: `~1.23 GB` when all models are warm):**

| Subsystem Layer | Open-Source Component | License | Execution Backend on M1 | Estimated Resident RAM | Role in Runtime vs. Compiler |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Desktop Shell & UI** | [tauri-apps/tauri](https://github.com/tauri-apps/tauri) (v2) + SolidJS / TypeScript | MIT / Apache-2.0 | Native `WKWebView` + Rust Core | `~95 MB` | **Runtime:** 60fps UI, polytonic Greek & pointed Tiberian Hebrew typography |
| **Spaced Repetition Core** | [open-spaced-repetition/fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) (`fsrs` crate, pinned) | MIT | Native Rust (`aarch64-apple-darwin`) | `< 15 MB` | **Runtime:** Per-modality DSR scheduling, sibling burying, parameter fitting |
| **Unconstrained ASR Check** | [ggml-org/whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com) (`whisper-base` / `small` Q5_1) | MIT | CoreML (ANE) + Metal F16 | `~280 MB` | **Runtime (Milestone 3+):** Utterance verification & word boundaries |
| **Phoneme Posterior Head** | [facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft) (ONNX) | Apache-2.0 | ONNX Runtime (`CoreMLExecutionProvider`) | `~420 MB` | **Runtime (Milestone 3+):** IPA frame posteriors & alignment confidence |
| **Rule-Based G2P (`grc`/`hbo`)** | Custom Deterministic Rust/Python G2P + [espeak-ng](https://github.com/espeak-ng/espeak-ng) / [epitran](https://github.com/dmort27/epitran) | MIT / GPLv3 | Native Rust / Python | `< 10 MB` | **Both:** Exact Tiberian Hebrew & Restored/Erasmian Koine Greek IPA mapping |
| **Local Neural TTS** | [gabrimatic/kokoro-mlx](https://github.com/gabrimatic/kokoro-mlx) / [Neywa-Labs/mlx-audio](https://github.com/Neywa-Labs/mlx-audio) + MMS-TTS | Apache-2.0 / CC-BY-NC | Apple MLX / ONNX | `~350 MB` | **Compiler (Primary) / Runtime (Optional):** Synthesizing supplemental audio |
| **Embedded Storage** | SQLite 3 (`rusqlite`, WAL mode) | Public Domain | Native C/Rust in-process | `~60 MB` | **Runtime:** `.klpack` course reader + `user_telemetry.db` writer |
| **Visual & Alignment Tools** | [apple-aiml-research/ml-mobileclip](https://github.com/apple-aiml-research/ml-mobileclip) + [Montreal-Forced-Aligner](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner) | Apple / MIT | Offline Python/Kaldi | *0 MB at Runtime* (`~900 MB` offline) | **Compiler Only:** Never loaded during study sessions |
| **Total Warm Runtime** | **All Active Study-Session Processes** | **100% Free** | **M1 Unified Memory** | **`~1.23 GB` Target (`< 2.0 GB` Cap)** | **Verified via `vmmap` / `footprint` in Milestone 1 & 3** |

### 4.2 Decomposed Speech Feedback Latency Budget
Prior drafts conflated user-perceived wait time after stopping speech with compute time after endpoint detection. We strictly decompose total turnaround time $T_{\text{total}}$ into five measurable stages to be benchmarked empirically in Milestone 3:

$$T_{\text{total}} = T_{\text{VAD\_tail}} + T_{\text{SNR\_gate}} + T_{\text{ASR\_verify}} + T_{\text{CTC\_align}} + T_{\text{UI\_render}}$$

| Stage | Metric Definition | Target Engineering Budget (M1) | Notes |
| :--- | :--- | :---: | :--- |
| **1. $T_{\text{VAD\_tail}}$** | Trailing silence window required by VAD to confirm end-of-utterance (unless user releases push-to-talk key) | `300 – 350 ms` *(0 ms on PTT release)* | Supporting spacebar/key **Push-to-Talk (PTT)** release eliminates trailing VAD wait entirely |
| **2. $T_{\text{SNR\_gate}}$** | Signal-to-noise ratio, clipping, and duration usability check | `< 5 ms` | Immediate abstention if audio is clipped or silent |
| **3. $T_{\text{ASR\_verify}}$** | Unconstrained `whisper.cpp` check for utterance identity | `< 110 ms` | Runs in parallel with Stage 4 on Metal/CoreML |
| **4. $T_{\text{CTC\_align}}$** | `wav2vec2-xlsr-53` forward pass + constrained Viterbi/DTW alignment | `< 140 ms` | Parallel with Stage 3; bounded by $\max(T_{\text{ASR}}, T_{\text{CTC}})$ |
| **5. $T_{\text{UI\_render}}$** | IPC serialization + SolidJS DOM update | `< 15 ms` | Single 60fps frame budget |
| **Total Post-Endpoint Compute** | $T_{\text{compute}} = T_{\text{total}} - T_{\text{VAD\_tail}}$ | **`< 175 ms` Target** | **`< 525 ms` total wall-clock with auto-VAD, or `< 175 ms` on PTT key release** |
