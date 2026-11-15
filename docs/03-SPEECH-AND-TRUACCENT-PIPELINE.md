# 03 — Speech Processing, Phonological Profiles & Calibrated Acoustic Assessment (v2.0 — Hardened)

**Project:** KaizenLinguistics  
**Primary Languages:** Koine Greek (`grc`) & Classical / Biblical Hebrew (`hbo`)  
**Subsystem:** Acoustic Phonetic Assessment, Forced Alignment, Abstention Gate & Reference Audio  

---

## 1. Epistemic Corrections: Recognition $\neq$ Articulatory Diagnosis

In v1.0, the speech specification treated phonetic posteriors from [facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft) as deterministic proof of "the exact substituted phoneme the learner articulated" and used grammar-constrained [whisper.cpp](https://github.com/ggerganov/whisper.cpp?ref=itsfoss.com) decoding to verify word identity.

Following architectural review, v2.0 corrects both flaws:
1. **A Phone Recognizer's Posterior is Model Evidence, Not Direct Articulatory Truth:**
   `wav2vec2-xlsr-53-espeak-cv-ft` is a cross-lingual CTC phone recognizer trained on [Mozilla Common Voice](https://commonvoice.mozilla.org/). Its output distribution $P(\phi \mid \mathbf{x}_t)$ reflects acoustic similarity to its training inventory—not direct measurement of tongue height or glottal state. Especially for historical reconstructions like **Restored Koine Greek (`grc`)** and **Tiberian Biblical Hebrew (`hbo`)**, acoustic scores carry model uncertainty and must be calibrated against real human recordings before gating progression.
2. **Eliminating Circular Constrained Decoding:**
   If an ASR decoder is constrained to only emit the expected target sentence $S^*$, its output cannot be used as independent evidence that the learner actually spoke $S^*$. Utterance verification must compare **unconstrained (or foil-expanded)** recognition against the target.
3. **First-Class Abstention (`UNABLE_TO_ASSESS`):**
   Background noise, microphone clipping, breath pops, or forced-alignment failure must never be reported to the learner as "You mispronounced phoneme X." Low-confidence analysis must explicitly return **`UNABLE_TO_ASSESS`**, prompt for another recording (or fall back to self-comparison playback), and **never penalize FSRS stability**.

---

## 2. Five-Stage Epistemic Separation in the Acoustic Pipeline

Every spoken response passes through five strictly ordered evaluation gates. If any upstream gate fails or falls below its confidence threshold, downstream phoneme grading is halted immediately:

```
Learner Recording (16kHz Mono PCM f32)
       │
       ▼
+-------------------------------------------------------------------------+
| GATE 1: SIGNAL QUALITY & USABILITY CHECK                                |
| - Peak clipping check (< 0.5% clipped samples)                          |
| - SNR estimation (>= 12 dB above ambient noise floor)                   |
| - Utterance duration bounds (not truncated by early VAD cutoff)         |
+------------------------------------+------------------------------------+
                                     │ Pass (Else -> ABSTAIN: LOW_SIGNAL_QUALITY)
                                     ▼
+-------------------------------------------------------------------------+
| GATE 2: UTTERANCE IDENTITY VERIFICATION (UNCONSTRAINED + FOIL SET)      |
| - Unconstrained whisper.cpp / CTC phone edit distance                   |
| - Compares likelihood of Target S* against Contrast Foil Sentences S_j  |
+------------------------------------+------------------------------------+
                                     │ Pass (Else -> INCORRECT_UTTERANCE or ABSTAIN)
                                     ▼
+-------------------------------------------------------------------------+
| GATE 3: FORCED ALIGNMENT CONFIDENCE GATE                                |
| - Aligns expected phone sequence Φ_target via Viterbi CTC / DTW         |
| - Checks frame alignment entropy & boundary monotonicity                |
+------------------------------------+------------------------------------+
                                     │ Pass (Else -> ABSTAIN: LOW_ALIGNMENT_CONF)
                                     ▼
+-------------------------------------------------------------------------+
| GATE 4: TARGETED CONTRAST & PHONEME EVIDENCE ASSESSMENT                 |
| - Evaluates Log-Posterior Ratio (GoP_LPR) & duration ratio specifically |
|   on the lesson's targeted phonological/morphological contrast spans    |
+------------------------------------+------------------------------------+
                                     │
                                     ▼
+-------------------------------------------------------------------------+
| GATE 5: CALIBRATED COACHING & SIDE-BY-SIDE WAVEFORM PLAYBACK            |
| - Always provides instant 1-click replay of Learner vs. Reference audio |
| - Surfaces phoneme/syllable highlights only when confidence >= θ_cal    |
+-------------------------------------------------------------------------+
```

---

## 3. Deterministic Grapheme-to-Phoneme (G2P) for Koine Greek & Biblical Hebrew

Modern general-purpose G2P libraries (`espeak-ng` default `el` and `he` voices) implement **Modern Greek** and **Modern Israeli Hebrew** phonology—which erases critical historical contrasts in Koine Greek and Tiberian Biblical Hebrew.

Because both **Koine Greek (polytonic orthography)** and **Biblical Hebrew (pointed Masoretic Text with *niqqud* and *te'amim*)** have **near-100% deterministic orthography-to-phonology rules**, KaizenLinguistics implements dedicated, rule-verified G2P transducers in Rust/Python:

### 3.1 Biblical Hebrew (`hbo`) Tiberian / Reconstructed G2P Engine
Reads Unicode Hebrew codepoints (`U+0591`–`U+05F4`) from [openscriptures/morphhb](https://github.com/openscriptures/morphhb) and [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew), normalizing cantillation stress and applying exact Masoretic phonological rules:
1. ***Begadkefat* Spirantization (`ב ג ד כ פ ת`):** Distinguishes plosive stops with *dagesh lene* (`/b g d k p t/`) from fricative allophones without *dagesh* (`/v ɣ ð x f θ/` in strict Tiberian, or `/v g d χ f t/` in Academic/Sephardic profile).
2. ***Dagesh Forte* (Consonant Gemination) vs. *Dagesh Lene*:** Automatically identifies *dagesh forte* after short vowels and emits geminate phoneme duration targets (`/Cː/`), which are critical for distinguishing **Piel** (`קִדֵּשׁ` `/qidˈdeːʃ/`) from **Qal** stems!
3. ***Qamatz Gadol* (`/ɔː/` or `/aː/`) vs. *Qamatz Qatan* (`/o/`):** Resolves closed unaccented syllables (using Masoretic cantillation stress accents `U+0591`–`U+05AF` and *meteg* `U+05BD`) to distinguish `/aː/` from `/o/`.
4. **Vocal *Shewa* (`/ə/`) vs. Silent *Shewa* ($\varnothing$):** Applies the five Masoretic vocal-shewa rules (word-initial, second of two consecutive shewas, under a geminated consonant with *dagesh forte*, after a long vowel with *meteg*, or before an identical consonant) and *Hataf* reduced vowels (`ֲ` `/ă/`, `ֱ` `/ĕ/`, `ֳ` `/ŏ/`).
5. **Furtive *Pathach* (`פַּתַח גְּנוּבָה`):** When word-final gutturals (`חַ`, `עַ`, `הַּ`) carry a *pathach* after a non-a long vowel (e.g., `רוּחַ`), transposes the phoneme order to `/ˈruːaħ/` rather than `/ˈruːħa/`.

### 3.2 Koine Greek (`grc`) Multi-Profile G2P Engine
Reads polytonic Greek Unicode from [morphgnt/sblgnt](https://github.com/morphgnt/sblgnt) and compiles into the user's selected phonological profile:

| Greek Grapheme | Profile A: Restored 1st-Century Koine (Buth / Kantor / *Alpha with Angela*) | Profile B: Academic Erasmian | Profile C: Modern / Liturgical Greek |
| :---: | :---: | :---: | :---: |
| `α / ᾳ` | `/a/` | `/a/` | `/a/` |
| `ε / αι` | `/ɛ/` (merged in Koine) | `/ɛ/` vs. `/ai/` | `/e/` |
| `η / ῃ` | `/eː/` (mid-close front) | `/ɛː/` or `/eɪ/` | `/i/` |
| `ι / ει` | `/i/` (merged in Koine) | `/i/` vs. `/eɪ/` | `/i/` |
| `ο / ω` | `/o/` (merged quality in Koine) | `/ɔ/` vs. `/oː/` | `/o/` |
| `υ / οι` | `/y/` (front rounded) | `/y/` or `/u/` vs. `/ɔɪ/` | `/i/` |
| `ου` | `/u/` | `/uː/` | `/u/` |
| `β, γ, δ` | `/b~β/, /g~ɣ/, /d~ð/` | `/b, g, d/` | `/v, ɣ~ʝ, ð/` |
| `φ, θ, χ` | `/pʰ, tʰ, kʰ/` (aspirated stops transitioning to `/ɸ, θ, x/`) | `/f, θ, x/` | `/f, θ, x~ç/` |
| `῾` (Rough breathing) | `/h/` (word-initial) | `/h/` | $\varnothing$ (silent) |
| `ζ` | `/z/` | `/dz/` or `/z/` | `/z/` |

---

## 4. Reference Audio Strategy for Ancient Languages

Because [Mozilla Common Voice](https://commonvoice.mozilla.org/) does not cover Classical Biblical Hebrew (`hbo`) or 1st-Century Restored Koine Greek (`grc`), KaizenLinguistics uses a **Three-Tier Reference Audio Hierarchy**:

1. **Tier 1 — Authentic Human Immersion Recordings (Highest Priority):**
   * **Biblical Hebrew (`hbo`):** Curated human audio from CC-licensed [Aleph with Beth (`freehebrew.online`)](https://freehebrew.online/the-divine-name/) lessons and public-domain cantillated/read Hebrew Bible recordings (e.g., Abraham Shmueloff / Mechon Mamre verse recordings segmented to clause level using [ETCBC/bhsa](https://github.com/ETCBC/bhsa) and [Montreal Forced Aligner](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner)).
   * **Koine Greek (`grc`):** Curated human audio from CC-licensed [Alpha with Angela (`freegreek.online`)](https://freegreek.online/about/), [kwozymoto/koine-greek (Everyday Koine)](https://github.com/kwozymoto/koine-greek), and open Restored-Koine New Testament recordings.
2. **Tier 2 — Phoneme-Conditioned Local Neural TTS Synthesis:**
   * Rather than feeding raw Hebrew or Greek orthography into a modern TTS front-end (which would apply Modern Israeli or Modern Athens pronunciation rules), our compiler runs our **Deterministic `hbo`/`grc` G2P Engine (Section 3)** first to emit **explicit IPA phoneme strings**, and feeds those phoneme sequences directly into the acoustic synthesizer ([Kokoro-82M MLX](https://github.com/gabrimatic/kokoro-mlx) / Piper ONNX / Meta `MMS-TTS-grc` & `MMS-TTS-heb` phoneme-input checkpoints).
3. **Tier 3 — Side-by-Side Acoustic Self-Monitoring (Zero-Latency Fallback):**
   * On every speaking exercise (`PROD_REPEAT` and `PROD_SPEAK`), the UI automatically records the learner, trims leading/trailing silence, normalizes loudness, and plays **[Learner Recording] $\rightarrow$ [150ms pause] $\rightarrow$ [Native Reference Audio]** with synchronized pitch ($F_0$) and stress-envelope waveforms. This guarantees high-value auditory motor calibration even before neural phoneme grading is enabled in Milestone 3.

---

## 5. Empirical Calibration Protocol (Milestone 3 Gate)

Before automated pronunciation scores are allowed to assign FSRS grades (`Again/Hard/Good/Easy`) without learner confirmation, the acoustic pipeline must be evaluated on a dedicated **Offline Calibration Dataset** (constructed in Milestone 3):
* **Test Corpus Composition:**
  * 100 native/expert reference recordings in `grc` and `hbo`.
  * 100 learner recordings containing acceptable allophonic variation (e.g., aspirated `/pʰ/` vs. fricative `/ɸ/` for Greek `φ`; Sephardic vs. Tiberian `/ħ/` for Hebrew `ח`).
  * 100 recordings with genuine phonological/morphological errors (e.g., dropping final `-ν` vs. `-ς` in Greek case endings; misplacing stress between `קָטְלָה` *3fs Qal* and `קָטְלָה` with suffix; substituting wrong vowel in Hebrew *Hiphil* vs. *Qal*).
  * 50 noisy, clipped, or off-topic recordings to verify **Abstention (`UNABLE_TO_ASSESS`)** behavior.
* **Tracked Calibration Metrics:**
  * **False Rejection Rate (FRR):** Percentage of acceptable learner utterances graded `< Good` (Target: `< 5%`).
  * **False Acceptance Rate (FAR):** Percentage of genuine contrast-destroying errors graded `>= Good` (Target: `< 10%`).
  * **Expected Calibration Error (ECE)** and **Abstention Precision** on low-SNR inputs.
