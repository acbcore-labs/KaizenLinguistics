# 03 — Speech Processing & Open-Source "TruAccent" Acoustic Pipeline

**Project:** KaizenLinguistics  
**Subsystem:** Real-Time Acoustic Phonetic Assessment, Forced Alignment & Neural Speech Synthesis  
**Hardware Optimization:** Apple Silicon M1 (`Metal`, `CoreML` ANE, `MLX`)  

---

## 1. Engineering Objective: Surpassing Proprietary "TruAccent"

Rosetta Stone's proprietary *TruAccent* speech engine evaluates learner pronunciation against native speaker models ([Rosetta Stone Method](https://www.rosettastone.com/our-method)), but operates as a closed black box that returns a single scalar score without showing the learner *which specific articulatory gesture* (tongue height, voice onset time, vowel duration, or pitch contour) caused a failure.

**KaizenLinguistics OpenTruAccent** replaces this with a transparent, sub-phonemic acoustic pipeline built exclusively on open-source weights and algorithms:
1. **Orthographic & Word Boundary Recognition:** [ggml-org/whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com) (or [KalebJS/whispermlx](https://github.com/KalebJS/whispermlx)) running on Apple Silicon Metal/CoreML ([Whisper.cpp Review](https://www.promptquorum.com/power-local-llm/whisper-cpp-review)).
2. **Multilingual IPA Frame Posterior Extraction:** [facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft) (and [facebook/wav2vec2-lv-60-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-lv-60-espeak-cv-ft)) or [barathanasln/phonetic-whisper-mlx-broad-multi](https://huggingface.co/barathanasln/phonetic-whisper-mlx-broad-multi), mapping 16 kHz audio frames directly to International Phonetic Alphabet (IPA) probability distributions.
3. **Constrained Forced Alignment & Goodness of Pronunciation (GoP):** Dynamic Time Warping (DTW) and Viterbi CTC forced alignment modeled on [Halleck45/OpenPronounce](https://github.com/Halleck45/OpenPronounce), [lingjzhu/charsiu](https://github.com/lingjzhu/charsiu), and [Montreal Forced Aligner (MFA)](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner).
4. **Articulatory Feature Diagnostics:** Mapping target vs. spoken IPA phones into 24-dimensional articulatory feature vectors via [dmort27/epitran](https://github.com/dmort27/epitran) and [espeak-ng](https://github.com/espeak-ng/espeak-ng) to generate human-readable coaching on *how* to adjust the mouth/tongue (whilst keeping active lesson UI visual via vocal-tract sagittal animations).
5. **High-Fidelity Local Neural TTS:** [Kokoro-82M](https://blaizzy.github.io/mlx-audio/models/tts/kokoro/) running natively on Apple Silicon Metal via [gabrimatic/kokoro-mlx](https://github.com/gabrimatic/kokoro-mlx) and [Neywa-Labs/mlx-audio](https://github.com/Neywa-Labs/mlx-audio).

---

## 2. End-to-End Acoustic Pipeline Architecture

```
Learner Microphone Input (48kHz CoreAudio -> 16kHz Mono PCM f32 Ring Buffer)
       │
       ▼
+-------------------------------------------------------------------------+
| Stage 1: Preprocessing & Voice Activity Detection (VAD)                 |
| - High-pass biquad filter (fc = 70 Hz) + peak loudness normalization    |
| - Silero VAD / WebRTC VAD endpoint detection (t_onset, t_offset)        |
+-------------------------------------------------------------------------+
       │
       ├──────────────────────────────────────┐
       ▼                                      ▼
+----------------------------------+   +----------------------------------+
| Stage 2A: Lexical/Word Head      |   | Stage 2B: Phonetic CTC Head      |
| - ggml-org/whisper.cpp (CoreML)  |   | - wav2vec2-xlsr-53-espeak-cv-ft  |
| - Grammar-constrained decoding   |   | - 50Hz frame rate (20ms stride)  |
| - Word timestamps [w_i, t0, t1]  |   | - Log-posterior matrix L (T x V) |
+----------------------------------+   +----------------------------------+
       │                                      │
       └──────────────────┬───────────────────┘
                          ▼
+-------------------------------------------------------------------------+
| Stage 3: Constrained CTC Viterbi / DTW Forced Alignment                 |
| - Target IPA sequence Φ = (φ_1, φ_2, ..., φ_M) from Kaikki / espeak-ng  |
| - Aligns learner frames [τ_start(m), τ_end(m)] to each target phone φ_m |
+-------------------------------------------------------------------------+
       │
       ▼
+-------------------------------------------------------------------------+
| Stage 4: Multi-Dimensional Acoustic Scoring Engine                      |
| 1. Phoneme Goodness of Pronunciation: GoP(φ_m) via Log-Posterior Ratio  |
| 2. Temporal Rhythm & Gemination Score: Duration ratio vs. MFA reference |
| 3. Prosody / Tone Contour Score: pYIN F0(t) DTW distance vs. reference  |
+-------------------------------------------------------------------------+
       │
       ▼
+-------------------------------------------------------------------------+
| Stage 5: Visual Feedback & FSRS Grade Synthesis                         |
| - Color-coded IPA + Orthographic Grapheme spans (Green / Amber / Red)   |
| - Sagittal vocal-tract SVG animation for substituted phonemes           |
+-------------------------------------------------------------------------+
```

---

## 3. Mathematical Formulation of Phoneme Assessment (OpenTruAccent)

### 3.1 Target Phonemic Representation (G2P)
For every target sentence $S^*$ in a lesson, the offline compiler resolves the canonical IPA sequence $\Phi_{\text{target}} = (\phi_1, \phi_2, \dots, \phi_M)$ using a two-tier lookup:
1. **Primary Dictionary Lookup:** Exact human-curated IPA transcriptions extracted from Wiktionary via [tatuylonen/wiktextract](https://github.com/tatuylonen/wiktextract) and [Kaikki.org](https://kaikki.org/dictionary/rawdata.html).
2. **Rule-Based Fallback:** Deterministic grapheme-to-phoneme conversion via [espeak-ng/espeak-ng](https://github.com/espeak-ng/espeak-ng) (or its byte-exact pure-Python port [TigreGotico/espyak](https://github.com/TigreGotico/espyak) covering 117 languages and dialects) and [bootphon/phonemizer](https://github.com/bootphon/phonemizer).

### 3.2 Frame-Level CTC Log-Posteriors
Let $\mathbf{x}_{1:T}$ denote the 16 kHz audio waveform segmented into $T$ acoustic frames (20 ms frame shift, 25 ms window) by the convolutional feature encoder of [facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft). For each frame $t \in \{1, \dots, T\}$, the model outputs logits $\mathbf{z}_t \in \mathbb{R}^{|\Sigma|}$ over the IPA vocabulary $\Sigma_{\text{IPA}} \cup \{\epsilon\}$ (where $\epsilon$ is the CTC blank token). The log-posterior probability of phone $q \in \Sigma_{\text{IPA}}$ at frame $t$ is:

$$\ell_t(q) = \log P(q \mid \mathbf{x}_t) = z_{t,q} - \log \sum_{k \in \Sigma_{\text{IPA}} \cup \{\epsilon\}} \exp(z_{t,k})$$

### 3.3 Constrained Forced Alignment & Goodness of Pronunciation (GoP)
Using constrained CTC Viterbi forced alignment (or Subsequence Dynamic Time Warping as implemented in [Halleck45/OpenPronounce](https://github.com/Halleck45/OpenPronounce) and [lingjzhu/charsiu](https://github.com/lingjzhu/charsiu)), each target phoneme $\phi_m \in \Phi_{\text{target}}$ is mapped to its contiguous frame interval $\mathcal{I}_m = [\tau_{\text{start}}(m), \tau_{\text{end}}(m)]$ of duration $N_m = |\mathcal{I}_m|$ frames.

We compute the **Log-Posterior Ratio Goodness of Pronunciation** ($\text{GoP}_{\text{LPR}}$) for phoneme $\phi_m$:

$$\text{GoP}_{\text{LPR}}(\phi_m) = \frac{1}{N_m} \sum_{t \in \mathcal{I}_m} \ell_t(\phi_m) - \max_{q \in \Sigma_{\text{IPA}}} \left( \frac{1}{N_m} \sum_{t \in \mathcal{I}_m} \ell_t(q) \right)$$

#### Properties of $\text{GoP}_{\text{LPR}}(\phi_m)$:
* **Upper Bound ($0.0$):** If the target phoneme $\phi_m$ is the most probable phoneme across interval $\mathcal{I}_m$, then $\text{GoP}_{\text{LPR}}(\phi_m) = 0.0$.
* **Substitution Detection ($< 0.0$):** If the learner substitutesphone $\hat{\phi} = \arg\max_{q} \frac{1}{N_m} \sum_{t \in \mathcal{I}_m} \ell_t(q)$ (for example, pronouncing Spanish alveolar trill `/r/` as alveolar approximant `/ɹ/` or tap `/ɾ/`), the system immediately identifies **both**:
  1. The severity of the deviation ($\text{GoP}_{\text{LPR}}(\phi_m) < \theta_{\text{warn}}$).
  2. The **exact substituted phoneme $\hat{\phi}$** the learner actually articulated!

### 3.4 Prosodic, Tonal & Temporal Rhythm Scoring
Phoneme identity alone is insufficient for tonal languages (Mandarin, Vietnamese, Thai), pitch-accent languages (Japanese, Swedish), or vowel-length/geminate languages (Japanese, Italian, Arabic, Finnish). OpenTruAccent computes two additional acoustic metrics:

1. **Normalized Phoneme Duration Score ($S_{\text{dur}}$):**
    fixes errors in vowel length (e.g., Japanese `/o/` vs. `/oː/`) and consonant gemination (e.g., Italian `/t/` vs. `/tː/`). Given speech-rate normalized duration $\tilde{d}_m = \frac{N_m}{\sum_k N_k}$ and the reference audio duration $\tilde{d}^*_m$ (pre-computed via [Montreal Forced Aligner](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner)):
   $$S_{\text{dur}}(\phi_m) = \exp\left( -\frac{\left(\log(\tilde{d}_m / \tilde{d}^*_m)\right)^2}{2\sigma_{\text{dur}}^2} \right)$$

2. **Normalized Pitch / Tone Contour Similarity ($S_{F_0}$):**
   Extracts fundamental frequency $F_0(t)$ on voiced frames using the probabilistic YIN (`pYIN`) estimator, converts to speaker-independent semitones relative to the speaker's median pitch $\mu_{F_0}$:
   $$\tilde{f}_0(t) = 12 \cdot \log_2\left(\frac{F_0(t)}{\mu_{F_0}}\right)$$
   and computes the cosine similarity and DTW slope correlation against the native reference contour $\tilde{f}^*_0(t)$.

### 3.5 Composite Utterance Score ($S_{\text{utterance}} \in [0, 100]$)
For UI display and threshold gating, per-phoneme and prosodic metrics combine into an interpretable $0\text{--}100$ score:

$$S_{\text{utterance}} = 100 \cdot \left( w_{\text{phon}} \cdot \frac{1}{M}\sum_{m=1}^{M} \exp\left(\alpha \cdot \text{GoP}_{\text{LPR}}(\phi_m)\right) + w_{\text{dur}} \cdot \bar{S}_{\text{dur}} + w_{\text{pitch}} \cdot S_{F_0} \right)$$

Where default weights are $(w_{\text{phon}}, w_{\text{dur}}, w_{\text{pitch}}) = (0.75, 0.15, 0.10)$ for non-tonal languages and $(0.55, 0.15, 0.30)$ for tonal languages.

---

## 4.Grapheme-to-Phoneme Alignment for Visual UI Coloring

Learners read orthographic text (e.g., French *"beaucoup"*), not raw IPA (`/boku/`) alone. To highlight the exact mispronounced letters in the target word alongside the IPA transcription:
* During offline compilation (`kaizen-compiler`), [dmort27/epitran](https://github.com/dmort27/epitran) and monotonic character-to-phoneme alignment tables map each orthographic grapheme span $g_j = S^*[a_j : b_j]$ to its corresponding phoneme indices $\phi_m$.
* In the example *"beaucoup"*:
  * `b` $\leftrightarrow$ `/b/`
  * `eau` $\leftrightarrow$ `/o/`
  * `c` $\leftrightarrow$ `/k/`
  * `oup` $\leftrightarrow$ `/u/` (with silent final `p` linked to the vowel span)
* When the learner mispronounces `/u/` as `/oʊ/`, both the orthographic span **`oup`** and the IPA symbol **`/u/`** glow amber/red, and clicking the span plays an isolated slow-motion native cut of that syllable alongside the learner's own recorded waveform segment $[\tau_{\text{start}}(m), \tau_{\text{end}}(m)]$.

---

## 5. Local Neural TTS Architecture (`Kokoro-82M` on MLX)

While human recordings from [Mozilla Common Voice](https://commonvoice.mozilla.org/) and Wiktionary are prioritized, complete immersion curricula require multi-speaker variety (male/female, fast/slow, conversational cadence) across thousands of minimal-pair sentences.

KaizenLinguistics standardizes on **[Kokoro-82M](https://blaizzy.github.io/mlx-audio/models/tts/kokoro/)** via [gabrimatic/kokoro-mlx](https://github.com/gabrimatic/kokoro-mlx) and [Neywa-Labs/mlx-audio](https://github.com/Neywa-Labs/mlx-audio):
* **Why `Kokoro-82M` on Apple Silicon:**
  * Only **82 million parameters** (`~350 MB` in RAM), yet matches or outperforms multi-billion-parameter TTS models in naturalness ([Best Local TTS for Mac 2026](https://www.promptquorum.com/power-local-llm/best-local-tts-mac)).
  * Executes natively on Apple Silicon GPU via **MLX** (`Metal Performance Shaders`), synthesizing `24 kHz` waveforms at `>15x` real-time speed on an M1 MacBook Pro.
  * Ships with **54 built-in voice presets** across multiple languages, allowing every $2 \times 2$ quad to rotate across distinct male and female vocal timbres so the learner's acoustic mental model generalizes across speakers rather than overfitting to a single voice.
* **Fallback Breadth Engine:** For low-resource languages outside `Kokoro-82M`'s core set, the compiler falls back to **Piper TTS** (ONNX VITS models) or **MMS-TTS** (Meta's Massively Multilingual Speech TTS covering 1,100+ languages).
