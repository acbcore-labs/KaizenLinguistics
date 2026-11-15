# 07 — UI/UX Design System: "Atelier Aldus" (Polyglot Scriptorium & Precision Acoustic Instrument)

**Project:** KaizenLinguistics  
**Design Paradigm:** 16th-Century Venetian Polyglot Press (*Aldus Manutius* / *Complutensian Polyglot*) $\times$ Swiss Editorial Grid $\times$ Tactile Brass Acoustic Telemetry  

---

## 1. Anti-Generic Manifesto: What This UI Rejects

Standard AI-generated interfaces default to an unmistakable, sterile SaaS template: `#0f172a` slate-blue backgrounds, `#38bdf8` cyan pills, `14px` rounded plastic cards, system sans-serif fonts, and emoji placeholders (`👼`, `👦`, `🪑`). That aesthetic is jarring and inappropriate for studying **Koine Greek (`grc`)** and **Classical Masoretic Hebrew (`hbo`)**.

**KaizenLinguistics ("Atelier Aldus")** replaces generic SaaS chrome with an original visual language rooted in two historical traditions of mastery:
1. **The Renaissance Humanist Polyglot Folio:** Exposed `1px` architectural hairlines, corner registration marks (`+`), asymmetrical marginalia (*scholia*) columns, Venetian vermilion rubrics (`#B8332A`), Lapis Lazuli syntactic markers (`#1D3B6C`), and museum-grade classical typography (`EB Garamond` / `Cormorant Garamond` / `SBL Greek` / `Frank Ruhl Libre` / `SBL Hebrew`).
2. **The Precision Acoustic & Philological Instrument:** Tactile stepped brass controls, physical keyboard/momentary switches, and a real-time HTML5 `<canvas>` harmonic pitch-contour ($F_0$) and envelope oscilloscope that plots Masoretic cantillation / Greek pitch-accent curves in burnished gold against the learner's voice in vermilion.

---

## 2. Dual-Atmosphere Color Architecture

The interface defaults to **Florentine Vellum (Day Scriptorium)**—honoring the physical reality of ink on cotton rag paper—and switches instantaneously to **Basalt & Gold Leaf (Nocturne Scriptorium)** for low-light study:

| Semantic Token | Florentine Vellum (Default Day Folio) | Basalt Nocturne (Night Scriptorium) | Functional Role |
| :--- | :--- | :--- | :--- |
| `--paper-bg` | `#F4EFE6` *(Raw Cotton Rag)* | `#11100E` *(Volcanic Basalt)* | Primary folio background |
| `--folio-surface` | `#EAE2D3` *(Warm Parchment)* | `#191714` *(Smoked Vellum)* | Contrast plates & scholia panels |
| `--folio-elevated` | `#FAF7F2` *(Alabaster Sheet)* | `#221F1B` *(Charcoal Stone)* | Active stimulus plate |
| `--ink-primary` | `#161412` *(Deep Iron-Gall Ink)* | `#F0ECE1` *(Warm Chalk Ink)* | Primary Greek & Hebrew script |
| `--ink-secondary` | `#4A433B` *(Sepia Bistre)* | `#B5ABA0` *(Ash Parchment)* | Secondary labels & IPA transcriptions |
| `--ink-muted` | `#7D7365` *(Dry Umber)* | `#70685E` *(Weathered Bronze)* | Folio indices, grid coordinates, metadata |
| `--rule-hairline` | `#CEC4B2` *(1px Engraving Rule)* | `#2E2A25` *(1px Basalt Rule)* | Exposed architectural grid borders |
| `--rubric-vermilion` | `#B8332A` *(Venetian Cinnabar)* | `#E05347` *(Luminous Vermilion)* | Bound-morpheme highlights & active rubrics |
| `--lapis-cobalt` | `#1D3B6C` *(Ultramarine Pigment)* | `#5B88C9` *(Byzantine Lapis)* | Syntactic kernel & Nominative/Agent markers |
| `--gold-burnished` | `#9A7428` *(Antiqued Brass)* | `#D4AF5A` *(Byzantine Gold Leaf)* | Reference $F_0$ contour, seals & mastery badges |
| `--verdigris-pass` | `#235E4A` *(Oxidized Copper)* | `#43A685` *(Malachite Green)* | Verified contrast match / high-confidence alignment |

---

## 3. Typography & Orthographic Hierarchy

* **Classical Display & Folio Headers:** `'Cormorant Garamond', 'EB Garamond', Georgia, serif` — small-caps tracking (`letter-spacing: 0.14em`), roman numerals, and italic scholia notes.
* **Polytonic Koine Greek (`grc`):** `'SBL Greek', 'New Athena Unicode', 'Cardo', 'EB Garamond', serif` — rendered at `2.1rem` to `2.6rem` with generous line height (`1.65`) so breathing marks (`῾ / ᾿`), iota subscripts (`ᾳ, ῃ, ῳ`), and circumflex accents (`ᾶ, ῆ, ῶ`) are razor-sharp.
* **Masoretic Biblical Hebrew (`hbo`):** `'SBL Hebrew', 'Ezra SIL', 'Frank Ruhl Libre', 'Cardo', serif` (`dir="rtl"`) — rendered at `2.25rem` to `2.75rem` with full OpenType mark-to-base (`mark`) and mark-to-mark (`mkmk`) positioning so *dagesh*, *niqqud* vowels, and *te'amim* cantillation accents never collide.
* **Telemetry, IPA & Oscilloscope Readouts:** `'JetBrains Mono', 'SF Mono', Menlo, monospace` — tabular numerals (`font-variant-numeric: tabular-nums`) for millisecond alignments, `dB` SNR readouts, and FSRS-6 stability vectors.

---

## 4. Bespoke Visual Primitives (Zero Emojis)

1. **Classical Engraving SVG Plates (`QUAD_SCENE` & `ASPECT_STRIP`):**
   Every contrast card renders a procedural **Classical Line-Art Engraving SVG** with architectural colonnades, geometric figures, directional valency vectors (solid Lapis arrows for Agent $\rightarrow$ Patient, dashed Vermilion arcs for Indirect Object / Dative Recipient, and 3-phase aspectual dials for Imperfective / Aorist / Perfect).
2. **Syntactic Dependency Arcs in the Polyglot Reader:**
   In `reader.html`, clicking any clause renders SVG syntactic dependency curves connecting the finite verb kernel (`V`) to its Subject (`S`), Direct Object (`O`), and prepositional adjuncts.
3. **Dual-Trace Harmonic Oscilloscope (`<canvas>`):**
   Plots the native reference pitch ($F_0$) and amplitude envelope in **Burnished Gold** and superimposes the learner's recorded utterance in **Venetian Vermilion**, marking exact phoneme boundaries (`/h/ | /o/ | /ˈa/ | /ŋɡ/ ...`) along the horizontal time axis.
