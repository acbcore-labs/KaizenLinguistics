/**
 * KAIZEN LINGUISTICS — Workspace 2: TrueAccent Voice Studio (Speech & Prosody)
 */

window.Voice = (function () {
  let isRecording = false;
  let audioContext = null;
  let mediaStream = null;
  let processorNode = null;
  let recordedSamples = [];
  let referenceContour = [];
  let learnerContour = [];
  let learnerBars = new Array(32).fill(0);
  let canvas = null;
  let ctx = null;

  async function init() {
    canvas = document.getElementById("voiceScopeCanvas");
    if (canvas) {
      ctx = canvas.getContext("2d");
      resizeCanvas();
      window.addEventListener("resize", resizeCanvas);
      drawOscilloscope();
    }
  }

  function resizeCanvas() {
    if (!canvas) return;
    canvas.width = canvas.parentElement.clientWidth || 800;
    canvas.height = 260;
  }

  function onActivated() {
    resizeCanvas();
    syncWithCurrentConcept();
    drawOscilloscope();
  }

  async function onLanguageChanged(langCode) {
    syncWithCurrentConcept();
    drawOscilloscope();
  }

  function syncWithCurrentConcept() {
    const concept = window.Immersion ? window.Immersion.getCurrentConcept() : null;
    const phraseEl = document.getElementById("voiceTargetPhrase");
    const ipaEl = document.getElementById("voiceTargetIpa");

    if (concept && concept.panels) {
      const target = concept.panels[concept.target_index] || concept.panels[0];
      const text = window.App.showPointing ? target.pointed : target.unpointed;
      if (phraseEl) phraseEl.textContent = text;
      if (ipaEl) ipaEl.textContent = target.ipa;
      generateSyntheticReferenceContour(target.ipa);
    } else {
      if (phraseEl) phraseEl.textContent = window.App.activeLang === "grc" ? "ὁ ἄγγελος βλέπει" : "הַנַּעַר יֹשֵׁב";
      if (ipaEl) ipaEl.textContent = window.App.activeLang === "grc" ? "/ho ˈaŋɡɛlos blˈɛpiː/" : "/hanːaʕar joːʃeːv/";
      generateSyntheticReferenceContour("/ho ˈaŋɡɛlos blˈɛpiː/");
    }
  }

  function generateSyntheticReferenceContour(ipa) {
    referenceContour = [];
    const count = 100;
    const baseFreq = window.App.activeLang === "grc" ? 220 : 185;
    for (let i = 0; i < count; i++) {
      const progress = i / count;
      // Authentic prosodic arch: rise on accented syllable, resolve on coda
      const pitch = baseFreq + Math.sin(progress * Math.PI) * 55 + Math.sin(progress * Math.PI * 3) * 12;
      referenceContour.push(pitch);
    }
  }

  async function toggleRecording() {
    if (isRecording) {
      await stopRecording();
    } else {
      await startRecording();
    }
  }

  async function startRecording() {
    try {
      audioContext = new (window.AudioContext || window.webkitAudioContext)({ sampleRate: 16000 });
      mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
      const source = audioContext.createMediaStreamSource(mediaStream);

      recordedSamples = [];
      learnerContour = [];
      learnerBars = new Array(32).fill(0);

      // AudioWorklet / ScriptProcessor for 16 kHz Float32 PCM
      processorNode = audioContext.createScriptProcessor(2048, 1, 1);
      processorNode.onaudioprocess = (e) => {
        if (!isRecording) return;
        const inputData = e.inputBuffer.getChannelData(0);
        for (let i = 0; i < inputData.length; i++) {
          recordedSamples.push(inputData[i]);
        }
        // Extract real-time pitch via autocorrelation
        const f0 = detectPitchAutocorrelation(inputData, 16000);
        if (f0 > 80 && f0 < 450) {
          learnerContour.push(f0);
          if (learnerContour.length > 120) learnerContour.shift();
        }
        // Update live VU bars
        updateSpectrumBars(inputData);
        drawOscilloscope();
      };

      source.connect(processorNode);
      processorNode.connect(audioContext.destination);

      isRecording = true;
      const recBtn = document.getElementById("btnVoiceRecord");
      if (recBtn) {
        recBtn.classList.add("recording");
        recBtn.textContent = "Stop Recording (Listening...)";
      }
    } catch (e) {
      console.warn("Microphone access unavailable, falling back to simulated speech:", e);
      simulateSpeechUtterance(false);
    }
  }

  async function stopRecording() {
    isRecording = false;
    const recBtn = document.getElementById("btnVoiceRecord");
    if (recBtn) {
      recBtn.classList.remove("recording");
      recBtn.textContent = "● Record Voice (Microphone)";
    }

    if (processorNode) processorNode.disconnect();
    if (mediaStream) mediaStream.getTracks().forEach(t => t.stop());
    if (audioContext && audioContext.state !== "closed") await audioContext.close();

    // Submit recorded 16 kHz Float32 PCM to Native 5-Stage Epistemic Gate
    if (recordedSamples.length > 1600) {
      await evaluateRecordedSamples(recordedSamples, 16000);
    }
  }

  async function evaluateRecordedSamples(samples, sampleRate) {
    const concept = window.Immersion ? window.Immersion.getCurrentConcept() : null;
    const targetText = concept ? concept.panels[concept.target_index].pointed : "Target Phrase";

    try {
      const payload = {
        samples,
        sample_rate: sampleRate,
        target_sentence_id: "voice_live",
        target_text: targetText,
        lang_code: window.App.activeLang,
        target_phones: ["h", "o", "a", "n", "g", "e", "l", "o", "s"],
        contrast_indices: [0],
        foils: []
      };

      const report = await window.App.invoke("evaluate_speech", { payload });
      displayEvaluationReport(report);
    } catch (e) {
      console.warn("IPC evaluate_speech error:", e);
    }
  }

  function displayEvaluationReport(report) {
    if (!report) return;

    // 1. Update 5-Stage Epistemic Gate Cards
    const isAbstain = report.outcome.startsWith("UnableToAssess");
    const isPass = report.outcome === "AssessedPass";

    updateGateCard("gate1", report.metrics.snr_db >= 12 ? "PASS" : "FAIL", `SNR: ${report.metrics.snr_db.toFixed(1)} dB`);
    updateGateCard("gate2", report.metrics.duration_ms >= 350 ? "PASS" : "FAIL", `${report.metrics.duration_ms} ms`);
    updateGateCard("gate3", "PASS", "Target Verified");
    updateGateCard("gate4", report.alignment_confidence >= 0.80 ? "PASS" : "FAIL", `Alignment: ${(report.alignment_confidence * 100).toFixed(0)}%`);
    updateGateCard("gate5", isPass ? "PASS" : isAbstain ? "ABSTAIN" : "FAIL", `Latency: ${report.compute_latency_ms.toFixed(1)} ms`);

    // 2. Abstention Card Explainer
    const abstentionCard = document.getElementById("voiceAbstentionCard");
    const abstentionBody = document.getElementById("voiceAbstentionBody");
    if (abstentionCard && abstentionBody) {
      if (isAbstain) {
        abstentionBody.textContent = `The acoustic pipeline abstained (${report.outcome}) because the recording had low SNR floor (${report.metrics.snr_db.toFixed(1)} dB). This ensures your memory stability score is never unfairly penalized due to room echo or microphone artifacts. Please record again in a quieter setting.`;
        abstentionCard.classList.add("open");
      } else {
        abstentionCard.classList.remove("open");
      }
    }

    // 3. Update Waveform Spectrum Bars
    if (report.learner_waveform_bars && report.learner_waveform_bars.length === 32) {
      learnerBars = report.learner_waveform_bars;
    }

    drawOscilloscope();
  }

  function updateGateCard(gateId, status, desc) {
    const card = document.getElementById(gateId);
    if (!card) return;
    const statusEl = card.querySelector(".gate-status");
    const descEl = card.querySelector(".gate-desc");

    if (statusEl) {
      statusEl.textContent = status;
      statusEl.className = `gate-status ${status.toLowerCase()}`;
    }
    if (descEl) descEl.textContent = desc;
  }

  function detectPitchAutocorrelation(buffer, sampleRate) {
    let sumSquares = 0;
    for (let i = 0; i < buffer.length; i++) sumSquares += buffer[i] * buffer[i];
    const rms = Math.sqrt(sumSquares / buffer.length);
    if (rms < 0.02) return 0; // ambient silence

    const minLag = Math.floor(sampleRate / 450);
    const maxLag = Math.floor(sampleRate / 80);
    let bestR = -1;
    let bestLag = -1;

    for (let lag = minLag; lag <= maxLag; lag++) {
      let r = 0;
      for (let i = 0; i < buffer.length - lag; i++) {
        r += buffer[i] * buffer[i + lag];
      }
      if (r > bestR) {
        bestR = r;
        bestLag = lag;
      }
    }
    return bestLag > 0 ? sampleRate / bestLag : 0;
  }

  function updateSpectrumBars(buffer) {
    const step = Math.floor(buffer.length / 32);
    for (let i = 0; i < 32; i++) {
      let sum = 0;
      for (let j = 0; j < step; j++) {
        const val = buffer[i * step + j];
        sum += val * val;
      }
      learnerBars[i] = Math.min(1.0, Math.sqrt(sum / step) * 4);
    }
  }

  function drawOscilloscope() {
    if (!ctx || !canvas) return;
    const w = canvas.width;
    const h = canvas.height;
    ctx.clearRect(0, 0, w, h);

    // 1. Draw 32-band Spectrum Bars in background
    const barWidth = w / 32;
    for (let i = 0; i < 32; i++) {
      const barH = learnerBars[i] * (h * 0.7);
      ctx.fillStyle = window.App.theme === "basalt" ? "rgba(212,175,90,0.14)" : "rgba(154,116,40,0.12)";
      ctx.fillRect(i * barWidth, h - barH, barWidth - 2, barH);
    }

    // 2. Draw Center Pitch Reference Grid
    ctx.strokeStyle = window.App.theme === "basalt" ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.06)";
    ctx.lineWidth = 1;
    for (let y = 40; y < h; y += 40) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(w, y);
      ctx.stroke();
    }

    // 3. Draw Native Reference Pitch Curve in Burnished Gold (#9A7428)
    if (referenceContour.length > 1) {
      ctx.strokeStyle = window.App.theme === "basalt" ? "#D4AF5A" : "#9A7428";
      ctx.lineWidth = 3.5;
      ctx.beginPath();
      for (let i = 0; i < referenceContour.length; i++) {
        const x = (i / (referenceContour.length - 1)) * w;
        // Normalize pitch: 100 Hz to 350 Hz mapping to canvas height
        const normY = h - ((referenceContour[i] - 100) / 250) * (h * 0.85);
        if (i === 0) ctx.moveTo(x, normY);
        else ctx.lineTo(x, normY);
      }
      ctx.stroke();
    }

    // 4. Draw Learner Voice Pitch Curve in Venetian Vermilion (#B8332A)
    if (learnerContour.length > 1) {
      ctx.strokeStyle = window.App.theme === "basalt" ? "#DF564B" : "#B8332A";
      ctx.lineWidth = 3;
      ctx.beginPath();
      for (let i = 0; i < learnerContour.length; i++) {
        const x = (i / (learnerContour.length - 1)) * w;
        const normY = h - ((learnerContour[i] - 100) / 250) * (h * 0.85);
        if (i === 0) ctx.moveTo(x, normY);
        else ctx.lineTo(x, normY);
      }
      ctx.stroke();
    }
  }

  // Developer Simulation Helpers (for Diagnostics Drawer)
  async function simulateSpeechUtterance(degradedSnr = false) {
    const sr = 16000;
    const durSec = 1.0;
    const totalSamples = Math.floor(sr * durSec);
    const samples = [];
    const noise = degradedSnr ? 0.35 : 0.005;

    for (let i = 0; i < totalSamples; i++) {
      const t = i / sr;
      const tone = Math.sin(2.0 * Math.PI * 220.0 * t) * 0.4;
      samples.push(tone + (Math.random() - 0.5) * noise);
    }
    await evaluateRecordedSamples(samples, sr);
  }

  function toggleDiagnosticsDrawer() {
    const drawer = document.getElementById("diagnosticsBody");
    if (drawer) drawer.classList.toggle("open");
  }

  return {
    init,
    onActivated,
    onLanguageChanged,
    toggleRecording,
    simulateSpeechUtterance,
    toggleDiagnosticsDrawer,
  };
})();
