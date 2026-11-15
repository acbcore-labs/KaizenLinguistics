# 04 — Assisted Course Compilation, Multi-Evidence Screening & Editorial Gate (v2.0 — Hardened)

**Project:** KaizenLinguistics  
**Primary Languages:** Koine Greek (`grc`) & Classical / Biblical Hebrew (`hbo`)  
**Subsystem:** Offline Assisted Course Compiler (`kaizen-compiler`)  
**Licensing Guarantee:** 100% Free, Open-Weight, Public Domain / CC0 / CC-BY / CC-BY-SA Data Sources  

---

## 1. Epistemic Shift: From "Unattended Raw Joins" to "Template-First Assisted Compilation"

In v1.0, the compiler specification implied that joining six raw datasets and filtering by a single [Apple MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip) margin (`>= 0.18`) would automatically guarantee "zero-ambiguity" lessons.

Following architectural review, v2.0 corrects two core assumptions:
1. **A Neural Similarity Margin is a Ranking Heuristic, Not a Proof of Meaning:**
   [Apple MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip) (`MobileCLIP-S2` on [mlfoundations/open_clip](https://github.com/mlfoundations/open_clip)) is an image-text retrieval model, not a formal verifier of pedagogical contrast. A high similarity score can be triggered by background co-occurrences rather than the targeted grammatical distinction.
2. **Authored Contrast Templates First; Automated Corpus Enrichment Second:**
   Reconciling word senses, registers, controlled minimal contrasts, and unambiguous visual scenes cannot be left to unconstrained cross-dataset SQL joins—especially for **Koine Greek (`grc`)** and **Biblical Hebrew (`hbo`)**. Instead, v2.0 uses a **Template-First Assisted Compilation** workflow:
   * **Stage A (Curated Contrast Blueprints):** Lessons start from structured YAML/TOML contrast blueprints modeled on proven zero-L1 sequences ([Aleph with Beth](https://dantes.io/resource/aleph-with-beth-free-biblical-hebrew-video-course-6abc0ecdf769b646f61723db) for Biblical Hebrew; [Alpha with Angela](https://freegreek.online/about/) and [Everyday Koine](https://github.com/kwozymoto/koine-greek) for Koine Greek).
   * **Stage B (Corpus Enrichment & Mining):** The compiler enriches templates and mines authentic clause examples from [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew), [ETCBC/bhsa](https://github.com/ETCBC/bhsa), [Clear-Bible/macula-greek](https://github.com/Clear-Bible/macula-greek), and [morphgnt/sblgnt](https://github.com/morphgnt/sblgnt).
   * **Stage C (6-Dimensional Evidence Evaluation + Human Editorial Sign-Off):** Every compiled exercise records a structured 6-part evidence manifest, and **no exercise enters a production `.klpack` for Milestone 1–3 without `editorial_status = 'APPROVED'`**.

---

## 2. Primary Open-Source Data Sources for `grc` & `hbo`

| Data Layer | Koine Greek (`grc`) Source | Biblical Hebrew (`hbo`) Source | General / Cross-Lingual Support |
| :--- | :--- | :--- | :--- |
| **1. Base Text & Morphology** | [morphgnt/sblgnt](https://github.com/morphgnt/sblgnt) & [LogosBible/SBLGNT](https://github.com/LogosBible/SBLGNT) (CC-BY 4.0 / CC-BY-SA) | [openscriptures/morphhb](https://github.com/openscriptures/morphhb) (Public Domain WLC + CC-BY 4.0 Morph) & [ETCBC/bhsa](https://github.com/ETCBC/bhsa) | [Kaikki.org Raw Data](https://kaikki.org/dictionary/rawdata.html) via [tatuylonen/wiktextract](https://github.com/tatuylonen/wiktextract) |
| **2. Syntax Trees, Semantic Domains & Referents** | [Clear-Bible/macula-greek](https://github.com/Clear-Bible/macula-greek) & [NuBerea/macula-sblgnt-syntax](https://huggingface.co/datasets/NuBerea/macula-sblgnt-syntax) (Louw-Nida Domains + Referents) | [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew) (UBS SDBH Semantic Domains + Participant Referents) | [Universal Dependencies](https://universaldependencies.org/): [UD_Ancient_Greek-PROIEL](https://github.com/UniversalDependencies/UD_Ancient_Greek-PROIEL) & [UD_Ancient_Hebrew-PTNK](https://github.com/UniversalDependencies/UD_Ancient_Hebrew-PTNK) |
| **3. Lexica & Principal Parts** | [translatable-exegetical-tools/Abbott-Smith](https://github.com/translatable-exegetical-tools/Abbott-Smith) & [morphgnt/morphological-lexicon](https://github.com/morphgnt/morphological-lexicon) | [openscriptures/HebrewLexicon](https://github.com/openscriptures/HebrewLexicon) (BDB + Strong's XML) & [BibleAquifer/BDBHebrewLexicon](https://github.com/BibleAquifer/BDBHebrewLexicon/blob/main/README.md) | [Wikidata Lexicographical Data](https://www.wikidata.org/wiki/Wikidata:Lexicographical_data) |
| **4. Immersion Scope, Visuals & Audio** | [Alpha with Angela (`freegreek.online`)](https://freegreek.online/about/), [kwozymoto/koine-greek](https://github.com/kwozymoto/koine-greek), [greek-learner-texts](https://github.com/greek-learner-texts/vocabulary-data) | [Aleph with Beth (`freehebrew.online`)](https://freehebrew.online/the-divine-name/) ([Dantes Review](https://dantes.io/resource/aleph-with-beth-free-biblical-hebrew-video-course-6abc0ecdf769b646f61723db)) | [Open Images V7](https://docs.ultralytics.com/datasets/detect/open-images-v7), Wikimedia Commons & [Kokoro-82M MLX](https://github.com/gabrimatic/kokoro-mlx) |

---

## 3. The Six-Dimensional Evidence Record (Replacing Binary "Verified" Claims)

Instead of stamping an exercise with a single boolean `verified = true` based on a CLIP threshold, `kaizen-compiler` evaluates and stores **six distinct evidence fields** in the `exercise_evidence` table for every candidate contrast exercise:

| Evidence Dimension | What It Measures | How It Is Computed | Role in Gating |
| :--- | :--- | :--- | :--- |
| **1. `structural_validity`** | Schema completeness, valid foreign keys, non-empty audio/visual blobs, valid Unicode normalization (`NFC` with canonical Hebrew *niqqud* ordering). | Deterministic Rust/Python schema validator. | **Hard Gate:** Must be `PASS` (`100%`). |
| **2. `linguistic_contrast_purity`** | Confirms that contrasting panels isolate the targeted feature (e.g., `Case=Nom` vs. `Case=Acc`, or `Binyan=Qal` vs. `Binyan=Hiphil`) without unintended lexical or morphological confounding. | Morphological feature diff over [MACULA](https://github.com/Clear-Bible/macula-hebrew) / [MorphGNT](https://github.com/morphgnt/sblgnt) tags. | **Hard Gate:** Flags `CONFOUNDED_LEXEME` or `MULTI_FEATURE_DRIFT` for human fix. |
| **3. `visual_discrimination_score`** | Heuristic estimate of how clearly target image $I_i$ matches scene description $T_i$ relative to distractor images $I_{j \neq i}$. | [Apple MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip) diagonal margin $\Delta_{\text{clip}} = \mathbf{M}_{i,i} - \max_{j \neq i} \mathbf{M}_{i,j}$ (stored as continuous `REAL`, e.g., `0.24`). | **Soft Screening Signal:** Ranks candidate images and flags `< 0.15` for editorial review; never treated as proof of truth. |
| **4. `audio_alignment_confidence`** | Acoustic quality and phoneme-boundary alignment probability between audio waveform and expected `grc`/`hbo` IPA string. | Mean log-posterior and frame coverage from [Montreal Forced Aligner](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner) / `wav2vec2-xlsr-53`. | **Quality Gate:** Flags low-confidence TTS or noisy recordings (`< 0.80`) for re-synthesis or human recording. |
| **5. `license_provenance_complete`** | Verifies every attached text, audio clip, and image carries a valid SPDX license identifier (`CC0-1.0`, `CC-BY-4.0`, `CC-BY-SA-4.0`, `Public-Domain`), author attribution, and source URI. | Deterministic provenance check against `media_assets`. | **Hard Gate:** Must be `PASS` (`100%`). |
| **6. `editorial_review_status`** | Human/author inspection confirming that the visual contrast, audio cadence, and morphological highlight genuinely teach the target concept without ambiguity. | Explicit CLI/UI review workflow (`DRAFT` $\rightarrow$ `APPROVED` / `REJECTED` with `rejection_reason`). | **Final Release Gate (Milestones 1–3):** Only `APPROVED` exercises are packaged into study sessions. |

---

## 4. Handling Abstract & Non-Picturable Language (Intermediate Bridge)

A classic failure mode of pure photo-matching systems is that intermediate language involves concepts that cannot be pictured unambiguously in a static photograph (e.g., *"in order that" (`ἵνα`)*, *"covenant loyalty" (`חֶסֶד`)*, *"therefore" (`οὖν`)*, *"for" (`γάρ` / `כִּי`)*, *"righteousness" (`δικαιοσύνη` / `צְדָקָה`)*).

KaizenLinguistics solves this without reverting to English grammar-translation lectures by transitioning across **four representational modalities** as the learner advances from Tier 1 to Tier 4:

1. **Concrete Perceptual Scenes (Tier 1–2):** Static SVG/illustrated scenes and curated photos for physical entities, spatial prepositions, concrete actions, number, gender, and Agent/Patient case roles.
2. **Sequential Comic / Aspect Strips (Tier 2–3):** 3-panel chronological sequences for verbal aspect (Greek Imperfective vs. Aorist vs. Perfect; Hebrew *Qatal* vs. *Yiqtol* vs. *Wayyiqtol*) and modal causation (*Qal* vs. *Hiphil*).
3. **Contextual $L_2$ Micro-Dialogues & Situational Frames (Tier 3):** Short 2-to-3 sentence exchanges using $100\%$ already-mastered vocabulary where discourse particles (`γάρ`, `οὖν`, `ἀλλά`, `כִּי`, `הִנֵּה`, `לָכֵן`) connect cause and effect, and the learner selects the logical continuation or speaker intent.
4. **Scaffolded Authentic Pericope Reader with Semantic Domain Links (Tier 4 — Intermediate):**
   Draws directly from [Clear-Bible/macula-greek](https://github.com/Clear-Bible/macula-greek) (Louw-Nida Semantic Domains) and [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew) (UBS Semantic Dictionary of Biblical Hebrew) to present authentic biblical clauses where unknown abstract words are explained first via **known $L_2$ synonyms/antonyms from the same semantic domain** and participant-referent arrows, with optional on-demand access to [Abbott-Smith](https://github.com/translatable-exegetical-tools/Abbott-Smith) or [OpenScriptures BDB](https://github.com/openscriptures/HebrewLexicon) lexical entries.
