# 02 — Dynamic Immersion Pedagogy & Multi-Modal FSRS Scheduling Engine

**Project:** KaizenLinguistics  
**Core Pedagogical Paradigm:** Zero-Translation Inductive Immersion + Coupled Multi-Modal Spaced Repetition  

---

## 1. Deconstructing & Advancing "Dynamic Immersion"

The core insight of [Rosetta Stone's Dynamic Immersion Method](https://www.rosettastone.com/our-method) is that adult learners can acquire a second language ($L_2$) using the same inductive pattern-recognition mechanisms as first-language ($L_1$) acquisition—by binding **visual contexts**, **spoken audio**, and **orthographic text** without $L_1$ translation ([Rosetta Stone Blog: Dynamic Immersion](https://blog.rosettastone.com/dynamic-immersion-learning-with-rosetta-stone/)).

However, traditional commercial implementations of Dynamic Immersion exhibit three well-documented structural weaknesses that **KaizenLinguistics** resolves mathematically:

| Classical Rosetta Stone Limitation | KaizenLinguistics Engineering Resolution |
| :--- | :--- |
| **1. Rigid, Non-Adaptive Repetition:** Fixed linear lesson sequences under-review difficult items and over-drill mastered items. | **Coupled Multi-Modal FSRS-5 Engine:** Integrates [open-spaced-repetition/fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs) ([FSRS Algorithm Wiki](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm)) to schedule each concept and grammatical contrast at the exact moment retrievability $R(t, S)$ drops to $0.90$. |
| **2. Opaque Grammar Induction Stalls:** Complex morphosyntax (e.g., case systems, subjunctive mood, aspectual pairs) can frustrate learners when presented purely via static photos without structural highlighting. | **Inductive Morphosyntactic Diff Highlighting:** Derived from [Universal Dependencies](https://universaldependencies.org/) treebanks, minimal-pair screens dynamically color-code bound morphemes (prefixes, suffixes, case endings, particles) *exclusively in $L_2$* once the learner makes an inductive choice. |
| **3. Coarse Pass/Fail Speech Grading:** A single opaque percentage bar fails to tell the learner *which* phoneme, vowel length, or tone contour failed. | **Articulatory Phoneme Diagnostics:** Uses [facebook/wav2vec2-xlsr-53-espeak-cv-ft](https://huggingface.co/facebook/wav2vec2-xlsr-53-espeak-cv-ft) + DTW ([Halleck45/OpenPronounce](https://github.com/Halleck45/OpenPronounce)) to pinpoint the exact mispronounced IPA phone and trigger an immediate minimal-pair remediation drill. |

---

## 2. Formal Instructional Primitives: The 2x2 Minimal-Pair Quad

The fundamental pedagogical atom of KaizenLinguistics is the **Visual-Semantic Minimal-Pair Quad** $\mathcal{Q}$. Rather than presenting isolated flashcards, every screen presents a structured $2 \times 2$ matrix of four visual scenes:

$$\mathcal{Q} = \left\{ (I_k, S_k, A_k, \mathcal{T}_k) \right\}_{k=1}^{4}$$

Where:
* $I_k$: Grounded visual scene (sourced from [Open Images V7](https://docs.ultralytics.com/datasets/detect/open-images-v7) or Wikimedia Commons and verified via [Apple MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip)).
* $S_k$: Target $L_2$ orthographic string (token-segmented with [Universal Dependencies](https://universaldependencies.org/) lemma and morphological feature tags).
* $A_k$: Time-aligned native or neural reference audio ([Mozilla Common Voice](https://commonvoice.mozilla.org/) or [Kokoro-82M MLX](https://github.com/gabrimatic/kokoro-mlx)).
* $\mathcal{T}_k$: Phonemic IPA alignment tier $\Phi_k = \{(\phi_{k,m}, t_{\text{start}}, t_{\text{end}})\}$ generated via [espeak-ng](https://github.com/espeak-ng/espeak-ng) and [Montreal Forced Aligner](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner).

### 2.1 How Grammatical Meaning is Induced Without Translation ($L_1 = \varnothing$)

To teach a new lexical item or grammatical feature $\Delta f$, the compiler constructs a Quad where pairs of panels differ by **Controlled One-Hot Semantic or Morphological Contrasts** ($\Delta f$):

1. **Lexical Noun/Verb Induction (1-Variable Contrast):**
   * Panel 1: *The boy is running.*
   * Panel 2: *The girl is running.* ($\Delta = \text{Subject Gender/Lexeme}$)
   * Panel 3: *The boy is eating.* ($\Delta = \text{Verb Predicate}$)
   * Panel 4: *The girl is eating.* ($\Delta = \text{2x2 Factorial Product}$)
2. **Morphological Inflection Induction (Number / Gender / Case):**
   * Panel 1: *Un caballo blanco* (1 white horse)
   * Panel 2: *Dos caballos blancos* (2 white horses — highlights `-s` plural agreement across noun and adjective)
   * Panel 3: *Una casa blanca* (1 white house — highlights `-a` feminine agreement)
   * Panel 4: *Dos casas blancas* (2 white houses — highlights `-as` feminine plural agreement)
3. **Syntactic & Aspectual Induction (Tense / Aspect / Prepositions):**
   * Sequences of 3 chronological frames depicting an action *about to happen* (Future / Prospective), *in progress* (Present Continuous), and *completed* (Past / Perfective).

---

## 3. Six-Stage Scaffolded Modality Progression

Each concept node $c \in \mathcal{C}$ progresses through six ordered cognitive stages within a unit before graduating to long-term FSRS maintenance. No stage uses $L_1$ text:

| Stage | Modality Code | Stimulus Presented to Learner | Required Learner Action | Cognitive Objective |
| :--- | :--- | :--- | :--- | :--- |
| **Stage 1: Bimodal Exposure & Discovery** | `EXP_BIMODAL` | 4 Images + Spoken $L_2$ Audio + Written $L_2$ Text | Select the matching image $I^*$ from the $2\times 2$ grid | Bind visual concept to simultaneous phonetic and orthographic form |
| **Stage 2: Pure Auditory Comprehension** | `COMP_LISTEN` | 4 Images + Spoken $L_2$ Audio *(No Text)* | Select the matching image $I^*$ using auditory cues alone | Prevent orthographic dependency; train real-time phoneme stream parsing |
| **Stage 3: Pure Orthographic Reading** | `COMP_READ` | 4 Images + Written $L_2$ Text *(Audio plays only after selection)* | Select the matching image $I^*$ from written morphology | Train visual recognition of inflections, particles, and script |
| **Stage 4: Scaffolded Imitation Speaking** | `PROD_REPEAT` | Target Image $I^*$ + Reference Audio $A^*$ + IPA/Waveform Prompt | Speak the phrase aloud into the microphone | Calibrate articulatory motor patterns with immediate GoP phoneme feedback |
| **Stage 5: Unprompted Visual Speaking** | `PROD_SPEAK` | Target Image $I^*$ *(Contextual visual cue only; no audio prompt)* | Produce the correct $L_2$ utterance from memory | Active spoken retrieval + morphosyntactic assembly + pronunciation scoring |
| **Stage 6: Cloze & Orthographic Dictation** | `PROD_WRITE` | Target Image $I^*$ + Spoken Audio $A^*$ | Type or assemble the exact $L_2$ orthographic string (including diacritics) | Consolidate spelling, morphological endings, and orthographic precision |

---

## 4. Mathematical Specification of the Coupled FSRS Engine

KaizenLinguistics uses the **Free Spaced Repetition Scheduler** ([FSRS Algorithm Wiki](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm), implemented in Rust via [open-spaced-repetition/fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs)), which is grounded in the **DSR (Difficulty, Stability, Retrievability)** Three-Component Model of Memory ([ABC of FSRS](https://github.com/open-spaced-repetition/fsrs4anki/wiki/abc-of-fsrs)).

### 4.1 Core FSRS Memory Equations

For any individual skill card $k$, its memory state at time $t$ (days elapsed since last review) is parameterized by:
* **Stability ($S > 0$):** The time interval (in days) at which the probability of successful recall (Retrievability $R$) equals $90\%$.
* **Difficulty ($D \in [1, 10]$):** The intrinsic complexity of the item for the learner.
* **Retrievability ($R \in (0, 1]$):** The instantaneous probability of recall after elapsed time $t$, governed by the power-law forgetting curve in FSRS-4.5 / FSRS-5 ([FSRS Algorithm Wiki](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm)):

$$R(t, S) = \left(1 + \text{FACTOR} \cdot \frac{t}{S}\right)^{\text{DECAY}}$$

Where $\text{DECAY} = -w_{20}$ (default $-0.5$) and $\text{FACTOR} = 0.9^{1/\text{DECAY}} - 1$ (default $\frac{19}{81}$), ensuring $R(S, S) = 0.90$.

When a learner completes an exercise with discrete rating $G \in \{1 \text{ (Again)}, 2 \text{ (Hard)}, 3 \text{ (Good)}, 4 \text{ (Easy)}\}$:
* **Next Difficulty $D'$:**Updated via mean-reverting linear damping toward initial difficulty $D_0(4)$:
  $$D' = w_7 \cdot D_0(4) + (1 - w_7) \cdot \left(D - w_6 \cdot (G - 3)\right)$$
* **Post-Recall Stability $S'_r(D, S, R, G)$ (for $G \in \{2, 3, 4\}$):**
  $$S'_r(D, S, R, G) = S \cdot \left( 1 + e^{w_8} \cdot (11 - D) \cdot S^{-w_9} \cdot \left(e^{w_{10}(1 - R)} - 1\right) \cdot w_{\text{penalty/bonus}}(G) \right)$$
* **Post-Lapse Stability $S'_f(D, S, R)$ (for $G = 1$):**
  $$S'_f(D, S, R) = w_{11} \cdot D^{-w_{12}} \cdot \left((S + 1)^{w_{13}} - 1\right) \cdot e^{w_{14}(1 - R)}$$

### 4.2 Multi-Modal Skill Coupling Matrix

Unlike standard flashcard apps where cards are isolated, a linguistic concept $c$ in KaizenLinguistics has **four distinct but coupled skill cards**:

$$\mathbf{S}_c = \begin{bmatrix} S_{c,\text{listen}} \\ S_{c,\text{read}} \\ S_{c,\text{speak}} \\ S_{c,\text{write}} \end{bmatrix}$$

When the learner reviews modality $m \in \{\text{listen}, \text{read}, \text{speak}, \text{write}\}$ and achieves stability increment $\Delta S_{c,m} = S'_{c,m} - S_{c,m}$ (with $G \ge 3$), passive collateral credit is propagated to sibling modalities $n \neq m$ using a conservative **Collateral Transfer Matrix** $\mathbf{\Gamma}$ so the user is not spammed with redundant reviews of the same word on the same day:

$$S_{c,n} \leftarrow S_{c,n} + \gamma_{m \to n} \cdot \max(0, \Delta S_{c,m}) \cdot (1 - R_{c,n})$$

| Active Reviewed Modality ($m$) | $\to \text{Listen}$ ($\gamma_{m,\text{listen}}$) | $\to \text{Read}$ ($\gamma_{m,\text{read}}$) | $\to \text{Speak}$ ($\gamma_{m,\text{speak}}$) | $\to \text{Write}$ ($\gamma_{m,\text{write}}$) | Pedagogical Rationale |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **Listening (`COMP_LISTEN`)** | `1.00` | `0.15` | `0.10` | `0.05` | Receptive audio strengthens phonological representation, slightly aiding speaking |
| **Reading (`COMP_READ`)** | `0.10` | `1.00` | `0.05` | `0.20` | Receptive orthography partially reinforces spelling recall |
| **Speaking (`PROD_SPEAK`)** | `0.35` | `0.15` | `1.00` | `0.10` | Active spoken production strongly reinforces auditory recognition |
| **Writing (`PROD_WRITE`)** | `0.15` | `0.40` | `0.10` | `1.00` | Active orthographic dictation strongly reinforces reading recognition |

*(Note: Active production modalities `PROD_SPEAK` and `PROD_WRITE` can never reach "Mastered" status purely via passive transfer from `COMP_LISTEN` or `COMP_READ`; $\gamma$ only nudges sibling intervals when $R_{c,n} < 0.95$ to prevent same-session collision).*

---

## 5. Deterministic Mapping from Continuous Telemetry to FSRS Grades ($G \in \{1..4\}$)

In commercial Rosetta Stone, user performance is never fed into an explicit psychomotor memory model. In KaizenLinguistics, the learner never manually clicks "Hard / Good / Easy" buttons (which introduces metacognitive bias). Instead, the Rust engine **computes the FSRS grade $G$ deterministically** from objective response telemetry:

### 5.1 Receptive Modalities (`COMP_LISTEN`, `COMP_READ`)
Given choice accuracy $a \in \{0, 1\}$, response latency $t_{\text{resp}}$ (in milliseconds), and the learner's rolling median latency $\mu_{\text{lat}}$ for that sentence length:
* **$G = 1$ (`Again`):** $a = 0$ (Selected an incorrect distractor panel in the $2\times 2$ quad).
* **$G = 2$ (`Hard`):** $a = 1$, but required audio replay $\ge 2$ times OR $t_{\text{resp}} > 2.2 \cdot \mu_{\text{lat}}$.
* **$G = 3$ (`Good`):** $a = 1$, zero or one audio replay, and $0.8 \cdot \mu_{\text{lat}} \le t_{\text{resp}} \le 2.2 \cdot \mu_{\text{lat}}$.
* **$G = 4$ (`Easy`):** $a = 1$, zero audio replays, first-attempt selection with $t_{\text{resp}} < 0.8 \cdot \mu_{\text{lat}}$.

### 5.2 Productive Speaking Modality (`PROD_SPEAK` via OpenTruAccent)
Given word-level transcription match $W_{\text{match}}$, utterance mean Goodness of Pronunciation $\overline{\text{GoP}}$, and minimum critical phoneme score $\min_m \text{GoP}(\phi_m)$:
* **$G = 1$ (`Again`):** Lexical mismatch ($W_{\text{match}} = 0$) OR $\overline{\text{GoP}} < -2.5$ OR any phoneme contrast critical to grammatical meaning (e.g., plural `/s/`, tense vowel) scores $< -3.0$.
* **$G = 2$ (`Hard`):** Correct words spoken ($W_{\text{match}} = 1$), passed on 2nd or 3rd retry, or $-2.5 \le \overline{\text{GoP}} < -1.4$.
* **$G = 3$ (`Good`):** First-attempt pass with $-1.4 \le \overline{\text{GoP}} < -0.6$ and all phonemes $> -2.0$.
* **$G = 4$ (`Easy`):** First-attempt native-like fluency with $\overline{\text{GoP}} \ge -0.6$, speech rate within $\pm 20\%$ of reference speaker, and intonation correlation $r_{F_0} \ge 0.80$.

### 5.3 Productive Writing / Dictation Modality (`PROD_WRITE`)
Given character-level Damerau-Levenshtein distance $d_{\text{DL}}(\hat{S}, S^*)$ and diacritic/accent accuracy:
* **$G = 1$ (`Again`):** $d_{\text{DL}} > 2$ or wrong morphological inflection suffix.
* **$G = 2$ (`Hard`):** Correct stem and inflection, but $d_{\text{DL}} \in \{1, 2\}$ (e.g., missing diacritic/accent mark corrected on immediate retry).
* **$G = 3$ (`Good`):** Exact character match ($d_{\text{DL}} = 0$) on first attempt within normal typing cadence.
* **$G = 4$ (`Easy`):** Exact character match ($d_{\text{DL}} = 0$) with zero backspaces and typing speed above 75th percentile.
