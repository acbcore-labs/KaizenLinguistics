/**
 * KAIZEN LINGUISTICS — Course Progression Journey & Roadmap Engine
 * Manages:
 * 1. Unit & Lesson Progression Tree (70 concepts grouped into 14 units of 5 lessons)
 * 2. Learner Mastery State & Unlock Flow per Profile
 * 3. Focused Lesson Studio Launching (Uncluttered, Single-Focus Stage)
 * 4. Lesson Completion Rewards & FSRS Hand-off
 */

window.Journey = (function () {
  let completedLessons = new Set();
  let explorerMode = false;
  let currentLessonIndex = 0;

  // 14 Pedagogical Units for the 70 Concepts
  const UNIT_DEFINITIONS_GRC = [
    {
      unitNum: 1,
      title: "The Sentence Core: Subject vs. Object",
      objective: "Master how inflectional endings (-ος vs. -ον) determine who does the action, regardless of word order.",
      startIdx: 0,
      endIdx: 4
    },
    {
      unitNum: 2,
      title: "Interaction, Motion & Space",
      objective: "Acquire prepositions (ἐν, εἰς, ἐκ) and dative recipient cases for giving and moving.",
      startIdx: 5,
      endIdx: 9
    },
    {
      unitNum: 3,
      title: "Pronouns, Roots & Stem Shifts",
      objective: "Navigate 2nd Aorist root changes, 3rd declension consonant stems, and demonstratives.",
      startIdx: 10,
      endIdx: 14
    },
    {
      unitNum: 4,
      title: "Verbal Moods & Living Literature",
      objective: "Participles, subjunctives, purpose clauses, and the John 1 Prologue syntactic ladder.",
      startIdx: 15,
      endIdx: 19
    },
    {
      unitNum: 5,
      title: "Prepositional Governance: Genitive vs. Accusative",
      objective: "Deepen understanding of prepositions governing multiple cases with contrasting meanings.",
      startIdx: 20,
      endIdx: 24
    },
    {
      unitNum: 6,
      title: "Spatial Prepositions: Agency & Vicinity",
      objective: "Learn agents with ὑπό, accompaniment with μετά, and substitution with ἀντί.",
      startIdx: 25,
      endIdx: 29
    },
    {
      unitNum: 7,
      title: "Contract Verbs: -αω, -εω, -οω",
      objective: "Recognize vowel contractions in high-frequency verbs of speaking, loving, and fulfilling.",
      startIdx: 30,
      endIdx: 34
    },
    {
      unitNum: 8,
      title: "Future & Aorist Stem Stems",
      objective: "Observe mute and dental consonant stem shifts under the sigma tense morpheme.",
      startIdx: 35,
      endIdx: 39
    },
    {
      unitNum: 9,
      title: "Liquid Verbs & Nasal Shifts",
      objective: "Master liquid aorists without sigma and internal stem vowel lengthening.",
      startIdx: 40,
      endIdx: 44
    },
    {
      unitNum: 10,
      title: "The Subjunctive Mood in Contingency",
      objective: "Acquire lengthened vowels indicating purpose, condition, and exhortation.",
      startIdx: 45,
      endIdx: 49
    },
    {
      unitNum: 11,
      title: "Command & Prohibition: The Imperative",
      objective: "Contrast present ongoing commands with aorist specific actions.",
      startIdx: 50,
      endIdx: 54
    },
    {
      unitNum: 12,
      title: "Participles: Action in Time & Circumstance",
      objective: "Read temporal, causal, and circumstantial verbal adjectives in flow.",
      startIdx: 55,
      endIdx: 59
    },
    {
      unitNum: 13,
      title: "Infinitival Constructions & Result",
      objective: "Articular infinitives expressing time, cause, and purpose in discourse.",
      startIdx: 60,
      endIdx: 64
    },
    {
      unitNum: 14,
      title: "Mastery: Complex Subordination & Discourse",
      objective: "Integrate relative clauses, conditional periods, and authentic classical rhetoric.",
      startIdx: 65,
      endIdx: 69
    }
  ];

  const UNIT_DEFINITIONS_HBO = [
    {
      unitNum: 1,
      title: "The Core Sentence & Gender Agreement",
      objective: "Acquire masculine vs. feminine participles, natural dual body parts, and the direct object marker אֵת.",
      startIdx: 0,
      endIdx: 4
    },
    {
      unitNum: 2,
      title: "Construct Chains, Pronouns & Past Action",
      objective: "Master chained nouns (Smikhut), pronominal suffixes, and Qatal past verb suffixes.",
      startIdx: 5,
      endIdx: 9
    },
    {
      unitNum: 3,
      title: "The Narrative Backbone & Derived Stems",
      objective: "Wayyiqtol storytelling chains and the active vs. passive Binyan contrasts (Qal, Niphal, Piel, Hiphil).",
      startIdx: 10,
      endIdx: 14
    },
    {
      unitNum: 4,
      title: "Weak Roots & Ancient Cantillation",
      objective: "Navigate guttural roots, hollow roots, geminate roots, and authentic Tiberian prose.",
      startIdx: 15,
      endIdx: 19
    },
    {
      unitNum: 5,
      title: "Inseparable Preposition Dynamics",
      objective: "Vocalic pointing shifts when prepositions merge with definite articles and gutturals.",
      startIdx: 20,
      endIdx: 24
    },
    {
      unitNum: 6,
      title: "Pronominal Suffixes on Prepositions",
      objective: "Master attached endings on prepositions like בִּי, בְּךָ, לָנוּ, and אִתּוֹ.",
      startIdx: 25,
      endIdx: 29
    },
    {
      unitNum: 7,
      title: "The Intensive & Factitive: Binyan Piel",
      objective: "Recognize the doubling dagesh forte in the middle root consonant for causative action.",
      startIdx: 30,
      endIdx: 34
    },
    {
      unitNum: 8,
      title: "The Passive of Intensity: Binyan Pual",
      objective: "Differentiate active factitive Piel from its passive counterpart Pual.",
      startIdx: 35,
      endIdx: 39
    },
    {
      unitNum: 9,
      title: "The Full Causative: Binyan Hiphil",
      objective: "Identify the he-prefix and hireq-yod vowel pattern causing an action to occur.",
      startIdx: 40,
      endIdx: 44
    },
    {
      unitNum: 10,
      title: "Passive Causation: Binyan Hophal",
      objective: "Acquire the qamats-qatan vowel shift signaling passive causation.",
      startIdx: 45,
      endIdx: 49
    },
    {
      unitNum: 11,
      title: "Reflexive & Reciprocal: Binyan Hithpael",
      objective: "Recognize the hit-prefix denoting self-directed or mutual interaction.",
      startIdx: 50,
      endIdx: 54
    },
    {
      unitNum: 12,
      title: "Weak Verb Paradigms: I-Guttural & I-Nun",
      objective: "Observe assimilating nun (dagesh in second root) and compensatory vowel lengthening.",
      startIdx: 55,
      endIdx: 59
    },
    {
      unitNum: 13,
      title: "Hollow Roots & III-He Stems",
      objective: "Two-consonant middle vowel roots (ע״ו / ע״י) and apocopated third-he jussives.",
      startIdx: 60,
      endIdx: 64
    },
    {
      unitNum: 14,
      title: "Mastery: Poetic Oracles & Narrative Flow",
      objective: "Read Genesis 1 and classical prose with unbroken linguistic intuition.",
      startIdx: 65,
      endIdx: 69
    }
  ];

  function getActiveUnitDefinitions() {
    return window.App.activeLang === "hbo" ? UNIT_DEFINITIONS_HBO : UNIT_DEFINITIONS_GRC;
  }

  function loadProgress() {
    const userId = window.App.activeUser ? window.App.activeUser.user_id : "default";
    const lang = window.App.activeLang;
    const key = `kaizen_progress_${userId}_${lang}`;
    try {
      const saved = localStorage.getItem(key);
      if (saved) {
        completedLessons = new Set(JSON.parse(saved));
      } else {
        completedLessons = new Set();
      }
    } catch (e) {
      completedLessons = new Set();
    }

    // Determine current next lesson
    currentLessonIndex = 0;
    while (completedLessons.has(currentLessonIndex) && currentLessonIndex < 69) {
      currentLessonIndex++;
    }
  }

  function saveProgress() {
    const userId = window.App.activeUser ? window.App.activeUser.user_id : "default";
    const lang = window.App.activeLang;
    const key = `kaizen_progress_${userId}_${lang}`;
    try {
      localStorage.setItem(key, JSON.stringify(Array.from(completedLessons)));
    } catch (e) {}
  }

  function init() {
    loadProgress();
    renderCourseRoadmap();
  }

  function onLanguageChanged() {
    loadProgress();
    renderCourseRoadmap();
  }

  /* -------------------------------------------------------------------------- */
  /* Roadmap Rendering                                                          */
  /* -------------------------------------------------------------------------- */

  function renderCourseRoadmap() {
    const catalog = window.Immersion.getCurrentConcept ? getFullCatalog() : [];
    const units = getActiveUnitDefinitions();
    const lang = window.App.activeLang;
    const langName = lang === "hbo" ? "Biblical Hebrew" : "Koine Greek";
    const autonym = lang === "hbo" ? "עִבְרִית מִקְרָאִית" : "Κοινή Ἑλληνική";

    // 1. Update Hero Card
    const autonymEl = document.getElementById("journeyHeroAutonym");
    const titleEl = document.getElementById("journeyHeroTitle");
    const progressTextEl = document.getElementById("journeyProgressText");
    const progressFillEl = document.getElementById("journeyProgressFill");
    const continueBtn = document.getElementById("journeyHeroContinueBtn");

    const completedCount = completedLessons.size;
    const totalCount = 70;
    const pct = ((completedCount / totalCount) * 100).toFixed(1);

    if (autonymEl) autonymEl.textContent = autonym;
    if (titleEl) titleEl.textContent = `${langName} Learning Journey`;
    if (progressTextEl) {
      progressTextEl.textContent = `${completedCount} of ${totalCount} Lessons Mastered (${pct}%) · Level A1 Foundation`;
    }
    if (progressFillEl) progressFillEl.style.width = `${pct}%`;

    const nextItem = catalog[currentLessonIndex] || {
      roman: "I",
      lemma: lang === "hbo" ? "יָשַׁב" : "ἄγγελος",
      title_en: lang === "hbo" ? "Who is Sitting? (Participle Agreement)" : "Who Does What? (Subject vs. Object)"
    };

    if (continueBtn) {
      continueBtn.innerHTML = `
        <span>▶ Continue: Lesson ${currentLessonIndex + 1} — ${nextItem.title_en || nextItem.lemma}</span>
        <span style="font-size:1.1rem;">➔</span>
      `;
      continueBtn.onclick = () => startLesson(currentLessonIndex);
    }

    // 2. Render Units Progression Tree
    const treeEl = document.getElementById("unitsProgressionTree");
    if (!treeEl) return;

    treeEl.innerHTML = units.map(u => {
      // Check unit status
      let unitCompletedCount = 0;
      for (let i = u.startIdx; i <= u.endIdx; i++) {
        if (completedLessons.has(i)) unitCompletedCount++;
      }

      const isCompleted = unitCompletedCount === (u.endIdx - u.startIdx + 1);
      const isCurrent = currentLessonIndex >= u.startIdx && currentLessonIndex <= u.endIdx;
      const isLocked = !explorerMode && u.startIdx > 0 && !completedLessons.has(u.startIdx - 1) && !isCurrent;

      const badgeText = isCompleted ? "COMPLETED ✓" : (isCurrent ? "IN PROGRESS" : (isLocked ? "LOCKED 🔒" : "AVAILABLE"));
      const badgeClass = isCompleted ? "completed" : (isCurrent ? "in-progress" : "");

      // Render lessons in this unit
      let lessonsHtml = "";
      for (let idx = u.startIdx; idx <= u.endIdx; idx++) {
        const item = catalog[idx] || {
          id: `concept_${idx}`,
          roman: toRoman(idx + 1),
          lemma: "...",
          title_en: `Lesson ${idx + 1}`,
          category_en: "Core Grammar"
        };

        const isLessonDone = completedLessons.has(idx);
        const isLessonNext = idx === currentLessonIndex;
        const isLessonLocked = !explorerMode && !isLessonDone && !isLessonNext;

        let cardClass = "lesson-journey-card";
        let statusBadge = "";
        let buttonHtml = "";

        if (isLessonDone) {
          cardClass += " completed available";
          statusBadge = `<span class="lesson-card-status" style="color:var(--verdigris-pass);">Passed ✓</span>`;
          buttonHtml = `<button class="lesson-start-btn" onclick="window.Journey.startLesson(${idx})">Review Lesson</button>`;
        } else if (isLessonNext) {
          cardClass += " next-up available";
          statusBadge = `<span class="lesson-card-status" style="color:var(--rubric-vermilion);">Next Up ➔</span>`;
          buttonHtml = `<button class="lesson-start-btn next-up-btn" onclick="window.Journey.startLesson(${idx})">Start Lesson ➔</button>`;
        } else if (isLessonLocked) {
          cardClass += " locked";
          statusBadge = `<span class="lesson-card-status" style="color:var(--ink-muted);">Locked 🔒</span>`;
          buttonHtml = `<button class="lesson-start-btn" disabled style="opacity:0.5;cursor:not-allowed;">Locked</button>`;
        } else {
          cardClass += " available";
          statusBadge = `<span class="lesson-card-status" style="color:var(--ink-secondary);">Unlocked</span>`;
          buttonHtml = `<button class="lesson-start-btn" onclick="window.Journey.startLesson(${idx})">Start Lesson</button>`;
        }

        lessonsHtml += `
          <div class="${cardClass}" onclick="${!isLessonLocked ? `window.Journey.startLesson(${idx})` : ''}">
            <div>
              <div class="lesson-card-header">
                <span class="lesson-card-num">LESSON ${idx + 1} · UNIT ${u.unitNum}</span>
                ${statusBadge}
              </div>
              <div class="lesson-card-title">${item.title_en || `Concept ${item.roman}`}</div>
            </div>
            <div>
              <div class="lesson-card-lemma">
                <span class="lesson-lemma-text">${item.lemma}</span>
                <span class="lesson-grammar-tag">${item.category_en || 'Grammar Focus'}</span>
              </div>
              <div style="margin-top:10px;">${buttonHtml}</div>
            </div>
          </div>
        `;
      }

      return `
        <div class="unit-cluster-card ${isCurrent ? 'active-unit' : ''}">
          <div class="unit-cluster-header">
            <div class="unit-cluster-title-box">
              <span class="unit-cluster-number">UNIT ${u.unitNum}</span>
              <h3 class="unit-cluster-name">${u.title}</h3>
              <p class="unit-cluster-objective">${u.objective}</p>
            </div>
            <span class="unit-cluster-badge ${badgeClass}">${badgeText}</span>
          </div>
          <div class="lessons-grid">${lessonsHtml}</div>
        </div>
      `;
    }).join("");
  }

  function getFullCatalog() {
    if (window.Immersion && typeof window.Immersion.getCatalog === "function") {
      const cat = window.Immersion.getCatalog();
      if (cat && cat.length > 0) return cat;
    }
    const item = window.Immersion && window.Immersion.getCurrentConcept ? window.Immersion.getCurrentConcept() : null;
    return (window._currentCatalog || (item ? [item] : []));
  }

  /* -------------------------------------------------------------------------- */
  /* Focused Lesson Launching                                                   */
  /* -------------------------------------------------------------------------- */

  function startLesson(conceptIndex) {
    currentLessonIndex = conceptIndex;

    // Hide success celebration banner
    const banner = document.getElementById("lessonSuccessBanner");
    if (banner) banner.classList.remove("open");

    // Select concept in Immersion controller
    window.Immersion.selectConcept(conceptIndex);

    // Update breadcrumb
    const catalog = getFullCatalog();
    const item = catalog[conceptIndex] || window.Immersion.getCurrentConcept();
    const unitNum = Math.floor(conceptIndex / 5) + 1;
    const titleText = item ? (item.title_en || item.lemma) : `Lesson ${conceptIndex + 1}`;

    const breadcrumbEl = document.getElementById("lessonBreadcrumbTitle");
    if (breadcrumbEl) {
      breadcrumbEl.textContent = `Unit ${unitNum} · Lesson ${conceptIndex + 1}: ${titleText}`;
    }

    // Switch view to focused lesson stage
    window.App.switchView("lesson");
    window.scrollTo({ top: 0, behavior: "smooth" });
  }

  function handleLessonPassed() {
    completedLessons.add(currentLessonIndex);
    saveProgress();

    const catalog = getFullCatalog();
    const item = catalog[currentLessonIndex] || window.Immersion.getCurrentConcept();
    const nextIdx = currentLessonIndex + 1;
    const nextItem = catalog[nextIdx];

    // Show celebratory success banner
    const banner = document.getElementById("lessonSuccessBanner");
    const titleEl = document.getElementById("successBannerTitle");
    const msgEl = document.getElementById("successBannerMessage");
    const nextBtn = document.getElementById("successNextBtn");

    if (titleEl) {
      titleEl.textContent = `Lesson ${currentLessonIndex + 1} Passed!`;
    }
    if (msgEl) {
      msgEl.innerHTML = `
        You have successfully acquired <strong>${item ? item.title_en : 'this concept'}</strong>.
        This concept is now scheduled into your <strong>FSRS-6 Memory Ledger</strong> across all 4 learning modalities.
      `;
    }

    if (nextBtn) {
      if (nextIdx < 70) {
        nextBtn.textContent = `Continue to Lesson ${nextIdx + 1} ➔`;
        nextBtn.onclick = () => startLesson(nextIdx);
        nextBtn.style.display = "inline-block";
      } else {
        nextBtn.textContent = `Course Completed! View Memory Ledger ➔`;
        nextBtn.onclick = () => window.App.switchWorkspace("ledger");
      }
    }

    if (banner) {
      banner.classList.add("open");
      banner.scrollIntoView({ behavior: "smooth", block: "center" });
    }
  }

  function toggleExplorerMode() {
    explorerMode = !explorerMode;
    const btn = document.getElementById("btnExplorerMode");
    if (btn) {
      btn.textContent = explorerMode ? "🔒 Re-lock Sequential Progression" : "🔓 Unlock All Lessons (Explorer Mode)";
    }
    renderCourseRoadmap();
  }

  function toRoman(num) {
    const romans = [
      "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X",
      "XI", "XII", "XIII", "XIV", "XV", "XVI", "XVII", "XVIII", "XIX", "XX",
      "XXI", "XXII", "XXIII", "XXIV", "XXV", "XXVI", "XXVII", "XXVIII", "XXIX", "XXX",
      "XXXI", "XXXII", "XXXIII", "XXXIV", "XXXV", "XXXVI", "XXXVII", "XXXVIII", "XXXIX", "XL",
      "XLI", "XLII", "XLIII", "XLIV", "XLV", "XLVI", "XLVII", "XLVIII", "XLIX", "L",
      "LI", "LII", "LIII", "LIV", "LV", "LVI", "LVII", "LVIII", "LIX", "LX",
      "LXI", "LXII", "LXIII", "LXIV", "LXV", "LXVI", "LXVII", "LXVIII", "LXIX", "LXX"
    ];
    return romans[num - 1] || `${num}`;
  }

  return {
    init,
    onLanguageChanged,
    renderCourseRoadmap,
    startLesson,
    handleLessonPassed,
    toggleExplorerMode,
    get currentLessonIndex() { return currentLessonIndex; },
    get completedLessons() { return completedLessons; },
  };
})();
