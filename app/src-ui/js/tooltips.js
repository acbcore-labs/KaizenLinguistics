/**
 * KAIZEN LINGUISTICS — Tooltips, Guidance Banners & Keyboard Shortcuts Engine
 */

window.Tooltips = (function () {
  function init() {
    // Global keyboard listener
    window.addEventListener("keydown", handleKeydown);

    // Guidance drawer toggles
    document.querySelectorAll(".guidance-toggle-btn").forEach(btn => {
      btn.addEventListener("click", () => {
        const targetId = btn.getAttribute("data-target");
        const drawer = document.getElementById(targetId);
        if (drawer) {
          drawer.classList.toggle("open");
          btn.textContent = drawer.classList.contains("open") ? "Hide Guidance ▲" : "What is this? ▼";
        }
      });
    });
  }

  function handleKeydown(e) {
    // If typing in input, ignore shortcuts
    if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA") return;

    if (e.key === "?") {
      toggleShortcutsModal();
    } else if (e.key === "1") {
      window.App.switchWorkspace("immersion");
    } else if (e.key === "2") {
      window.App.switchWorkspace("voice");
    } else if (e.key === "3") {
      window.App.switchWorkspace("reader");
    } else if (e.key === "4") {
      window.App.switchWorkspace("ledger");
    } else if (e.key === "5") {
      window.App.switchWorkspace("method");
    } else if (e.key === " " || e.code === "Space") {
      e.preventDefault();
      if (window.App.activeWorkspace === "immersion") {
        window.Immersion.playAudio();
      } else if (window.App.activeWorkspace === "voice") {
        window.Voice.toggleRecording();
      }
    } else if (e.key === "r" || e.key === "R") {
      if (window.App.activeWorkspace === "voice" || window.App.activeWorkspace === "immersion") {
        window.Voice.toggleRecording();
      }
    } else if (e.key === "[" || e.key === "ArrowLeft") {
      if (window.App.activeWorkspace === "immersion") {
        window.Immersion.stepConcept(-1);
      }
    } else if (e.key === "]" || e.key === "ArrowRight") {
      if (window.App.activeWorkspace === "immersion") {
        window.Immersion.stepConcept(1);
      }
    } else if (e.key === "t" || e.key === "T") {
      window.App.toggleTheme();
    } else if (e.key === "p" || e.key === "P") {
      window.App.togglePointing();
    } else if (e.key === "Escape") {
      closeAllModals();
    }
  }

  function toggleShortcutsModal() {
    const modal = document.getElementById("shortcutsModal");
    if (modal) {
      modal.classList.toggle("open");
    }
  }

  function closeAllModals() {
    document.querySelectorAll(".modal-overlay").forEach(m => m.classList.remove("open"));
  }

  return {
    init,
    toggleShortcutsModal,
    closeAllModals,
  };
})();
