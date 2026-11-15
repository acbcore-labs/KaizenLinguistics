# 02 — Contrast-Graph Pedagogy & Pinned FSRS Scheduling Engine (v2.0 — Hardened)

**Project:** KaizenLinguistics  
**Primary Languages:** Koine Greek (`grc`) & Classical / Biblical Hebrew (`hbo`)  
**Core Pedagogical Paradigm:** Contrast-Graph Inductive Immersion + Observed-Skill FSRS-6 Scheduling  

---

## 1. From "4-Picture Flashcards" to a Formal Contrast Graph

In v1.0, every exercise was rigidly modeled as a $2 \times 2$ photo grid. As highlighted in architectural review, a static 4-photo grid works well for concrete nouns and physical actions, but fails when teaching **verbal aspect** (e.g., Koine Greek Imperfective vs. Aorist vs. Perfect), **discourse sequentiality** (Biblical Hebrew *Wayyiqtol* narrative chains), or **pragmatic dialogue particles** (`γάρ`, `οὖν`, `μέν... δέ`, `הִנֵּה`, `כִּי`).

In v2.0, the native pedagogical representation of KaizenLinguistics is a **Contrast Graph**:

```
Semantic Situation / Communicative Intent
    ↕ expressed by
Language-Specific Morphosyntactic Forms (grc / hbo)
    ↕ realized as
Multi-Modal Stimulus (Visual Scenes, Aspect Timelines, Audio Pairs, Short Dialogues)
    ↕ distinguished through
Controlled Minimal-Contrast Exercises (holding all non-target variables constant)
    ↕ observed through
Learner Responses (with recorded latency, retries, and acoustic/orthographic telemetry)
```

### 1.1 Five Fundamental Axioms of the Contrast Graph
1. **Concept Identity $\neq$ Word Identity:** A single semantic situation (e.g., *proclaiming good news*) maps to distinct language-specific lexemes and argument frames (`εὐαγγελίζομαι + Dat/Acc` in Greek vs. `בִּשֵּׂר` *Piel* in Hebrew).
2. **Strict Single-Delta Contrast Design (No Confounded Variables):**
   * *The Confounding Bug to Avoid:* Contrasting Spanish *un caballo blanco* (masc.) with *una casa blanca* (fem.) changes **both** the noun lexeme and the grammatical gender simultaneously—the learner cannot tell whether `-o/-a` marks gender or object type.
   * *The Hardened Rule:* Every minimal-pair contrast exercise must hold the lexical roots or predicates **strictly invariant** across panels so the targeted grammatical feature is the *only* covarying linguistic signal.
3. **Presentation Geometry Matches the Contrast Type:**
   * **2x2 Scene Grid (`QUAD_SCENE`):** For lexical nouns, verbs, spatial prepositions, number, gender, and Agent/Patient case assignment.
   * **3-Frame Aspectual Timeline (`ASPECT_STRIP`):** For contrasting ongoing process (Greek Present/Imperfect; Hebrew Participle/*Yiqtol*), bounded event snapshot (Greek Aorist; Hebrew *Qatal*), and resultant state (Greek Perfect).
   * **2-Turn Dialogue / Discourse Micro-Scene (`DIALOGUE_PAIR`):** For questions/answers, imperatives/cohortatives (`שְׁמַע` vs. `נִשְׁמְעָה`), and discourse connectors (`οὖν`, `γάρ`, `וַיְהִי`, `הִנֵּה`).
4. **An Exercise Measures a Contrast; It Is Not the Skill Itself:** Passing a single multiple-choice screen is a noisy observation of recognition, not proof of mastery. Mastery requires successful generalization to held-out instantiations of the same contrast.

---

## 2. Worked Unconfounded Contrast Examples (`grc` & `hbo`)

### 2.1 Koine Greek Case & Word-Order Independence (`QUAD_SCENE`)
English speakers instinctively interpret the first noun in a sentence as the Subject (Agent). In Koine Greek ([Clear-Bible/macula-greek](https://github.com/Clear-Bible/macula-greek)), case endings—not word order—determine who acts upon whom. To force the learner's brain to parse Nominative (`-ος`) vs. Accusative (`-ον`), the exercise holds the two participants and the verb constant while crossing **Agent/Patient role** with **Word Order**:

* **Scene A (Angel sees Brother):**
  * Form 1 (SVO): `ὁ ἄγγελος βλέπει τὸν ἀδελφόν.`
  * Form 2 (OVS — same meaning!): `τὸν ἀδελφόν βλέπει ὁ ἄγγελος.`
* **Scene B (Brother sees Angel):**
  * Form 3 (SVO): `ὁ ἀδελφός βλέπει τὸν ἄγγελον.`
  * Form 4 (OVS — same meaning!): `τὸν ἄγγελον βλέπει ὁ ἀδελφός.`

Here, lexical items (`ἄγγελος`, `ἀδελφός`, `βλέπει`) are 100% invariant; the learner *must* attend to `ὁ ... -ος` (Agent) vs. `τὸν ... -ον` (Patient) to select the right scene.

### 2.2 Biblical Hebrew Gender & Verbal Agreement (`QUAD_SCENE`)
Following the proven visual-immersion progression of [Aleph with Beth (`freehebrew.online`)](https://dantes.io/resource/aleph-with-beth-free-biblical-hebrew-video-course-6abc0ecdf769b646f61723db) and [ETCBC/bhsa](https://github.com/ETCBC/bhsa), gender and number agreement are introduced with invariant actions across matched human subjects:

* **Panel 1 (Masc. Sg.):** `הַנַּעַר יֹשֵׁב עַל הָאֶבֶן` (*The boy is sitting on the stone* — zero ending on participle `יֹשֵׁב`)
* **Panel 2 (Fem. Sg.):** `הַנַּעֲרָה יֹשֶׁבֶת עַל הָאֶבֶן` (*The girl is sitting on the stone* — `-ָה` noun + `-ֶת` fem. sg. participle)
* **Panel 3 (Masc. Pl.):** `הַנְּעָרִים יֹשְׁבִים עַל הָאֶבֶן` (*The boys are sitting on the stone* — `-ִים` masc. pl. across noun & participle)
* **Panel 4 (Fem. Pl.):** `הַנְּעָרוֹת יֹשְׁבוֹת עַל הָאֶבֶן` (*The girls are sitting on the stone* — `-וֹת` fem. pl. across noun & participle)

### 2.3 Biblical Hebrew *Binyanim* Valency Contrast (`QUAD_SCENE`)
To teach how Hebrew verbal stems (*binyanim*) transform causation and voice on the exact same triliteral root (e.g., `י-שׁ-ב` *to sit/dwell* or `שׁ-מ-ע` *to hear*):
* **Qal (Simple Active):** `הַיֶּלֶד יָשַׁב עַל הַכִּסֵּא` (*The child sat on the chair*)
* **Hiphil (Causative Active):** `הָאִשָּׁה הוֹשִׁיבָה אֶת־הַיֶּלֶד עַל הַכִּסֵּא` (*The woman seated [caused to sit] the child on the chair* — highlights `הוֹ- / -ִ-` *Hiphil* template + `אֶת־` object marker).

---

## 3. Six-Stage Scaffolded Modality Progression

Each contrast node progresses through six ordered cognitive stages before graduating to long-term FSRS review:

| Stage | Modality Code | Stimulus Presented | Learner Action | Tracked FSRS Skill Card |
| :--- | :--- | :--- | :--- | :--- |
| **1. Bimodal Discovery** | `EXP_BIMODAL` | Visual Contrast + Spoken $L_2$ Audio + Written $L_2$ Text | Select matching scene + inspect highlighted morpheme contrast | *(Onboarding scaffold)* |
| **2. Auditory Discrimination** | `COMP_LISTEN` | Visual Contrast + Spoken $L_2$ Audio *(No Text)* | Identify the meaning purely from spoken phonology and morphology | `LISTEN` |
| **3. Orthographic Reading** | `COMP_READ` | Visual Contrast + Written $L_2$ Text *(Audio plays post-choice)* | Identify the meaning from script, diacritics/niqqud, and case/stem endings | `READ` |
| **4. Recorded Imitation** | `PROD_REPEAT` | Target Scene + Reference Audio + Waveform/IPA Prompt | Speak phrase aloud; compare learner recording side-by-side with native audio | `SPEAK` *(Scaffold)* |
| **5. Unprompted Spoken Recall** | `PROD_SPEAK` | Target Scene + Contextual Cue *(No reference audio prompt)* | Produce the target $L_2$ utterance aloud from memory | `SPEAK` |
| **6. Orthographic Construction** | `PROD_WRITE` | Target Scene + Spoken Audio | Assemble morpheme tiles or type full $L_2$ text with vowels/accents | `WRITE` |

---

## 4. Pinned FSRS-6 Specification & Independent Skill Histories

### 4.1 Resolution of FSRS Version & Parameter Count
In v1.0, the text referred to "FSRS-5" while citing a 21-parameter vector `w0..20` with trainable decay. Per the upstream [FSRS Algorithm Specification](https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm):
* **FSRS-5** uses **19 parameters** (`w[0]..w[18]`).
* **FSRS-6** uses **21 parameters** (`w[0]..w[20]`), where `w[20]` is the individual trainable forgetting-curve decay parameter:
  $$R(t, S) = \left(1 + \text{factor} \cdot \frac{t}{S}\right)^{-w_{20}}, \quad \text{where } \text{factor} = 0.9^{-1/w_{20}} - 1$$

**Hardened Engineering Rule:** KaizenLinguistics pins a single exact version of the Rust [`fsrs` crate (`open-spaced-repetition/fsrs-rs`)](https://github.com/open-spaced-repetition/fsrs-rs) in `Cargo.toml` and delegates all stability ($S$), difficulty ($D$), and retrievability ($R$) calculations directly to the upstream crate functions (`FSRS::next_states`) rather than re-implementing hand-rolled floating-point equations. Every logged review records the exact `fsrs_crate_version` and serialized parameter vector `w` used at the moment of scheduling.

### 4.2 Replacing Unvalidated Cross-Modal Stability Mutation with Sibling Burying
Architectural review rightly identified a major risk in v1.0's proposed Collateral Transfer Matrix $\mathbf{\Gamma}$: automatically increasing `SPEAK` or `WRITE` stability whenever a learner passes a passive `LISTEN` multiple-choice card is an unvalidated hypothesis that can mask weak active production.

In v2.0, we enforce **Strict Observation-Driven Scheduling**:
1. **Separate Modality Histories:** Each `(language_code, concept_id, modality)` tuple (`LISTEN`, `READ`, `SPEAK`, `WRITE`) is an independent FSRS card whose stability $S$ and difficulty $D$ are updated **if and only if that specific modality was directly tested**.
2. **Same-Session Sibling Burying / Spacing:** To prevent a learner from seeing the exact same sentence in `LISTEN`, `READ`, `SPEAK`, and `WRITE` back-to-back on the same day, when modality $m$ of concept $c$ is reviewed at timestamp $t_{\text{now}}$, any other due sibling card $n \neq m$ of the same concept $c$ is temporarily **buried (deferred) by a minimum spacing window** $\Delta t_{\text{bury}} = 24\text{ hours}$ without falsifying its underlying stability $S_{c,n}$!
3. **Shadow-Mode Transfer Logging (Experimental Telemetry Only):** During Milestone 4+, a background evaluator can log predicted cross-modality retrievability $\hat{R}_{c,n}$ prior to direct testing of modality $n$ to measure empirical correlation across skills, without altering due dates until statistically validated on personal review history.

---

## 5. Calibrated Mapping from Telemetry to FSRS Grades ($G \in \{1..4\}$)

To avoid noisy or punitive grading while keeping review sessions frictionless:
* **Receptive Modalities (`COMP_LISTEN`, `COMP_READ`):**
  * **$G = 1$ (`Again`):** Incorrect contrast panel selected.
  * **$G = 2$ (`Hard`):** Correct panel selected, but required $\ge 2$ audio replays or response latency $> 2.5\times$ the learner's rolling median for that clause length.
  * **$G = 3$ (`Good`):** Correct first-attempt selection within normal latency bounds ($0.75\times \text{ to } 2.5\times$ median).
  * **$G = 4$ (`Easy`):** Instantaneous first-attempt selection ($< 0.75\times$ median) with zero audio replays.
* **Productive Modalities (`PROD_SPEAK`, `PROD_WRITE`):**
  * During **Milestone 1–2 (Pre-Calibration)** and whenever the acoustic engine returns **`ABSTAIN / UNABLE_TO_ASSESS`**, the UI plays the learner's recorded audio immediately followed by the native reference audio (highlighting the target morphological contrast) and presents a transparent **4-button self-grade bar (`1: Again`, `2: Hard`, `3: Good`, `4: Easy`)**.
  * Once the acoustic engine passes empirical calibration in **Milestone 3**, automated grade proposals are pre-selected as defaults that the learner can accept with a single `Space/Enter` tap or override if a microphone glitch occurred.
