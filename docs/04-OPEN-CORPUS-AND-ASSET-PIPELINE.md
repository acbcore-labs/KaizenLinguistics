# 04 — Open-Source Content, Linguistic Corpus & Visual Grounding Pipeline

**Project:** KaizenLinguistics  
**Subsystem:** Offline Proposer-Verifier Course Compiler (`kaizen-compiler`)  
**Licensing Guarantee:** 100% Free, Open-Weight, Public Domain / CC0 / CC-BY-SA Data Sources  

---

## 1. The Zero-Cost Content Challenge

Building a commercial-grade [Rosetta Stone Dynamic Immersion](https://www.rosettastone.com/our-method) curriculum traditionally required millions of dollars in studio photography, bilingual lexicographers, and voice actors ([Rosetta Stone Profile](https://economy.ac/wiki/company/rosetta-stone)).

**KaizenLinguistics** replaces manual content production with an automated, deterministic **Open-Source Linguistic & Visual ETL Compiler** (`kaizen-compiler`) that synthesizes complete, multi-level courses for any target language ($L_2$) by joining six open-data pillars:

| Data Layer | Primary Open-Source Dataset / Tool | License | Role in Course Synthesis |
| :--- | :--- | :--- | :--- |
| **1. Cross-Lingual Concept Graph** | [Wikidata Lexicographical Data](https://www.wikidata.org/wiki/Wikidata:Lexicographical_data) (`L-ID` Lexemes + `Q-ID` Concepts) | CC0 (Public Domain) | Language-independent semantic anchor linking words across all languages to concrete visualizable entities (`P18` image, `P5137` item for sense) |
| **2. Lexicon, IPA & Inflection Tables** | [Kaikki.org Raw Data](https://kaikki.org/dictionary/rawdata.html) via [tatuylonen/wiktextract](https://github.com/tatuylonen/wiktextract) & [Kaikki-Wiktionary-Ultimate-SQLite](https://huggingface.co/datasets/wave101828228/Kaikki-Wiktionary-Ultimate-SQLite) | CC-BY-SA / GFDL | Complete morphological inflection paradigms, gender/noun classes, IPA pronunciations, audio links, and semantic glosses |
| **3. Morphosyntactic Grammar Spine** | [Universal Dependencies (UD)](https://universaldependencies.org/) (200+ Treebanks across 150+ Languages) | CC-BY-SA / GPL / MIT | Gold-standard CoNLL-U syntactic dependency trees (`UPOS`, `FEATS`, `DEPREL`) used to order grammar concepts and extract minimal-pair syntactic templates |
| **4. Graded Sentence Corpus** | [Tatoeba Corpus](https://en.wikipedia.org/wiki/Tatoeba) & [Helsinki-NLP/tatoeba-challenge](https://github.com/Helsinki-nlp/tatoeba-challenge) | CC-BY 2.0 FR / CC0 | Millions of natural, learner-oriented sentences indexed by length, lemma frequency, and syntactic structure |
| **5. Grounded Visual Scenes** | [Open Images Dataset V7](https://docs.ultralytics.com/datasets/detect/open-images-v7) ([openimages/dataset](https://github.com/openimages/dataset/blob/master/READMEV3.md)) + Wikimedia Commons | CC-BY 4.0 / CC0 | 9M+ images with object bounding boxes, segmentation masks, visual relationships (`boy riding horse`), and localized narratives |
| **6. Native & Neural Speech Audio** | [Mozilla Common Voice](https://commonvoice.mozilla.org/) + [Kokoro-82M MLX](https://github.com/gabrimatic/kokoro-mlx) ([Neywa-Labs/mlx-audio](https://github.com/Neywa-Labs/mlx-audio)) | CC0 + Apache-2.0 | Human native recordings supplemented by 24 kHz multi-speaker neural TTS force-aligned via [Montreal Forced Aligner](https://github.com/MontrealCorpusTools/Montreal-Forced-Aligner) |

---

## 2. Five-Stage Proposer-Verifier Compilation Pipeline

```
+-----------------------------------------------------------------------------+
| STAGE 1: CONCEPT & LEXEME INGESTION                                         |
| Wikidata Lexemes (L-IDs / Q-IDs) + Kaikki.org (wiktextract JSONL / SQLite)  |
| -> Builds Canonical Target-Language Lexeme & Inflection Paradigm Graph      |
+--------------------------------------+--------------------------------------+
                                       ▼
+-----------------------------------------------------------------------------+
| STAGE 2: MORPHOSYNTACTIC CURRICULUM SEQUENCING                              |
| Universal Dependencies (UD) Treebanks + Tatoeba Sentence Corpus             |
| -> Ranks sentences by i+1 Concept Coverage & UD Feature Complexity          |
+--------------------------------------+--------------------------------------+
                                       ▼
+-----------------------------------------------------------------------------+
| STAGE 3: VISUAL SCENE MATCHING & DISTRACTOR MINING (PROPOSER)               |
| Open Images V7 Visual Relationships + Wikimedia Commons                     |
| -> Proposes 4-Image Minimal-Pair Quads (1 Target + 3 Hard Distractors)      |
+--------------------------------------+--------------------------------------+
                                       ▼
+-----------------------------------------------------------------------------+
| STAGE 4: DETERMINISTIC MULTIMODAL VERIFIER GATE                             |
| 1. Apple MobileCLIP Cosine Margin Gate (Δ_clip >= 0.18, zero ambiguity)     |
| 2. UD One-Hot Feature Delta Check (exact morphological/lexical contrast)    |
| 3. Montreal Forced Aligner / Wav2Vec2 Audio-Phoneme Alignment Check         |
+--------------------------------------+--------------------------------------+
                                       ▼
+-----------------------------------------------------------------------------+
| STAGE 5: IMMUTABLE MERKLE COURSE PACKAGER (.klpack)                         |
| Compiles verified SQLite DB + 48kbps Opus Audio + WebP 512x512 Tiles        |
+-----------------------------------------------------------------------------+
```

---

## 3. Detailed Stage Specifications

### 3.1 Stage 1: Cross-Lingual Concept Graph & Morphological Paradigms
Rather than anchoring a course in English translations, every teachable unit in KaizenLinguistics is anchored to a language-agnostic **Concept Node** (`concept_qid`, e.g., `Q726` = *horse*, `Q146` = *cat*, `Q1079023` = *running*):
1. **Wikidata Lexeme Resolution:** Querying [Wikidata Lexicographical Data](https://www.wikidata.org/wiki/Wikidata:Lexicographical_data/Documentation), each Lexeme (`L-ID`) in target language $L_2$ links its specific Sense (`S-ID`) to a Wikidata Item (`Q-ID`) via property `P5137` (*item for this sense*) or `P5972` (*translation*), and carries explicit grammatical Forms (`F-ID`) annotated with grammatical features (e.g., singular, plural, nominative, accusative, past participle).
2. **Kaikki / Wiktextract Enrichment:** The compiler joins each lemma with [Kaikki.org](https://kaikki.org/dictionary/rawdata.html) ([tatuylonen/wiktextract](https://github.com/tatuylonen/wiktextract)) to extract complete inflectional tables, IPA transcriptions, hyphenation/syllabification, and Wikimedia Commons audio files.

### 3.2 Stage 2: Inductive $i + 1$ Curriculum Ordering via Universal Dependencies
To replicate Rosetta Stone's carefully graded ramp (where every new screen introduces at most **one** unknown lexical or morphological element), the compiler parses candidate sentences from [Tatoeba](https://en.wikipedia.org/wiki/Tatoeba) into [Universal Dependencies](https://universaldependencies.org/) `CoNLL-U` structures:
* Each sentence $S$ is represented as a tuple of:
  * **Lemma Set:** $\mathcal{L}(S) = \{\ell_1, \dots, \ell_k\}$
  * **Morphological Feature Set:** $\mathcal{F}(S) = \bigcup_{i=1}^k \text{FEATS}(w_i)$ (e.g., `Number=Plur`, `Gender=Fem`, `Tense=Pres`, `Aspect=Prog`, `Case=Acc`)
  * **Syntactic Dependency Relations:** $\mathcal{D}(S) = \{\text{nsubj}, \text{obj}, \text{amod}, \text{case}, \dots\}$
* **Greedy Topological $i+1$ Ordering:** Given the set of already-introduced concepts $\mathcal{K}_t$ at curriculum step $t$, the next lesson unit selects a cluster of minimal-pair sentences $S_{\text{next}}$ that minimizes unknown elements:
  $$\left| \left(\mathcal{L}(S_{\text{next}}) \cup \mathcal{F}(S_{\text{next}})\right) \setminus \mathcal{K}_t \right| = 1$$

### 3.3 Stage 3 & 4: Visual Grounding & The MobileCLIP Verifier Gate
A major failure mode in automated visual flashcard generation is **visual ambiguity**—if the audio says *"The woman is drinking water"* and two different images in the $2 \times 2$ quad show a woman holding a cup, the immersion contract breaks.

KaizenLinguistics prevents visual ambiguity using a two-step **Visual Relationship + [Apple MobileCLIP](https://github.com/apple-aiml-research/ml-mobileclip) Verification Gate**:

1. **Structured Scene Retrieval from Open Images V7:**
   [Open Images V7](https://docs.ultralytics.com/datasets/detect/open-images-v7) contains explicit **Visual Relationship Triplets** `(Subject, Predicate, Object)`—such as `(Man, riding, Bicycle)`, `(Girl, playing, Guitar)`, `(Dog, on, Chair)`, `(Car, is, Red)`—alongside localized narratives ([Open Images Deep Dive](https://roboflow.com/blog/open-images-dataset)).
2. **Factorial Distractor Selection:**
   To build a $2 \times 2$ quad for target triplet $(Subj^*, Pred^*, Obj^*)$, the Proposer selects three hard distractors that share structural overlap:
   * Distractor $I_2$: $(Subj^*, Pred^*, Obj_{\text{alt}})$ — Same subject and action, different object.
   * Distractor $I_3$: $(Subj_{\text{alt}}, Pred^*, Obj^*)$ — Different subject, same action and object.
   * Distractor $I_4$: $(Subj_{\text{alt}}, Pred_{\text{alt}}, Obj_{\text{alt}})$ — Foil contrast.
3. **Deterministic MobileCLIP Disambiguation Margin:**
   Using [apple-aiml-research/ml-mobileclip](https://github.com/apple-aiml-research/ml-mobileclip) (`MobileCLIP-S2`, built on [mlfoundations/open_clip](https://github.com/mlfoundations/open_clip)), the Verifier computes normalized embeddings $\hat{\mathbf{v}}_k = E_{\text{img}}(I_k)$ for all four candidate images ($k \in \{1..4\}$) and text embeddings $\hat{\mathbf{u}}_k = E_{\text{txt}}(T_k)$ for the canonical scene descriptions.
   A candidate quad $\mathcal{Q}$ is **accepted into the curriculum if and only if** the confusion matrix $\mathbf{M}_{i,j} = \hat{\mathbf{u}}_i^\top \hat{\mathbf{v}}_j$ is strictly diagonally dominant with margin $\tau_{\text{clip}} = 0.18$:

   $$\forall i \in \{1, 2, 3, 4\}: \quad \mathbf{M}_{i,i} - \max_{j \neq i} \mathbf{M}_{i,j} \ge \tau_{\text{clip}}$$

   If any distractor image $I_j$ has high semantic similarity to prompt $T_i$ (for example, a background person in $I_j$ is also performing action $T_i$), the Verifier automatically rejects the image and swaps in a clean candidate.

---

## 4. Curated Core Curriculum Taxonomy (Levels 1 to 5 — 20 Units)

Following Rosetta Stone's proven functional progression while grounding every unit in [Universal Dependencies](https://universaldependencies.org/) morphosyntax, every compiled `.klpack` course is organized into **5 Levels (20 Units, 80 Core Lessons)**:

| Level | Units | Core Semantic Domains | Targeted Morphosyntactic & Phonological Structures |
| :--- | :---: | :--- | :--- |
| **Level 1: Foundations of Perception** | Units 1–4 | People, everyday actions, colors, numbers (1–20), clothing, food, animals, family, household objects | Present continuous/indicative, grammatical gender & noun classes, singular/plural inflection, adjective agreement, basic negation, yes/no & `wh-` questions |
| **Level 2: Spatial, Temporal & Social Navigation** | Units 5–8 | Locations in town, directions, transit, clock time, calendar/seasons, weather, shopping, dining out | Locative/directional adpositions & cases, past perfective/imperfective tense, future tense, pronominal clitics (direct/indirect object), polite vs. familiar register |
| **Level 3: Narrative, Comparison & Agency** | Units 9–12 | Work & trades, health & body, travel & lodging, measurements, emotions, sequential events | Comparative & superlative degree, reflexive/reciprocal verbs, imperative mood, temporal subordinate clauses (*before, after, while*), passive/impersonal voice |
| **Level 4: Complex Modality & Abstract Causation** | Units 13–16 | Commerce, repairs & machinery, environment, cultural traditions, problem resolution | Conditional & counterfactual clauses, subjunctive/optative mood, relative clauses, modal auxiliaries (*must, should, might*), causative constructions |
| **Level 5: Fluency, Discourse & Technical Precision** | Units 17–20 | Extended narratives, debate/opinion, idiomatic register, technical/industrial descriptions | Reported/indirect speech, aspectual nuances, complex multi-clause subordination, native prosodic speed drills |
