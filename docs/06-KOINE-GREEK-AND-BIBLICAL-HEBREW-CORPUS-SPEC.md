# 06 — Koine Greek (`grc`) & Biblical Hebrew (`hbo`) Linguistic & Open-Resource Specification

**Project:** KaizenLinguistics  
**Primary Target Languages:** Hellenistic / Koine Greek (`ISO 639-3: grc`) & Classical / Biblical Hebrew (`ISO 639-3: hbo`)  
**Proficiency Target:** Absolute Beginner (Script & Phonology) $\rightarrow$ Intermediate Reading, Listening & Spoken Internalization  

---

## 1. Biblical Hebrew (`hbo`) vs. Modern Israeli Hebrew (`heb`): Why They Cannot Be Conflated

A foundational requirement for **KaizenLinguistics** is targeting **Ancient / Biblical Hebrew (`hbo`)** rather than Modern Israeli Hebrew (`heb`). While Modern Hebrew revived the consonantal script and core triliteral root morphology of Classical Hebrew, the two varieties are classified as distinct languages in [Universal Dependencies (`UD_Ancient_Hebrew-PTNK` vs. `UD_Hebrew-HTB`)](https://aclanthology.org/2022.emnlp-main.292.pdf) and differ across five structural dimensions:

| Linguistic Dimension | Ancient / Biblical Hebrew (`hbo`) | Modern Israeli Hebrew (`heb`) | Impact on KaizenLinguistics Design |
| :--- | :--- | :--- | :--- |
| **1. Canonical Word Order & Clause Syntax** | Predominantly **VSO (Verb–Subject–Object)** in narrative clauses, driven by discourse-level clause chaining ([ETCBC/bhsa](https://github.com/ETCBC/bhsa)). | Predominantly **SVO (Subject–Verb–Object)** influenced by Standard Average European syntax. | Contrast exercises and syntax trees must use [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew) and [UD_Ancient_Hebrew-PTNK](https://github.com/UniversalDependencies/UD_Ancient_Hebrew-PTNK), never Modern Hebrew Tatoeba sentences. |
| **2. Verbal System (Aspect/Sequentiality vs. Absolute Tense)** | **Aspectual & Discourse-Sequential:** Uses Suffix conjugation (*Qatal* / Perfective), Prefix conjugation (*Yiqtol* / Imperfective), and **Waw-Consecutive (*Wayyiqtol* `וַיֹּאמֶר` / *Weqatal*)** narrative chains, plus Infinitive Absolute (`שָׁמוֹר תִּשְׁמְרוּן`) and Cohortative/Jussive moods. | **Absolute Tri-Tense System:** Reorganized into Past (*Qatal*), Present (*Qotel* participle), and Future (*Yiqtol*); *Wayyiqtol* is extinct in everyday speech. | The Contrast Graph must explicitly teach *Qatal* vs. *Yiqtol* vs. *Wayyiqtol* discourse sequences and the 7 *Binyanim* (`Qal`, `Niphal`, `Piel`, `Pual`, `Hiphil`, `Hophal`, `Hithpael`). |
| **3. Orthography, Vocalization (`Niqqud`) & Cantillation (`Te'amim`)** | Preserved in the **Masoretic Text** ([openscriptures/morphhb](https://github.com/openscriptures/morphhb)) with full Tiberian **niqqud** (distinguishing *Qamatz Gadol/Qatan*, *Tzere*, *Segol*, *Pathach*, Vocal/Silent *Shewa*, *Dagesh Lene/Forte*) and **cantillation marks (*te'amim*)** encoding prosodic stress and syntactic pauses. | Written **unpointed (*ktav male*)** in daily life; several Tiberian vowel distinctions are merged into a 5-vowel `/a e i o u/` inventory. | The UI must render full Tiberian Unicode (`U+0591`–`U+05C7` via *SBL Hebrew* / *Ezra SIL* open fonts) with toggleable layers: (1) Full Masoretic (`Consonants + Niqqud + Te'amim`), (2) Pedagogical (`Consonants + Niqqud`), and (3) Advanced Unpointed (`Consonants Only`). |
| **4. Consonantal Phonology (`Begadkefat`, Pharyngeals & Emphatics)** | Distinguishes pharyngeal fricatives (`ח` `/ħ/`, `ע` `/ʕ/`), glottal stop (`א` `/ʔ/`), uvular/emphatics (`ט` `/tˤ/`, `צ` `/sˤ/`, `ק` `/q/`), and all 6 *Begadkefat* spirantized pairs (`ב ג ד כ פ ת`). | General Israeli pronunciation merges `א/ע`, often merges `ח/כּ` to `/χ/`, `ט/ת` to `/t/`, and `ק/כ` to `/k/`. | Our G2P and pronunciation profiles must support **Reconstructed Classical / Tiberian Biblical Pronunciation** (and optional Academic/Sephardic Teaching Pronunciation) rather than penalizing historical contrasts using a Modern Hebrew ASR model. |
| **5. Lexical Semantics & False Friends** | Classical semantic ranges grounded in Ancient Near Eastern life (`דָּבָר` = *word/speech/matter*; `נֶפֶשׁ` = *throat/breath/living being/self*; `חֶסֶד` = *covenant loyalty/steadfast love*). | Shifted modern secular meanings (`דָּבָר` = *thing/object*; `חֶסֶד` = *charity/favor*). | Glosses and semantic domain links must be sourced strictly from [openscriptures/HebrewLexicon](https://github.com/openscriptures/HebrewLexicon) (BDB/Strong's) and UBS Semantic Domains in [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew). |

---

## 2. Koine Greek (`grc`): Linguistic Architecture for Intermediate Mastery

Hellenistic / Koine Greek (`grc`, ~300 BCE – 300 CE)—the language of the Septuagint (LXX), the Greek New Testament, the Apostolic Fathers, Epictetus, and Plutarch—requires three specialized architectural provisions in KaizenLinguistics:

1. **Configurable Pronunciation Profiles (Phonological Layer):**
   Because scholarship and immersion programs use different pronunciation reconstructions, the course compiler and G2P engine explicitly support three selectable IPA profiles:
   * **Profile A — Restored 1st-Century Imperial Koine (Recommended for Audio Immersion):** Follows Randall Buth / Benjamin Kantor reconstruction used by [Alpha with Angela (`freegreek.online`)](https://freegreek.online/about/), reflecting authentic 1st-century CE phonology while preserving spoken fluency.
   * **Profile B — Academic Erasmian:** Preserves one-to-one grapheme-to-phoneme distinctions acrossiotacized vowels (`η`, `ι`, `υ`, `ει`, `οι`, `υι`), helpful during orthographic dictation drills (`PROD_WRITE`).
   * **Profile C — Modern / liturgical Greek:** Full iotacism (`/i/`) and fricative voiced stops (`/v ð ɣ/`).
2. **Aspect-First Verbal Pedagogy (Not English Tense Mapping):**
   Koine Greek verbs encode **Grammatical Aspect** primary to time in all moods outside the Indicative:
   * **Imperfective (`Present` / `Imperfect` stem):** Unfolding, ongoing, or repeated process (*"was loosing / kept loosing"*).
   * **Perfective (`Aorist` stem):** Complete, summary view of the action as a whole (*"loosed"*).
   * **Stative / Combinative (`Perfect` / `Pluperfect` stem):** Completed action with resulting state (*"stands loosed"*).
   Visual contrasts in KaizenLinguistics use **3-frame aspectual strips** (unfolding process vs. whole event snapshot vs. resultant state) rather than misleading English past/present/future labels.
3. **5-Case Argument Structure & Information Packaging:**
   Since Koine Greek word order is pragmatically flexible (driven by discourse prominence and topic/focus rather than fixed SVO position), minimal-pair visual quads directly contrast **Nominative (Agent/Subject)** vs. **Accusative (Patient/Direct Object)** vs. **Dative (Recipient/Instrument/Locative)** vs. **Genitive (Source/Possessor/Partitive)** while permuting word order (`ὁ ἄγγελος βλέπει τὸν ἀδελφόν` vs. `τὸν ἄγγελον βλέπει ὁ ἀδελφός`).

---

## 3. Master Catalog of Downloadable Open-Source Corpora & Resources

Unlike modern conversational languages that rely on noisy web scrapes, **Koine Greek and Biblical Hebrew possess the most exhaustively annotated, word-by-word morphologically parsed and syntactically treebanked open-source corpora of any languages on Earth.** Every single word of the Hebrew Bible and Greek New Testament is already lemmatized, morphologically tagged, syntactically treebanked, and mapped to semantic domains under permissive open licenses.

### 3.1 Biblical Hebrew (`hbo`) Downloadable Open Repositories

| Category | Repository / Dataset | License | Format | Exact Role in KaizenLinguistics |
| :--- | :--- | :--- | :--- | :--- |
| **Morphology, Clause Hierarchy & Discourse DB** | [ETCBC/bhsa (Biblia Hebraica Stuttgartensia Amstelodamensis)](https://github.com/ETCBC/bhsa) & [ETCBC/shebanq](https://github.com/ETCBC/shebanq) | CC-BY-NC 4.0 / Open Research | Python `Text-Fabric` (`.tf`) | Gold-standard word, phrase, clause, and sentence annotations (`sp` part of speech, `vt` verbal tense/aspect, `vs` verbal stem *binyan*, `function` syntactic role) across the entire Hebrew Bible. |
| **Syntax Trees, Semantic Domains & Referents** | [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew) | CC-BY 4.0 | XML (`Nodes` / `LowFat`) & TSV | Complete Westminster Leningrad Codex (Public Domain) + morphology + syntactic trees + **UBS Semantic Dictionary of Biblical Hebrew (SDBH)** domains + **Participant Referent tracking** (resolving every pronoun suffix to its exact entity!). |
| **Base Text, Lemma, Morphology & Cantillation** | [openscriptures/morphhb (Open Scriptures Hebrew Bible)](https://github.com/openscriptures/morphhb) | Public Domain (WLC) / CC-BY 4.0 (Morph) | OSIS XML / JSON | Immutable word IDs, Strong's + morphological parsing (`HC/R/Ncmsc`), and cantillation hierarchy (`MAPM`) for prosodic phrasing. |
| **Universal Dependencies Treebank** | [UniversalDependencies/UD_Ancient_Hebrew-PTNK](https://github.com/UniversalDependencies/UD_Ancient_Hebrew-PTNK) | CC-BY-SA 4.0 | `CoNLL-U` | Standard UD dependency parses derived from ETCBC BHSA for uniform cross-lingual querying in `kaizen-compiler`. |
| **Comprehensive XML Lexicon (BDB + Strong's)** | [openscriptures/HebrewLexicon](https://github.com/openscriptures/HebrewLexicon) & [BibleAquifer/BDBHebrewLexicon](https://github.com/BibleAquifer/BDBHebrewLexicon/blob/main/README.md) | Public Domain / CC-BY 4.0 | Modular XML (`BrownDriverBriggs.xml`, `HebrewStrong.xml`, `LexicalIndex.xml`) | Maps every triliteral root and lexeme to BDB semantic hierarchies, Strong's numbers, and TWOT cross-references. |
| **Dead Sea Scrolls Extension (Intermediate+)** | [ETCBC/dss](https://github.com/etcbc/dss) | Open Research | `Text-Fabric` | Morphologically tagged Qumran texts for advanced historical reading progression. |
| **Open-Source Visual-Audio Immersion Curriculum** | [Aleph with Beth (`freehebrew.online`)](https://freehebrew.online/the-divine-name/) ([Overview](https://dantes.io/resource/aleph-with-beth-free-biblical-hebrew-video-course-6abc0ecdf769b646f61723db)) | Creative Commons (CC-BY-SA) | Downloadable Video, Lesson Scripts, Visual Props & Quizzes | **100+ sequential zero-L1 immersion lessons** created by Beth & Andrew Case. Their downloadable lesson scripts provide a ready-made, human-tested $i+1$ scope and sequence from Lesson 1 (`אִישׁ / אִשָּׁה`, singular/plural) to intermediate narrative prose! |

---

### 3.2 Koine Greek (`grc`) Downloadable Open Repositories

| Category | Repository / Dataset | License | Format | Exact Role in KaizenLinguistics |
| :--- | :--- | :--- | :--- | :--- |
| **Morphologically Parsed Greek New Testament** | [morphgnt/sblgnt](https://github.com/morphgnt/sblgnt) & [LogosBible/SBLGNT](https://github.com/LogosBible/SBLGNT) | CC-BY 4.0 (Text) / CC-BY-SA (Morph) | Columnar TSV / Plain Text | Complete word-by-word part-of-speech, person, tense, voice, mood, case, number, gender, word, normalized word, and lemma for every token in the SBLGNT. |
| **Syntax Trees, Semantic Domains & Referents** | [Clear-Bible/macula-greek](https://github.com/Clear-Bible/macula-greek) & [NuBerea/macula-sblgnt-syntax](https://huggingface.co/datasets/NuBerea/macula-sblgnt-syntax) | CC-BY 4.0 | XML (`LowFat` / `Nodes`), TSV & Parquet | Full syntactic constituency/dependency trees, **Louw & Nida Semantic Domains**, semantic roles (`Agent`, `Verb`, `Patient`), and participant referent tracking across SBLGNT and Nestle 1904. |
| **Derivational & Principal Parts Lexicon** | [morphgnt/morphological-lexicon](https://github.com/morphgnt/morphological-lexicon) & [jtauber/greek-lemma-mappings](https://github.com/jtauber/greek-lemma-mappings) | CC-BY-SA | YAML / TSV | Complete inflectional paradigms, derivational families, and principal parts for GNT verbs and nominals. |
| **Universal Dependencies Treebank** | [UniversalDependencies/UD_Ancient_Greek-PROIEL](https://github.com/UniversalDependencies/UD_Ancient_Greek-PROIEL) | CC-BY-NC-SA 4.0 | `CoNLL-U` | Gold-standard dependency annotations and information-structure tags across the Greek New Testament ([UD grc_proiel](https://universaldependencies.org/treebanks/grc_proiel/index.html)). |
| **Public Domain Lexica (Rich & Concise)** | [translatable-exegetical-tools/Abbott-Smith](https://github.com/translatable-exegetical-tools/Abbott-Smith) & Dodson Lexicon in [kwozymoto/koine-greek](https://github.com/kwozymoto/koine-greek) | Public Domain / CC0 | TEI XML / CSV / JSON | Abbott-Smith's *Manual Greek Lexicon of the New Testament* (with LXX background notes) and Dodson's concise glosses. |
| **Graded Learner Texts & Septuagint / Patristics** | [greek-learner-texts/vocabulary-data](https://github.com/greek-learner-texts/vocabulary-data), [biblicalhumanities/greek-new-testament](https://github.com/biblicalhumanities/greek-new-testament) & [PerseusDL/canonical-greekLit](https://github.com/PerseusDL/canonical-greekLit) | CC-BY-SA 4.0 | XML / Text / TSV | Graded reader progressions, Nestle 1904 syntax trees, Septuagint (LXX), Apostolic Fathers, and Hellenistic prose. |
| **Open-Source Visual-Audio Immersion Curriculum** | [Alpha with Angela (`freegreek.online`)](https://freegreek.online/about/) & [kwozymoto/koine-greek (Everyday Koine)](https://github.com/kwozymoto/koine-greek) | Creative Commons (CC) / Open Source | Downloadable Video, Scripts, Audio & Paradigm Drills | Sister project to *Aleph with Beth* providing zero-L1 communicative Koine Greek immersion in Restored Koine pronunciation, plus Everyday Koine's 27 offline chapters with audio. |

---

## 4. Curriculum Trajectory: From Zero to Intermediate Reading & Internalization

Why do traditional grammar-translation seminary students struggle to read a page of Genesis or the Gospel of Mark without thumbing a lexicon every third word? Because they memorize abstract English paradigm tables (`λύω, λύεις, λύει...`) rather than internalizing high-frequency morphosyntactic contrasts through direct visual and auditory comprehension.

By combining the **curated $i+1$ progression of [Aleph with Beth](https://dantes.io/resource/aleph-with-beth-free-biblical-hebrew-video-course-6abc0ecdf769b646f61723db) and [Alpha with Angela](https://freegreek.online/about/)** with the **exhaustive syntactic and semantic-domain indices of [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew), [ETCBC/bhsa](https://github.com/ETCBC/bhsa), [Clear-Bible/macula-greek](https://github.com/Clear-Bible/macula-greek), and [morphgnt/sblgnt](https://github.com/morphgnt/sblgnt)**, KaizenLinguistics structures the path to intermediate fluency into **Five Concrete Tiers**:

### Tier 1: Script, Phonology & Concrete Visual Grounding (0 $\rightarrow$ 150 Core Words)
* **Biblical Hebrew (`hbo`):**
  * Consonants + Tiberian *Niqqud* vowel discrimination via minimal acoustic pairs (`בָּא` vs. `בֹּא`, *dagesh* gemination, guttural syllable rules).
  * Concrete visual nouns (`אִישׁ`, `אִשָּׁה`, `נַעַר`, `סוּס`, `בַּיִת`, `מַיִם`, `לֶחֶם`, `עֵץ`), cardinal numbers 1–10, and spatial prepositions (`בְּ`, `לְ`, `מִן`, `עַל`, `תַּחַת`, `לִפְנֵי`).
  * Gender & number agreement across singular, dual (`יָדַיִם`, `עֵינַיִם`), and plural (`-ִים` / `-וֹת`) with active participles (`עֹמֵד / עֹמֶדֶת`, `יֹשֵׁב / יֹשֶׁבֶת`, `הֹלֵךְ / הֹלֶכֶת`).
* **Koine Greek (`grc`):**
  * Alphabet, diphthongs, rough/smooth breathing (`ἁ` vs. `ἀ`), and pitch/stress accent placement.
  * Concrete visual nouns across 1st, 2nd, and 3rd declensions with definite articles (`ὁ, ἡ, τό`), spatial prepositions governing specific cases (`ἐν + Dat`, `εἰς + Acc`, `ἐκ + Gen`, `πρός + Acc`), and 3rd-person Present Indicative Active & Middle/Passive verbs (`βλέπει`, `λέγει`, `ἔρχεται`, `κάθηται`).

### Tier 2: Argument Structure, Case / Construct Chains & Pronominal Attachment (150 $\rightarrow$ 400 Words)
* **Biblical Hebrew (`hbo`):**
  * Definite direct object marker (`אֵת / אֶת־`) contrasting Agent vs. Patient in flexible VSO/SVO clauses.
  * **Construct State (*Smikhut*):** Visual contrast between absolute phrase (`סוּס הַמֶּלֶךְ` — *the horse of the king*) and adjective agreement (`הַסּוּס הַטּוֹב` — *the good horse*).
  * **Pronominal Suffixes** attached to nouns (`סוּסִי, סוּסְךָ, סוּסֵךְ, סוּסוֹ, סוּסָהּ`) and prepositions (`לִי, לְךָ, לָךְ, לוֹ, לָהּ`).
* **Koine Greek (`grc`):**
  * Full **Nominative vs. Accusative vs. Dative vs. Genitive** visual minimal pairs where word order is deliberately inverted so the learner relies 100% on case morphology rather than English word-order habits.
  * Personal, demonstrative (`οὗτος, ἐκεῖνος`), and relative pronouns (`ὅς, ἥ, ὅ`) linked via [Clear-Bible/macula-greek](https://github.com/Clear-Bible/macula-greek) participant referent chains.

### Tier 3: Verbal Aspect, Narrative Discourse Chains & Root/Stem Families (400 $\rightarrow$ 800 Words)
* **Biblical Hebrew (`hbo`):**
  * Transition from participles to **Qatal (Suffix)**, **Yiqtol (Prefix)**, and **Wayyiqtol (`וַיֹּאמֶר`, `וַיֵּלֶךְ`, `וַיַּרְא`)** narrative sequences, plus Imperative and Infinitive Construct (`לִשְׁמֹר`).
  * **The 7 Binyanim Contrast Grid:** Using identical triliteral roots across stems so the learner sees valency/causation shifts directly:
    * e.g., Root `מ-ל-ך` or `ק-ד-שׁ` or `כ-ת-ב` or `שׁ-ב-ר`: Simple Active (`Qal`) $\leftrightarrow$ Passive/Middle (`Niphal`) $\leftrightarrow$ Intensive/Factitive (`Piel`) $\leftrightarrow$ Causative (`Hiphil` — *"caused to..."*) $\leftrightarrow$ Reflexive (`Hithpael`).
  * Weak root phonology (Gutturals, `פ״נ`, `פ״י`, `ע״ו/י` Hollow verbs, `ל״ה` III-He verbs).
* **Koine Greek (`grc`):**
  * **Aspectual Triad Contrasts:** Present/Imperfect (ongoing process) vs. Aorist (bounded event) vs. Perfect (resultant state) across Indicative, Imperative, Infinitive, and **Adverbial/Attributive Participles** (including **Genitive Absolute** constructions).
  * Subjunctive mood (`ἵνα`, `ἐάν`, hortatory, prohibitions with `μή + Aorist Subjunctive`).

### Tier 4: Graded Corpus Bridge & Syntactic Scaffolding (800 $\rightarrow$ 1,500+ Words — Intermediate Fluency)
Once a learner masters the top **800–1,000 lemmas** in Biblical Hebrew (which account for **$>85\%$ of all word tokens** in the Hebrew Bible) and the top **1,000 lemmas** in Koine Greek (which account for **$>88\%$ of all word tokens** in the Greek New Testament), KaizenLinguistics transitions from synthetic/visual contrast quads into the **Scaffolded Authentic Text Reader**:
1. **Dynamic Clause Simplification ("Onion Peeling"):**
   Using the constituency/dependency trees in [Clear-Bible/macula-hebrew](https://github.com/Clear-Bible/macula-hebrew) and [Clear-Bible/macula-greek](https://github.com/Clear-Bible/macula-greek), any authentic verse (e.g., Jonah 1, Ruth 1, Genesis 1–3, Mark 1, 1 John 1, John 1) can be presented in **3 progressive syntactic layers**:
   * **Layer 1 (Core Kernel):** Displays only the main clause `Verb + Subject + Object` (stripping adjunct prepositional phrases and relative clauses).
   * **Layer 2 (Expanded Modifiers):** Reveals adjectives, construct chains, and locative/temporal phrases.
   * **Layer 3 (Full Authentic Text):** Full Masoretic / SBLGNT verse with synchronized audio and instant one-click morphological/semantic-domain inspection.
2. **Automated $i+1$ Pericope Selector:**
   Queries the full [ETCBC/bhsa](https://github.com/ETCBC/bhsa) and [morphgnt/sblgnt](https://github.com/morphgnt/sblgnt) databases against the learner's `fsrs_cards` table (`stability >= 7.0`) to surface authentic biblical passages where **$\ge 95\%$ of tokens are already known**, turning the remaining $\le 5\%$ unknown words into pre-reading visual/contextual contrast exercises!
