/**
 * KAIZEN LINGUISTICS — Workspace 3: Authentic Text Reader (Syntax Peeler)
 */

window.Reader = (function () {
  let pericopes = [];
  let currentPericopeKey = "mark1";
  let currentStratum = 3; // 1 = Kernel, 2 = Expansion, 3 = Full Text

  async function init() {
    await loadPericopes();
  }

  async function loadPericopes() {
    try {
      const data = await window.App.invoke("get_reader_pericopes");
      if (data && data.length > 0) {
        pericopes = data;
        populatePassageSelector();
        renderCurrentPassage();
      }
    } catch (e) {
      console.warn("Failed to load pericopes via IPC:", e);
    }
  }

  function onActivated() {
    renderCurrentPassage();
  }

  async function onLanguageChanged(langCode) {
    // Select first matching passage for this language
    const match = pericopes.find(p => p.lang === langCode);
    if (match) {
      currentPericopeKey = match.key;
    }
    populatePassageSelector();
    renderCurrentPassage();
  }

  function populatePassageSelector() {
    const selectEl = document.getElementById("readerPassageSelect");
    if (!selectEl) return;

    // Filter by active language
    const langPericopes = pericopes.filter(p => p.lang === window.App.activeLang);
    const displayList = langPericopes.length > 0 ? langPericopes : pericopes;

    selectEl.innerHTML = displayList.map(p => `
      <option value="${p.key}" ${p.key === currentPericopeKey ? 'selected' : ''}>
        [${p.coverage_pct}% ${p.readiness_status}] ${p.title}
      </option>
    `).join("");

    if (!displayList.some(p => p.key === currentPericopeKey) && displayList.length > 0) {
      currentPericopeKey = displayList[0].key;
    }
  }

  function selectPassage(key) {
    currentPericopeKey = key;
    renderCurrentPassage();
  }

  function setStratum(stratumNum) {
    currentStratum = stratumNum;
    document.querySelectorAll(".strata-btn").forEach(btn => {
      btn.classList.toggle("active", parseInt(btn.getAttribute("data-stratum")) === stratumNum);
    });

    const noteEl = document.getElementById("strataExplanationNote");
    if (noteEl) {
      if (stratumNum === 1) {
        noteEl.textContent = "Stratum 1 (Core Clause): Finite verb and core arguments only. All modifiers peeled away.";
      } else if (stratumNum === 2) {
        noteEl.textContent = "Stratum 2 (Expansion): Prepositional phrases, adverbials, and genitive qualifiers unveiled.";
      } else {
        noteEl.textContent = "Stratum 3 (Full Text): Complete authentic literature with discourse particles and cantillation.";
      }
    }

    // Apply filtering to tokens
    document.querySelectorAll(".reader-token").forEach(t => {
      const tokenStratum = parseInt(t.getAttribute("data-stratum"));
      t.classList.toggle("stratum-hidden", tokenStratum > currentStratum);
    });

    // Re-render syntax arcs for the active stratum
    renderSyntacticArcsForCurrentPassage();
  }

  function renderCurrentPassage() {
    const passage = pericopes.find(p => p.key === currentPericopeKey);
    if (!passage) return;

    const stageEl = document.getElementById("readerManuscriptStage");
    if (!stageEl) return;

    const statusClass = (passage.readiness_status || "ACCESSIBLE").toLowerCase();

    stageEl.innerHTML = `
      <div class="passage-title-block">
        <div>
          <h2 class="passage-title">${passage.title}</h2>
          <div style="font-family:var(--font-mono);font-size:0.75rem;color:var(--ink-muted);margin-top:4px;">
            ${passage.source}
          </div>
        </div>
        <span class="passage-sub">${passage.lang.toUpperCase()} · AUTHENTIC TEXT</span>
      </div>

      <!-- Real-Time Vocabulary Readiness Gauge -->
      <div class="readiness-gauge-bar">
        <div class="readiness-info-group">
          <span class="readiness-status-badge ${statusClass}">
            ${passage.readiness_status || 'ACCESSIBLE'}
          </span>
          <span>Lexical Coverage: <strong>${passage.coverage_pct || 88.0}%</strong></span>
          <div class="readiness-mini-track">
            <div class="readiness-mini-fill" style="width:${passage.coverage_pct || 88.0}%;"></div>
          </div>
          <span style="color:var(--ink-muted);">(${passage.unique_lemmas || 12} unique lemmas in text)</span>
        </div>
        <div style="color:var(--ink-secondary); font-size:0.68rem;">
          Cross-referenced live with user_telemetry.db FSRS Cards
        </div>
      </div>

      <div class="passage-verses-list" dir="${passage.dir}" style="margin-top:20px;">
        ${passage.verses.map((v, vIdx) => `
          <div class="verse-block" id="verse_block_${vIdx}">
            <span class="verse-ref">${v.ref_text}</span>
            <div class="syntax-arcs-stage" id="arcs_stage_${vIdx}"></div>
            <div class="verse-tokens-line" id="tokens_line_${vIdx}">
              ${v.tokens.map((tok, tIdx) => `
                <span class="reader-token ${tok.s > currentStratum ? 'stratum-hidden' : ''}"
                      id="tok_${vIdx}_${tIdx}"
                      data-stratum="${tok.s}"
                      onclick="window.Reader.inspectWord('${passage.key}', ${vIdx}, ${tIdx}, this)">
                  ${tok.w}
                </span>
              `).join("")}
            </div>
          </div>
        `).join("")}
      </div>
    `;

    setTimeout(renderSyntacticArcsForCurrentPassage, 60);
  }

  function renderSyntacticArcsForCurrentPassage() {
    const passage = pericopes.find(p => p.key === currentPericopeKey);
    if (!passage) return;

    passage.verses.forEach((v, vIdx) => {
      const stage = document.getElementById(`arcs_stage_${vIdx}`);
      const line = document.getElementById(`tokens_line_${vIdx}`);
      if (!stage || !line) return;

      const isBasalt = window.App.theme === "basalt";
      const subjectCol = isBasalt ? "#6A98D8" : "#1D3B6C"; // Lapis
      const objectCol = isBasalt ? "#DF564B" : "#B8332A";  // Vermilion
      const prepCol = isBasalt ? "#D4AF5A" : "#9A7428";    // Gold

      // Find verb kernel, subject, and object indices
      let verbIdx = -1;
      let subjIdx = -1;
      let objIdx = -1;

      v.tokens.forEach((t, idx) => {
        if (t.s > currentStratum) return;
        const dLower = t.d.toLowerCase();
        const mLower = t.m.toLowerCase();
        if (verbIdx === -1 && (dLower.includes("kernel") || mLower.startsWith("v"))) {
          verbIdx = idx;
        } else if (subjIdx === -1 && dLower.includes("subject")) {
          subjIdx = idx;
        } else if (objIdx === -1 && (dLower.includes("object") || dLower.includes("patient"))) {
          objIdx = idx;
        }
      });

      const lineRect = line.getBoundingClientRect();
      const w = lineRect.width || 800;
      let svgHtml = `<svg viewBox="0 0 ${w} 44" preserveAspectRatio="none">`;
      svgHtml += `
        <defs>
          <marker id="arrowSubj_${vIdx}" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="${subjectCol}" />
          </marker>
          <marker id="arrowObj_${vIdx}" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="${objectCol}" />
          </marker>
        </defs>
      `;

      function getCenterRelX(tIdx) {
        const el = document.getElementById(`tok_${vIdx}_${tIdx}`);
        if (!el) return null;
        const r = el.getBoundingClientRect();
        return r.left - lineRect.left + r.width / 2;
      }

      // Draw Subject -> Verb Arc
      if (subjIdx !== -1 && verbIdx !== -1) {
        const x1 = getCenterRelX(subjIdx);
        const x2 = getCenterRelX(verbIdx);
        if (x1 !== null && x2 !== null) {
          const midX = (x1 + x2) / 2;
          svgHtml += `
            <path d="M ${x1} 40 Q ${midX} 6 ${x2} 36" fill="none" stroke="${subjectCol}" stroke-width="2" marker-end="url(#arrowSubj_${vIdx})" />
            <text x="${midX}" y="12" fill="${subjectCol}" font-family="var(--font-mono)" font-size="10" text-anchor="middle" font-weight="bold">Subject</text>
          `;
        }
      }

      // Draw Verb -> Object Arc
      if (verbIdx !== -1 && objIdx !== -1) {
        const x1 = getCenterRelX(verbIdx);
        const x2 = getCenterRelX(objIdx);
        if (x1 !== null && x2 !== null) {
          const midX = (x1 + x2) / 2;
          svgHtml += `
            <path d="M ${x1} 36 Q ${midX} 8 ${x2} 36" fill="none" stroke="${objectCol}" stroke-width="2" marker-end="url(#arrowObj_${vIdx})" />
            <text x="${midX}" y="14" fill="${objectCol}" font-family="var(--font-mono)" font-size="10" text-anchor="middle" font-weight="bold">Direct Object</text>
          `;
        }
      }

      svgHtml += `</svg>`;
      stage.innerHTML = svgHtml;
    });
  }

  function inspectWord(passageKey, verseIdx, tokenIdx, el) {
    const passage = pericopes.find(p => p.key === passageKey);
    if (!passage) return;
    const token = passage.verses[verseIdx].tokens[tokenIdx];
    if (!token) return;

    document.querySelectorAll(".reader-token").forEach(t => t.classList.remove("active-inspect"));
    if (el) el.classList.add("active-inspect");

    const cardEl = document.getElementById("readerWordInspectCard");
    if (!cardEl) return;

    cardEl.innerHTML = `
      <div>
        <div class="inspect-field-label">Selected Word</div>
        <div class="inspect-field-val target-script">${token.w}</div>
        <div style="font-family:var(--font-mono);font-size:0.75rem;color:var(--gold-burnished);">${token.ipa}</div>
      </div>
      <div>
        <div class="inspect-field-label">Dictionary Root / Lemma</div>
        <div class="inspect-field-val">${token.l}</div>
      </div>
      <div>
        <div class="inspect-field-label">Grammatical Code &amp; Role</div>
        <div class="inspect-field-val">${token.m}</div>
        <div style="font-size:0.78rem;color:var(--ink-secondary);">${token.d}</div>
      </div>
      <div>
        <div class="inspect-field-label">English Philological Gloss</div>
        <div class="inspect-field-val" style="color:var(--rubric-vermilion);">${token.lex}</div>
        <div style="font-size:0.75rem;color:var(--ink-muted);">Stratum ${token.s} Layer</div>
      </div>
    `;
    cardEl.classList.add("open");
  }

  function playPassageRecitation() {
    window.Immersion.playAudio();
  }

  return {
    init,
    onActivated,
    onLanguageChanged,
    selectPassage,
    setStratum,
    inspectWord,
    playPassageRecitation,
  };
})();
