/**
 * KAIZEN LINGUISTICS — Workspace 1: Immersion Studio (Contrast Lab)
 * Enforces:
 * 1. Rich English Pedagogical Walkthrough & Instructions for Every Lesson
 * 2. Pure minimal situational contrasts without raw JSON leaks
 * 3. Clear English descriptions on each situational contrast plate
 * 4. Toggleable English meaning and grammatical morpheme analysis
 */

window.Immersion = (function () {
  let catalog = [];
  let currentIndex = 0;
  let currentModality = "EXP_BIMODAL"; // Discovery, Listening, Reading, Speaking
  let startTime = 0;
  let showGloss = false;
  let showWalkthrough = true;

  async function init() {
    await loadCatalog(window.App.activeLang);
  }

  async function loadCatalog(langCode) {
    try {
      const items = await window.App.invoke("get_catalog", { lang: langCode });
      if (items && items.length > 0) {
        catalog = items;
        window._currentCatalog = items;
        currentIndex = 0;
        showGloss = false;
        renderSyllabusList();
        renderCurrentStimulus();
        if (window.Journey && typeof window.Journey.onLanguageChanged === "function") {
          window.Journey.onLanguageChanged();
        }
      }
    } catch (e) {
      console.warn("Failed to load catalog via IPC:", e);
    }
  }

  function setModality(modalityCode) {
    currentModality = modalityCode;
    document.querySelectorAll(".stage-btn").forEach(btn => {
      btn.classList.toggle("active", btn.getAttribute("data-stage") === modalityCode);
    });
    renderCurrentStimulus();
  }

  function renderSyllabusList() {
    const listEl = document.getElementById("immersionConceptList");
    if (!listEl) return;

    listEl.innerHTML = catalog.map((item, idx) => {
      const title = item.title_en || formatConceptFallbackTitle(item);
      const tag = item.category_en || cleanDimensionLabel(item.contrast);
      return `
        <div class="concept-item ${idx === currentIndex ? 'active' : ''}" onclick="window.Immersion.selectConcept(${idx})">
          <div class="concept-item-top">
            <span class="concept-num">${item.roman}</span>
            <span class="concept-tag">${tag}</span>
          </div>
          <div class="concept-title-en">${title}</div>
          <div class="concept-lemma">${item.lemma}</div>
        </div>
      `;
    }).join("");
  }

  function formatConceptFallbackTitle(item) {
    if (item.contrast.includes("CASE_NOM_VS_ACC")) return "Who Does What? (Subject vs. Object)";
    if (item.contrast.includes("NUMBER_SG_VS_PL")) return "One vs. Many (Singular & Plural)";
    if (item.contrast.includes("GENDER")) return "Gender & Agreement";
    if (item.contrast.includes("PREP")) return "Prepositions & Direction";
    if (item.contrast.includes("PERSON")) return "Who is Speaking? (Verb Person)";
    if (item.contrast.includes("ASPECT")) return "Action Over Time (Verbal Aspect)";
    if (item.contrast.includes("VOICE")) return "Acting vs. Being Acted Upon";
    return `Concept ${item.roman}: ${item.lemma}`;
  }

  function cleanDimensionLabel(contrastStr) {
    if (contrastStr.includes("CASE_NOM_VS_ACC")) return "Nom/Acc Roles";
    if (contrastStr.includes("BINYAN")) return "Binyanim Stems";
    if (contrastStr.includes("ASPECT")) return "Aspect Timelines";
    if (contrastStr.includes("GENDER")) return "Gender & Number";
    if (contrastStr.includes("PREP")) return "Prepositions";
    return contrastStr.replace("CASE_", "").replace("_VS_", " / ").replace(/_/g, " ").slice(0, 18);
  }

  function selectConcept(index) {
    if (index >= 0 && index < catalog.length) {
      currentIndex = index;
      showGloss = false;
      renderSyllabusList();
      renderCurrentStimulus();
    }
  }

  function stepConcept(delta) {
    let next = currentIndex + delta;
    if (next < 0) next = catalog.length - 1;
    if (next >= catalog.length) next = 0;
    selectConcept(next);
  }

  function renderCurrentStimulus() {
    if (!catalog[currentIndex]) return;
    const item = catalog[currentIndex];
    const targetPanel = item.panels[item.target_index] || item.panels[0];

    // 1. Render English Pedagogical Walkthrough Card
    const unitTagEl = document.getElementById("walkthroughUnitTag");
    const grammarTagEl = document.getElementById("walkthroughGrammarTag");
    const titleEl = document.getElementById("walkthroughTitle");
    const explanationEl = document.getElementById("walkthroughExplanation");

    const unitNum = Math.floor(currentIndex / 5) + 1;
    const lessonNum = (currentIndex % 5) + 1;

    if (unitTagEl) unitTagEl.textContent = `UNIT ${unitNum} · LESSON ${lessonNum}`;
    if (grammarTagEl) grammarTagEl.textContent = item.category_en || cleanDimensionLabel(item.contrast);
    if (titleEl) titleEl.textContent = item.title_en || formatConceptFallbackTitle(item);
    if (explanationEl) {
      explanationEl.innerHTML = item.explanation_en || `
        In this lesson, examine the single grammatical difference in <strong>${item.lemma}</strong> across each situation. Notice which inflectional morpheme signals the relationship.
      `;
    }

    // 2. Stimulus Meta Badges
    const badgeEl = document.getElementById("immersionUnitBadge");
    if (badgeEl) badgeEl.textContent = item.category_en || cleanDimensionLabel(item.contrast);

    // 3. Target Script Display
    const scriptEl = document.getElementById("immersionTargetScript");
    if (scriptEl) {
      const text = window.App.showPointing ? targetPanel.pointed : targetPanel.unpointed;
      scriptEl.textContent = text;
      scriptEl.classList.toggle("hidden-for-listening", currentModality === "COMP_LISTEN");
    }

    // 4. Target IPA (Guaranteed Clean Text, NEVER Raw JSON)
    const ipaEl = document.getElementById("immersionTargetIpa");
    if (ipaEl) {
      let rawIpa = targetPanel.ipa || "/.../";
      // Safety: parse JSON if a raw string was somehow passed
      if (typeof rawIpa === "string" && rawIpa.trim().startsWith("{")) {
        try {
          const parsed = JSON.parse(rawIpa);
          rawIpa = parsed.ipa || rawIpa;
        } catch (e) {}
      }
      ipaEl.textContent = rawIpa;
      ipaEl.style.display = currentModality === "COMP_LISTEN" ? "none" : "block";
    }

    // 5. English Translation & Morpheme Analysis
    const glossBox = document.getElementById("glossContentBox");
    const glossTrans = document.getElementById("glossTranslationLine");
    const glossMorph = document.getElementById("glossMorphemeLine");
    if (glossBox) {
      glossBox.style.display = showGloss ? "block" : "none";
    }
    if (glossTrans) {
      glossTrans.textContent = item.target_translation_en ? `"${item.target_translation_en}"` : `Target sentence featuring ${item.lemma}.`;
    }
    if (glossMorph) {
      glossMorph.textContent = `${item.lemma} (${item.lemma_gloss_en || item.tag}) · Focus: ${item.category_en || cleanDimensionLabel(item.contrast)}`;
    }

    // 6. Hide feedback insight banner until learner answers
    const feedbackBanner = document.getElementById("feedbackInsightBanner");
    if (feedbackBanner) feedbackBanner.classList.remove("open");

    // 7. Render 4-Plate Situational Contrast Grid
    renderPlates(item);
    startTime = Date.now();

    // Auto-play audio if in Discovery or Listening mode
    if (currentModality === "EXP_BIMODAL" || currentModality === "COMP_LISTEN") {
      playAudio();
    }
  }

  function toggleGloss() {
    showGloss = !showGloss;
    const glossBox = document.getElementById("glossContentBox");
    const btnLabel = document.getElementById("glossToggleLabel");
    if (glossBox) glossBox.style.display = showGloss ? "block" : "none";
    if (btnLabel) btnLabel.textContent = showGloss ? "Hide English Meaning" : "Show English Meaning & Analysis";
  }

  function toggleWalkthrough() {
    showWalkthrough = !showWalkthrough;
    const body = document.getElementById("walkthroughBody");
    const btn = document.getElementById("btnToggleWalkthrough");
    if (body) body.style.display = showWalkthrough ? "block" : "none";
    if (btn) btn.textContent = showWalkthrough ? "Hide Guide ▴" : "Lesson Guide ▾";
  }

  function formatSituationalDescription(role, glyph) {
    if (!role) return glyph || "Target Situation";
    let desc = role;
    desc = desc.replace(/^(SVO_|OVS_|SOV_|VSO_)/, "");
    desc = desc.replace(/_Sees_/g, " sees ");
    desc = desc.replace(/_Hears_/g, " hears ");
    desc = desc.replace(/_Speaks_/g, " speaks ");
    desc = desc.replace(/_Calls_/g, " calls ");
    desc = desc.replace(/_Writes_/g, " writes ");
    desc = desc.replace(/_Sends_/g, " sends ");
    desc = desc.replace(/_Releases_/g, " releases ");
    desc = desc.replace(/Angel_Sees_Brother/g, "Angel sees Brother");
    desc = desc.replace(/Brother_Sees_Angel/g, "Brother sees Angel");
    desc = desc.replace(/Nom_Sg/g, "Singular Subject (one)");
    desc = desc.replace(/Nom_Pl/g, "Plural Subject (many)");
    desc = desc.replace(/Acc_Sg/g, "Singular Object (one)");
    desc = desc.replace(/Acc_Pl/g, "Plural Object (many)");
    desc = desc.replace(/Fem_Sg_Subject/g, "Feminine Subject");
    desc = desc.replace(/Fem_Pl_Subject/g, "Feminine Plural");
    desc = desc.replace(/Masc_Sg_Subject/g, "Masculine Subject");
    desc = desc.replace(/Masc_Pl_Subject/g, "Masculine Plural");
    desc = desc.replace(/Neut_Sg_Nom/g, "Neuter Subject (singular)");
    desc = desc.replace(/Neut_Pl_Nom/g, "Neuter Subject (plural)");
    desc = desc.replace(/Neut_Sg_Acc/g, "Neuter Object (singular)");
    desc = desc.replace(/Neut_Pl_Acc/g, "Neuter Object (plural)");
    desc = desc.replace(/Active_Voice/g, "Active: Actor performs action");
    desc = desc.replace(/Passive_Voice.*|Passive_Plural.*/g, "Passive: Action done unto subject");
    desc = desc.replace(/Imperfective_Unfolding/g, "Present: Ongoing action");
    desc = desc.replace(/Aorist_Bounded_Event/g, "Aorist: Completed event");
    desc = desc.replace(/Perfect_Enduring_State/g, "Perfect: Enduring state");
    desc = desc.replace(/Imperfect_Past_Ongoing/g, "Imperfect: Ongoing past");
    desc = desc.replace(/Masc_Sg_Participle/g, "Masculine Action");
    desc = desc.replace(/Fem_Sg_Participle_Ah_Et/g, "Feminine Action");
    desc = desc.replace(/Masc_Pl_Im/g, "Masculine Plural Action");
    desc = desc.replace(/Fem_Pl_Ot/g, "Feminine Plural Action");
    desc = desc.replace(/_/g, " ");

    return glyph ? `${desc} (${glyph})` : desc;
  }

  function renderPlates(item) {
    const gridEl = document.getElementById("immersionContrastGrid");
    if (!gridEl) return;

    gridEl.innerHTML = item.panels.map((p, idx) => {
      const text = window.App.showPointing ? p.pointed : p.unpointed;
      const svg = generateProceduralEngravingSvg(p.motif, idx, item.contrast);
      const situationRole = p.clean_role || formatSituationalDescription(p.role, p.glyph);

      return `
        <div class="contrast-plate" data-index="${idx}" onclick="window.Immersion.handlePlateChoice(${idx})">
          <div class="plate-header">
            <span class="plate-scenario-letter">Situation ${['A', 'B', 'C', 'D'][idx]}</span>
            <span class="plate-description-text">${situationRole}</span>
          </div>
          <div class="plate-stage">${svg}</div>
          <div class="plate-caption">${text}</div>
        </div>
      `;
    }).join("");
  }

  async function handlePlateChoice(chosenIndex) {
    const item = catalog[currentIndex];
    const latency = Date.now() - startTime;
    const isCorrect = chosenIndex === item.target_index;

    // Visual Feedback on Plates
    const plates = document.querySelectorAll(".contrast-plate");
    plates.forEach((p, idx) => {
      p.classList.remove("selected-correct", "selected-wrong");
      if (idx === item.target_index) {
        p.classList.add("selected-correct");
      } else if (idx === chosenIndex && !isCorrect) {
        p.classList.add("selected-wrong");
      }
    });

    // Display Grammatical Insight Note
    const feedbackBanner = document.getElementById("feedbackInsightBanner");
    const feedbackText = document.getElementById("feedbackInsightText");
    if (feedbackBanner && feedbackText) {
      feedbackText.innerHTML = isCorrect
        ? `<strong>Correct!</strong> ${item.scholia || 'Single-delta contrast successfully distinguished.'}`
        : `<strong>Not quite.</strong> Notice how the target morpheme signals: ${item.scholia || 'Examine the inflectional ending.'}`;
      feedbackBanner.classList.add("open");
    }

    // Trigger Course Journey Lesson Passed celebration if correct
    if (isCorrect && window.Journey && typeof window.Journey.handleLessonPassed === "function") {
      window.Journey.handleLessonPassed();
    }

    // Record review in native FSRS Telemetry
    try {
      const modalityKey = currentModality === "COMP_LISTEN" ? "LISTEN" : "READ";
      const rating = isCorrect ? 3 : 1;
      await window.App.invoke("record_review", {
        lang: window.App.activeLang,
        conceptId: item.id,
        modality: modalityKey,
        rating,
        latencyMs: latency
      });
    } catch (e) {
      console.warn("Failed to record review via IPC:", e);
    }
  }

  function playAudio() {
    if (!catalog[currentIndex]) return;
    try {
      const ctx = new (window.AudioContext || window.webkitAudioContext)();
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.type = "sine";
      const baseFreq = window.App.activeLang === "grc" ? 220 : 185;
      osc.frequency.setValueAtTime(baseFreq, ctx.currentTime);
      osc.frequency.exponentialRampToValueAtTime(baseFreq * 1.25, ctx.currentTime + 0.25);
      osc.frequency.exponentialRampToValueAtTime(baseFreq, ctx.currentTime + 0.65);

      gain.gain.setValueAtTime(0.001, ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.18, ctx.currentTime + 0.05);
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.7);

      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start();
      osc.stop(ctx.currentTime + 0.72);
    } catch (e) {
      console.warn("Web audio playback note:", e);
    }
  }

  function handoffToVoiceStudio() {
    window.App.switchWorkspace("voice");
  }

  // Classical Procedural Line-Art Engraving SVG Generator
  function generateProceduralEngravingSvg(motif, idx, contrast) {
    const isBasalt = window.App.theme === "basalt";
    const strokeCol = isBasalt ? "#D4AF5A" : "#1D3B6C";
    const fillSoft = isBasalt ? "rgba(212,175,90,0.12)" : "rgba(29,59,108,0.08)";
    const vermilion = isBasalt ? "#DF564B" : "#B8332A";

    if (motif === "AGENT_SUBJECT") {
      return `
        <svg viewBox="0 0 160 110" width="100%" height="100%">
          <rect x="15" y="15" width="130" height="80" fill="${fillSoft}" stroke="${strokeCol}" stroke-width="1.5" stroke-dasharray="2 2" />
          <circle cx="50" cy="55" r="20" fill="none" stroke="${strokeCol}" stroke-width="2" />
          <path d="M 50 35 L 50 75 M 30 55 L 70 55" stroke="${strokeCol}" stroke-width="1.5" />
          <path d="M 75 55 L 120 55 M 110 47 L 120 55 L 110 63" fill="none" stroke="${vermilion}" stroke-width="2.5" />
          <circle cx="125" cy="55" r="10" fill="${strokeCol}" />
        </svg>
      `;
    } else if (motif === "PATIENT_OBJECT") {
      return `
        <svg viewBox="0 0 160 110" width="100%" height="100%">
          <rect x="15" y="15" width="130" height="80" fill="${fillSoft}" stroke="${strokeCol}" stroke-width="1.5" stroke-dasharray="2 2" />
          <circle cx="35" cy="55" r="10" fill="${strokeCol}" />
          <path d="M 50 55 L 95 55 M 85 47 L 95 55 L 85 63" fill="none" stroke="${strokeCol}" stroke-width="2" />
          <circle cx="120" cy="55" r="20" fill="none" stroke="${vermilion}" stroke-width="2.5" />
          <path d="M 120 35 L 120 75 M 100 55 L 140 55" stroke="${vermilion}" stroke-width="1.5" />
        </svg>
      `;
    } else if (motif === "VALENCY_CAUSATION") {
      return `
        <svg viewBox="0 0 160 110" width="100%" height="100%">
          <rect x="15" y="15" width="130" height="80" fill="${fillSoft}" stroke="${strokeCol}" stroke-width="1.5" />
          <circle cx="40" cy="55" r="14" fill="none" stroke="${strokeCol}" stroke-width="2" />
          <path d="M 58 45 Q 80 25 102 45" fill="none" stroke="${vermilion}" stroke-width="2" stroke-dasharray="3 3" />
          <circle cx="120" cy="55" r="18" fill="none" stroke="${strokeCol}" stroke-width="2" />
          <rect x="112" y="47" width="16" height="16" fill="${strokeCol}" />
        </svg>
      `;
    } else {
      return `
        <svg viewBox="0 0 160 110" width="100%" height="100%">
          <line x1="25" y1="55" x2="135" y2="55" stroke="${strokeCol}" stroke-width="1.5" />
          <circle cx="40" cy="55" r="6" fill="${strokeCol}" />
          <circle cx="80" cy="55" r="10" fill="none" stroke="${vermilion}" stroke-width="2" />
          <circle cx="120" cy="55" r="6" fill="${strokeCol}" />
        </svg>
      `;
    }
  }

  return {
    init,
    loadCatalog,
    setModality,
    selectConcept,
    stepConcept,
    renderCurrentStimulus,
    handlePlateChoice,
    playAudio,
    handoffToVoiceStudio,
    toggleGloss,
    toggleWalkthrough,
    getCurrentConcept: () => catalog[currentIndex],
    getCatalog: () => catalog,
    get currentIndex() { return currentIndex; },
  };
})();
