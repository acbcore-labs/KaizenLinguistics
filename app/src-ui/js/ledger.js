/**
 * KAIZEN LINGUISTICS — Workspace 4: Memory Ledger (Spaced Repetition & Analytics)
 */

window.Ledger = (function () {
  let deckSummary = null;

  async function init() {
    await refreshLedger();
  }

  function onActivated() {
    refreshLedger();
  }

  async function onLanguageChanged(langCode) {
    await refreshLedger();
  }

  async function refreshLedger() {
    try {
      const summary = await window.App.invoke("get_fsrs_deck_summary", { lang: window.App.activeLang });
      if (summary) {
        deckSummary = summary;
        renderMetrics();
        renderSkills();
        renderChronicle();
      }
    } catch (e) {
      console.warn("Failed to fetch deck summary via IPC:", e);
    }
  }

  function renderMetrics() {
    if (!deckSummary) return;

    const dueEl = document.getElementById("metricDueCount");
    const learnEl = document.getElementById("metricLearningCount");
    const masterEl = document.getElementById("metricMasteredCount");
    const buriedEl = document.getElementById("metricBuriedCount");

    if (dueEl) dueEl.textContent = deckSummary.due_cards;
    if (learnEl) learnEl.textContent = deckSummary.learning_cards;
    if (masterEl) masterEl.textContent = deckSummary.mastered_cards;
    if (buriedEl) buriedEl.textContent = deckSummary.buried_cards;
  }

  function renderSkills() {
    if (!deckSummary || !deckSummary.skills) return;

    const containerEl = document.getElementById("skillsGridContainer");
    if (!containerEl) return;

    const skillNames = {
      LISTEN: "Listening Comprehension",
      READ: "Reading Comprehension",
      SPEAK: "Spoken Production",
      WRITE: "Orthographic Writing"
    };

    containerEl.innerHTML = deckSummary.skills.map(s => {
      // Max expected stability target for mastery is 21 days
      const fillPct = Math.min(100, Math.round((s.avg_stability_days / 21.0) * 100));
      return `
        <div class="skill-card">
          <div class="skill-header">
            <span>${skillNames[s.modality] || s.modality}</span>
            <span style="color:var(--rubric-vermilion);">${s.due_count} Due</span>
          </div>
          <div class="stability-bar-track">
            <div class="stability-bar-fill" style="width: ${fillPct}%;"></div>
          </div>
          <div class="skill-stat-line">
            <span>Avg Stability: ${s.avg_stability_days.toFixed(2)}d</span>
            <span>${s.card_count} Total Cards</span>
          </div>
        </div>
      `;
    }).join("");
  }

  function renderChronicle() {
    if (!deckSummary || !deckSummary.recent_reviews) return;

    const tableEl = document.getElementById("ledgerChronicleTableBody");
    if (!tableEl) return;

    if (deckSummary.recent_reviews.length === 0) {
      tableEl.innerHTML = `<tr><td colspan="5" style="text-align:center;color:var(--ink-muted);padding:14px;">No reviews recorded yet for this language. Complete an exercise in Immersion Studio!</td></tr>`;
      return;
    }

    tableEl.innerHTML = deckSummary.recent_reviews.map(r => `
      <tr>
        <td>${r.modality}</td>
        <td>Rating ${r.rating} (${['', 'Again', 'Hard', 'Good', 'Easy'][r.rating] || 'Good'})</td>
        <td>${r.stability_after.toFixed(2)} days</td>
        <td>${r.latency_ms} ms</td>
        <td>${new Date(r.timestamp_ms).toLocaleTimeString()}</td>
      </tr>
    `).join("");
  }

  async function runIntegrityAudit() {
    try {
      const summary = await window.App.invoke("check_integrity");
      alert(
        `Dual-Database Veritas Audit:\n` +
        `• Koine Greek Archive: ${summary.grc_integrity_ok ? 'VERIFIED (PASS)' : 'FAILED'}\n` +
        `• Biblical Hebrew Archive: ${summary.hbo_integrity_ok ? 'VERIFIED (PASS)' : 'FAILED'}\n` +
        `• Total Units: ${summary.grc_concepts + summary.hbo_concepts}\n` +
        `• FSRS Scheduler: ${summary.fsrs_version}`
      );
    } catch (e) {
      alert(`Integrity Check Error: ${e}`);
    }
  }

  return {
    init,
    onActivated,
    onLanguageChanged,
    refreshLedger,
    runIntegrityAudit,
  };
})();
