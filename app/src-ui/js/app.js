/**
 * KAIZEN LINGUISTICS — Application Shell & Multi-View Router
 * Orchestrates:
 * 1. Landing Page & Learner Profile Management (Sign-in / Multi-User)
 * 2. Dedicated Language Selection Screen ("Choose a Language to Learn")
 * 3. Course Progression Journey & Roadmap (Uncluttered Unit Progression)
 * 4. Focused Lesson Studio (Distraction-Free, Single-Lesson Stage)
 * 5. Advanced Studios (Voice Studio, Authentic Reader, Memory Ledger, Method Guide)
 */

window.App = (function () {
  let activeView = "landing"; // "landing", "languages", "course", "lesson", "studios"
  let activeLang = "grc"; // Active target language ("grc" or "hbo")
  let activeWorkspace = "voice"; // Within studios view
  let theme = "vellum";
  let showPointing = true;
  let availableLanguages = [];
  let userProfiles = [];
  let activeUser = null;

  // Robust Tauri IPC invocation helper with graceful web fallback
  async function invoke(cmd, args = {}) {
    if (window.__TAURI__ && window.__TAURI__.core && typeof window.__TAURI__.core.invoke === "function") {
      return await window.__TAURI__.core.invoke(cmd, args);
    }
    if (window.__TAURI__ && typeof window.__TAURI__.invoke === "function") {
      return await window.__TAURI__.invoke(cmd, args);
    }
    console.warn(`[Tauri IPC Mock] Invoking '${cmd}' with args:`, args);
    return mockIpcFallback(cmd, args);
  }

  function mockIpcFallback(cmd, args) {
    if (cmd === "get_available_languages") {
      return Promise.resolve([
        {
          code: "grc",
          name: "Koine Greek",
          autonym: "Κοινή Ἑλληνική",
          family: "Hellenic · Indo-European",
          script_direction: "ltr",
          concept_count: 70,
          pack_id: "klpack_grc_koine_v1",
          semver: "0.3.0"
        },
        {
          code: "hbo",
          name: "Biblical Hebrew",
          autonym: "עִבְרִית מִקְרָאִית",
          family: "Northwest Semitic · Afroasiatic",
          script_direction: "rtl",
          concept_count: 70,
          pack_id: "klpack_hbo_masoretic_v1",
          semver: "0.3.0"
        }
      ]);
    }
    if (cmd === "get_user_profiles") {
      return Promise.resolve([
        {
          user_id: "user_josh",
          display_name: "Josh Shay",
          avatar: "🏛️",
          created_at_ms: Date.now() - 86400000 * 3,
          last_active_ms: Date.now(),
          current_lang: "grc",
          streak_days: 1,
          total_reviews: 42
        }
      ]);
    }
    if (cmd === "select_user_profile") {
      return Promise.resolve({
        user_id: args.userId || "user_josh",
        display_name: "Josh Shay",
        avatar: "🏛️",
        created_at_ms: Date.now() - 86400000 * 3,
        last_active_ms: Date.now(),
        current_lang: "grc",
        streak_days: 1,
        total_reviews: 42
      });
    }
    if (cmd === "create_user_profile") {
      return Promise.resolve({
        user_id: `user_${Date.now()}`,
        display_name: args.displayName || "New Scholar",
        avatar: args.avatar || "🦉",
        created_at_ms: Date.now(),
        last_active_ms: Date.now(),
        current_lang: "grc",
        streak_days: 1,
        total_reviews: 0
      });
    }
    if (cmd === "set_user_language") {
      return Promise.resolve();
    }
    if (cmd === "check_integrity") {
      return Promise.resolve({
        grc_integrity_ok: true,
        hbo_integrity_ok: true,
        grc_concepts: 70,
        hbo_concepts: 70,
        fsrs_version: "fsrs-6-pinned-21param-v0.2.0"
      });
    }
    return Promise.reject(`Command ${cmd} not mocked in web fallback`);
  }

  async function init() {
    // 1. Query available languages
    try {
      availableLanguages = await invoke("get_available_languages");
      renderLanguageModal();
      renderLanguageChoices();
    } catch (e) {
      console.warn("Failed to query available languages:", e);
    }

    // 2. Query learner profiles
    await loadProfiles();

    // 3. Initialize workspace modules
    window.Tooltips.init();
    await window.Immersion.init();
    await window.Voice.init();
    await window.Reader.init();
    await window.Ledger.init();
    if (window.Journey) window.Journey.init();

    // 4. Check for active user session in localStorage
    const savedUserId = localStorage.getItem("kaizen_active_user_id");
    const savedLang = localStorage.getItem("kaizen_active_lang");
    if (savedLang) {
      activeLang = savedLang;
    }

    if (savedUserId) {
      const found = userProfiles.find(p => p.user_id === savedUserId);
      if (found) {
        await selectProfile(savedUserId, false); // select without auto-advancing
        // User is returning: show their Course Journey roadmap!
        switchView("course");
        return;
      }
    }

    // Default to Landing Page for clear profile selection
    switchView("landing");
  }

  /* -------------------------------------------------------------------------- */
  /* Top-Level View Routing                                                     */
  /* -------------------------------------------------------------------------- */

  function switchView(viewName) {
    activeView = viewName;

    document.querySelectorAll(".app-view").forEach(el => {
      el.classList.remove("active");
    });

    const targetEl = document.getElementById(`view_${viewName}`);
    if (targetEl) {
      targetEl.classList.add("active");
    }

    closeUserDropdown();
    closeLanguageModal();

    if (viewName === "landing") {
      renderProfiles();
    } else if (viewName === "languages") {
      renderLanguageChoices();
    } else if (viewName === "course") {
      updateLanguageDisplay();
      updateUserChipDisplay();
      if (window.Journey) window.Journey.renderCourseRoadmap();
    } else if (viewName === "lesson") {
      updateLanguageDisplay();
      updateUserChipDisplay();
    } else if (viewName === "studios") {
      updateLanguageDisplay();
      updateUserChipDisplay();
    }
  }

  /* -------------------------------------------------------------------------- */
  /* Learner Profiles Management                                                */
  /* -------------------------------------------------------------------------- */

  async function loadProfiles() {
    try {
      userProfiles = await invoke("get_user_profiles");
      renderProfiles();
    } catch (e) {
      console.warn("Failed to load user profiles from IPC:", e);
    }
  }

  function renderProfiles() {
    const listEl = document.getElementById("profilesList");
    if (!listEl) return;

    if (!userProfiles || userProfiles.length === 0) {
      listEl.innerHTML = `<p style="color:var(--ink-muted);font-size:0.85rem;">No learner profiles found. Create one below to begin!</p>`;
      return;
    }

    listEl.innerHTML = userProfiles.map(p => {
      const isActive = activeUser && activeUser.user_id === p.user_id;
      const langName = p.current_lang === "hbo" ? "Biblical Hebrew" : "Koine Greek";
      return `
        <div class="profile-card ${isActive ? 'active-user' : ''}" onclick="window.App.selectProfile('${p.user_id}', true)">
          <div class="profile-card-top">
            <div class="profile-avatar">${p.avatar || '🏛️'}</div>
            <div class="profile-details">
              <div class="profile-name">${p.display_name}</div>
              <span class="profile-streak-badge">🔥 ${p.streak_days || 1} Day Streak</span>
            </div>
          </div>
          <div class="profile-stats-row">
            <span>Language: <strong>${langName}</strong></span>
            <span>${p.total_reviews || 0} Lifetime Reviews</span>
          </div>
          <button class="profile-enter-btn" onclick="event.stopPropagation(); window.App.selectProfile('${p.user_id}', true)">
            ${isActive ? 'Continue Learning ➔' : 'Select Profile ➔'}
          </button>
        </div>
      `;
    }).join("");
  }

  async function selectProfile(userId, advanceToLanguages = true) {
    try {
      activeUser = await invoke("select_user_profile", { userId });
      localStorage.setItem("kaizen_active_user_id", activeUser.user_id);
      if (activeUser.current_lang) {
        activeLang = activeUser.current_lang;
        localStorage.setItem("kaizen_active_lang", activeLang);
      }
      updateUserChipDisplay();
      renderProfiles();

      if (advanceToLanguages) {
        // Once profile is chosen, present the Language Selection screen
        switchView("languages");
      }
    } catch (e) {
      console.warn("Failed to select profile via IPC:", e);
    }
  }

  let selectedAvatar = "🏛️";

  function setFormAvatar(avatar) {
    selectedAvatar = avatar;
    document.querySelectorAll(".avatar-option-btn").forEach(btn => {
      btn.classList.toggle("selected", btn.getAttribute("data-avatar") === avatar);
    });
  }

  function toggleCreateProfileForm() {
    const form = document.getElementById("createProfileForm");
    if (form) form.classList.toggle("open");
  }

  async function handleCreateProfileSubmit(e) {
    if (e) e.preventDefault();
    const nameInput = document.getElementById("newProfileName");
    const name = nameInput ? nameInput.value.trim() : "";
    if (!name) return;

    try {
      const newProfile = await invoke("create_user_profile", {
        displayName: name,
        avatar: selectedAvatar
      });
      userProfiles.unshift(newProfile);
      if (nameInput) nameInput.value = "";
      toggleCreateProfileForm();
      await selectProfile(newProfile.user_id, true);
    } catch (err) {
      console.error("Failed to create profile:", err);
    }
  }

  function signOut() {
    localStorage.removeItem("kaizen_active_user_id");
    activeUser = null;
    closeUserDropdown();
    switchView("landing");
  }

  function updateUserChipDisplay() {
    document.querySelectorAll(".user-chip-btn").forEach(chipEl => {
      if (activeUser) {
        chipEl.innerHTML = `
          <span>${activeUser.avatar || '🏛️'}</span>
          <strong>${activeUser.display_name}</strong>
          <span class="profile-streak-badge" style="font-size:0.62rem;padding:1px 5px;">🔥 ${activeUser.streak_days || 1}d</span>
          <span class="lang-caret">▼</span>
        `;
      } else {
        chipEl.innerHTML = `<span>👤 Sign In ▼</span>`;
      }
    });

    document.querySelectorAll(".user-dropdown-name").forEach(el => {
      if (activeUser) el.textContent = activeUser.display_name;
    });
    document.querySelectorAll(".user-dropdown-streak").forEach(el => {
      if (activeUser) el.textContent = `🔥 ${activeUser.streak_days || 1} Day Streak`;
    });
    document.querySelectorAll(".user-dropdown-reviews").forEach(el => {
      if (activeUser) el.textContent = `${activeUser.total_reviews || 0} Total Reviews`;
    });
  }

  function toggleUserDropdown() {
    document.querySelectorAll(".user-dropdown-menu").forEach(menu => {
      menu.classList.toggle("open");
    });
  }

  function closeUserDropdown() {
    document.querySelectorAll(".user-dropdown-menu").forEach(menu => {
      menu.classList.remove("open");
    });
  }

  /* -------------------------------------------------------------------------- */
  /* Language Selection Studio                                                  */
  /* -------------------------------------------------------------------------- */

  function renderLanguageChoices() {
    const grid = document.getElementById("languagesChoiceGrid");
    if (!grid) return;

    grid.innerHTML = availableLanguages.map(l => {
      const isCurrent = l.code === activeLang;
      const isGrc = l.code === "grc";
      const desc = isGrc
        ? "Classical & Hellenistic Greek of the Mediterranean Basin (Codex SBLGNT & Septuagint). Master case endings, verbal aspects, and authentic syntax without translation formulas."
        : "Classical Masoretic Hebrew of the Ancient Levant (Codex Leningradensis & BHSA). Acquire the 7 derived binyanim, pronominal suffixes, and authentic Tiberian cantillation.";

      return `
        <div class="language-master-card ${isCurrent ? 'active-lang' : ''}" onclick="window.App.launchLanguage('${l.code}')">
          <div>
            <div class="lang-autonym-hero">${l.autonym}</div>
            <div class="lang-name-en">${l.name}</div>
            <div class="lang-family-tag">${l.family} · ${l.script_direction.toUpperCase()}</div>
            <p class="lang-card-summary">${desc}</p>
          </div>
          <div>
            <div class="lang-stats-box">
              <span><strong>${l.concept_count}</strong> Curated Units</span>
              <span>Level <strong>A1 Foundation</strong></span>
              <span>4 Skills Active</span>
            </div>
            <button class="lang-launch-btn" onclick="event.stopPropagation(); window.App.launchLanguage('${l.code}')">
              Start Learning ${l.name} ➔
            </button>
          </div>
        </div>
      `;
    }).join("");
  }

  async function launchLanguage(langCode) {
    activeLang = langCode;
    localStorage.setItem("kaizen_active_lang", activeLang);
    if (activeUser) {
      activeUser.current_lang = activeLang;
      try {
        await invoke("set_user_language", { userId: activeUser.user_id, lang: activeLang });
      } catch (e) {
        console.warn("Failed to set user language via IPC:", e);
      }
    }
    updateLanguageDisplay();
    // Propagate language change across all studios and journey
    await window.Immersion.loadCatalog(activeLang);
    await window.Voice.onLanguageChanged(activeLang);
    await window.Reader.onLanguageChanged(activeLang);
    await window.Ledger.onLanguageChanged(activeLang);
    if (window.Journey) window.Journey.onLanguageChanged();

    // Directly open the Course Journey Roadmap (uncluttered progression view!)
    switchView("course");
  }

  /* -------------------------------------------------------------------------- */
  /* Studio Workspaces (Voice, Reader, Memory Ledger, Method Guide)             */
  /* -------------------------------------------------------------------------- */

  function openStudioWorkspace(workspaceId) {
    if (workspaceId === "immersion") {
      switchView("course");
      return;
    }
    switchView("studios");
    switchWorkspace(workspaceId);
  }

  function switchWorkspace(workspaceId) {
    if (workspaceId === "immersion") {
      switchView("course");
      return;
    }

    if (activeView !== "studios") {
      switchView("studios");
    }

    activeWorkspace = workspaceId;

    // Update tab buttons
    document.querySelectorAll(".tab-btn").forEach(btn => {
      btn.classList.toggle("active", btn.getAttribute("data-workspace") === workspaceId);
    });

    // Update panels
    document.querySelectorAll(".workspace-panel").forEach(panel => {
      panel.classList.toggle("active", panel.id === `panel_${workspaceId}`);
    });

    // Notify workspace controllers
    if (workspaceId === "voice") {
      window.Voice.onActivated();
    } else if (workspaceId === "ledger") {
      window.Ledger.onActivated();
    } else if (workspaceId === "reader") {
      window.Reader.onActivated();
    }
  }

  async function setLanguage(langCode) {
    await launchLanguage(langCode);
    closeLanguageModal();
  }

  function updateLanguageDisplay() {
    const langObj = availableLanguages.find(l => l.code === activeLang) || {
      code: activeLang,
      name: activeLang === "grc" ? "Koine Greek" : "Biblical Hebrew",
      autonym: activeLang === "grc" ? "Κοινή" : "עִבְרִית",
      direction: activeLang === "hbo" ? "rtl" : "ltr"
    };

    document.querySelectorAll(".active-lang-btn").forEach(btn => {
      btn.innerHTML = `
        <span class="lang-badge">${langObj.autonym}</span>
        <span>${langObj.name}</span>
        <span class="lang-caret">▼</span>
      `;
    });
  }

  function renderLanguageModal() {
    const listEl = document.getElementById("languageSelectionList");
    if (!listEl) return;

    listEl.innerHTML = availableLanguages.map(l => `
      <div class="language-card ${l.code === activeLang ? 'active-lang-card' : ''}" onclick="window.App.setLanguage('${l.code}')">
        <div style="display:flex; justify-content:space-between; align-items:baseline; margin-bottom:4px;">
          <span style="font-family:var(--font-target); font-size:1.35rem; font-weight:700; color:var(--lapis-cobalt);">${l.autonym}</span>
          <span style="font-family:var(--font-mono); font-size:0.72rem; color:var(--rubric-vermilion); font-weight:700;">${l.code.toUpperCase()}</span>
        </div>
        <div style="font-size:1.05rem; font-weight:700; color:var(--ink-primary); margin-bottom:2px;">${l.name}</div>
        <div style="font-size:0.78rem; color:var(--ink-secondary); margin-bottom:8px;">${l.family}</div>
        <div style="display:flex; justify-content:space-between; font-family:var(--font-mono); font-size:0.68rem; color:var(--ink-muted); border-top:1px dashed var(--rule-hairline); padding-top:6px;">
          <span>${l.concept_count} Units Verified</span>
          <span>v${l.semver}</span>
        </div>
      </div>
    `).join("");
  }

  function openLanguageModal() {
    const modal = document.getElementById("languageModal");
    if (modal) modal.classList.add("open");
  }

  function closeLanguageModal() {
    const modal = document.getElementById("languageModal");
    if (modal) modal.classList.remove("open");
  }

  function toggleTheme() {
    theme = theme === "vellum" ? "basalt" : "vellum";
    document.documentElement.setAttribute("data-theme", theme);
    document.querySelectorAll(".theme-toggle-btn").forEach(btn => {
      btn.textContent = theme === "vellum" ? "Atmosphere: Vellum" : "Atmosphere: Basalt";
    });
  }

  function togglePointing() {
    showPointing = !showPointing;
    document.querySelectorAll(".pointing-toggle-btn").forEach(btn => {
      btn.textContent = showPointing ? "Diacritics: On" : "Diacritics: Off";
    });
    window.Immersion.renderCurrentStimulus();
  }

  return {
    init,
    invoke,
    switchView,
    switchWorkspace,
    openStudioWorkspace,
    setLanguage,
    launchLanguage,
    openLanguageModal,
    closeLanguageModal,
    toggleTheme,
    togglePointing,
    // Profiles
    selectProfile,
    toggleCreateProfileForm,
    handleCreateProfileSubmit,
    setFormAvatar,
    signOut,
    toggleUserDropdown,
    closeUserDropdown,
    get activeLang() { return activeLang; },
    get activeWorkspace() { return activeWorkspace; },
    get activeUser() { return activeUser; },
    get showPointing() { return showPointing; },
    get theme() { return theme; },
  };
})();
