import init, {
  detect_pitch,
  parse_musicxml,
  analyze_performance,
  generate_exercise,
  get_curriculum,
} from "./pkg/trumpet_rs.js";
import { state, resetPerformance } from "./state.js";
import { BUILTIN_PIECES, DEFAULT_BUILTIN_ID } from "./constants.js";
import { sizeCanvas, midiToName, getCurrentNote, roundRect } from "./renderers/base.js";
import { drawGameView } from "./renderers/game-view.js";
import { drawNotationView } from "./renderers/notation-view.js";
import { startMic, setDetectPitch } from "./audio.js";
import { loadBuiltinPiece, loadFileUpload, setParseMusicxml } from "./score-loader.js";

// DOM elements
const canvas = document.getElementById("score");
const ctx = canvas.getContext("2d");
const fileInput = document.getElementById("file-input");
const builtinSelect = document.getElementById("builtin-select");
const loadBuiltinBtn = document.getElementById("load-builtin-btn");
const playBtn = document.getElementById("play-btn");
const pauseBtn = document.getElementById("pause-btn");
const micBtn = document.getElementById("mic-btn");
const tempoEl = document.getElementById("tempo");
const targetNoteEl = document.getElementById("target-note");
const currentNoteEl = document.getElementById("current-note");
const freqEl = document.getElementById("freq");
const statusEl = document.getElementById("status");
const viewGameBtn = document.getElementById("view-game");
const viewNotationBtn = document.getElementById("view-notation");
const resultsPanel = document.getElementById("results-panel");
const pitchMeterCanvas = document.getElementById("pitch-meter");
const pitchMeterCtx = pitchMeterCanvas ? pitchMeterCanvas.getContext("2d") : null;

// Mode tabs
const modePlayAlongBtn = document.getElementById("mode-play-along");
const modePracticeBtn = document.getElementById("mode-practice");
const panelPlayAlong = document.getElementById("panel-play-along");
const panelPractice = document.getElementById("panel-practice");
const curriculumTree = document.getElementById("curriculum-tree");

const PITCH_TRAIL_MAX = 300;
const PROGRESS_KEY = "trumpet_trainer_progress";

// Progress tracking
function loadProgress() {
  try {
    return JSON.parse(localStorage.getItem(PROGRESS_KEY) || "{}");
  } catch {
    return {};
  }
}

function saveProgress(exerciseKey, score) {
  const progress = loadProgress();
  const prev = progress[exerciseKey] || 0;
  if (score > prev) {
    progress[exerciseKey] = score;
    localStorage.setItem(PROGRESS_KEY, JSON.stringify(progress));
  }
  return progress;
}

function getStagePassCount(stageExercises, progress) {
  let count = 0;
  for (const ex of stageExercises) {
    for (const key of ex.keys) {
      const progressKey = `${ex.exercise_type}_${key}`;
      if ((progress[progressKey] || 0) >= 80) {
        count++;
        break; // Count exercise as passed if any key >= 80
      }
    }
  }
  return count;
}

function isStageUnlocked(stageIndex, curriculum) {
  if (stageIndex === 0) return true;
  const progress = loadProgress();
  const prevStage = curriculum[stageIndex - 1];
  return getStagePassCount(prevStage.exercises, progress) >= 3;
}

// Mode switching
function setActiveMode(mode) {
  state.mode = mode;
  if (modePlayAlongBtn) modePlayAlongBtn.classList.toggle("active", mode === "play_along");
  if (modePracticeBtn) modePracticeBtn.classList.toggle("active", mode === "practice");
  if (panelPlayAlong) panelPlayAlong.style.display = mode === "play_along" ? "" : "none";
  if (panelPractice) panelPractice.style.display = mode === "practice" ? "" : "none";
}

// Curriculum rendering
let curriculumData = null;
let currentExerciseKey = null;

function renderCurriculumTree(curriculum) {
  if (!curriculumTree) return;
  const progress = loadProgress();
  let html = "";

  curriculum.forEach((stage, stageIdx) => {
    const unlocked = isStageUnlocked(stageIdx, curriculum);
    const passCount = getStagePassCount(stage.exercises, progress);
    const lockLabel = unlocked
      ? `${passCount}/${stage.exercises.length}`
      : "Locked";
    const lockClass = unlocked ? "unlocked" : "";
    const collapsed = !unlocked ? "collapsed" : "";

    html += `<div class="curriculum-stage" data-stage="${stageIdx}">
      <div class="stage-header" data-stage-toggle="${stageIdx}">
        <h4>Stage ${stage.stage}: ${stage.name}</h4>
        <span class="stage-lock ${lockClass}">${lockLabel}</span>
      </div>
      <div class="stage-exercises ${collapsed}" id="stage-exercises-${stageIdx}">`;

    if (unlocked) {
      stage.exercises.forEach((ex, exIdx) => {
        const tempoMin = ex.tempo_range[0];
        const tempoMax = ex.tempo_range[1];
        const defaultTempo = Math.round((tempoMin + tempoMax) / 2);

        // Key selector
        let keyOptions = ex.keys
          .map((k, i) => `<option value="${k}" ${i === 0 ? "selected" : ""}>${k}</option>`)
          .join("");

        // Best score
        const bestKey = `${ex.exercise_type}_${ex.keys[0]}`;
        const best = progress[bestKey] || 0;
        const bestClass = best >= 80 ? "passed" : "";

        html += `<div class="exercise-item" data-ex-type="${ex.exercise_type}" data-difficulty="${ex.difficulty}" data-midi-low="${ex.midi_range[0]}" data-midi-high="${ex.midi_range[1]}">
          <div class="ex-name">${ex.name}</div>
          <div class="ex-desc">${ex.description}</div>
          <div class="ex-controls">
            <select class="ex-key" data-ex-idx="${stageIdx}-${exIdx}">${keyOptions}</select>
            <div class="tempo-slider">
              <input type="range" class="ex-tempo" min="${tempoMin}" max="${tempoMax}" value="${defaultTempo}" data-ex-idx="${stageIdx}-${exIdx}" />
              <span class="ex-tempo-label">${defaultTempo}</span>
            </div>
            <button class="ex-generate" data-ex-idx="${stageIdx}-${exIdx}">Go</button>
            <span class="ex-best ${bestClass}">${best > 0 ? `Best: ${best}` : ""}</span>
          </div>
        </div>`;
      });
    } else {
      html += `<p style="font-size: 12px; color: var(--muted); padding: 8px 0;">Score 80+ on 3 exercises in Stage ${stage.stage - 1} to unlock.</p>`;
    }

    html += `</div></div>`;
  });

  curriculumTree.innerHTML = html;

  // Event listeners for stage toggles
  curriculumTree.querySelectorAll("[data-stage-toggle]").forEach((el) => {
    el.addEventListener("click", () => {
      const idx = el.dataset.stageToggle;
      const exercises = document.getElementById(`stage-exercises-${idx}`);
      if (exercises) exercises.classList.toggle("collapsed");
    });
  });

  // Tempo slider labels
  curriculumTree.querySelectorAll(".ex-tempo").forEach((slider) => {
    slider.addEventListener("input", (e) => {
      const label = e.target.parentElement.querySelector(".ex-tempo-label");
      if (label) label.textContent = e.target.value;
    });
  });

  // Generate buttons
  curriculumTree.querySelectorAll(".ex-generate").forEach((btn) => {
    btn.addEventListener("click", (e) => {
      const item = e.target.closest(".exercise-item");
      if (!item) return;

      const exType = item.dataset.exType;
      const difficulty = parseInt(item.dataset.difficulty);
      const midiLow = parseInt(item.dataset.midiLow);
      const midiHigh = parseInt(item.dataset.midiHigh);
      const keySelect = item.querySelector(".ex-key");
      const tempoSlider = item.querySelector(".ex-tempo");
      const key = keySelect ? keySelect.value : "C4";
      const tempo = tempoSlider ? parseFloat(tempoSlider.value) : 100;

      currentExerciseKey = `${exType}_${key}`;

      try {
        const score = generate_exercise(exType, key, tempo, difficulty, midiLow, midiHigh);
        state.playing = false;
        state.lastBeat = 0;
        state.score = score;
        state.totalBeats = score.total_beats || 0;
        state.secondsPerBeat = 60 / score.tempo;
        tempoEl.textContent = `${score.tempo.toFixed(0)} bpm`;
        resetPerformance();
        if (resultsPanel) resultsPanel.classList.remove("visible");
        setStatus(`Exercise loaded. Press Play.`);
        draw(0);
        updateTargetDisplay(0);
      } catch (err) {
        setStatus(`Exercise error: ${err}`);
      }
    });
  });
}

function setStatus(text) {
  statusEl.textContent = text;
}

function draw(beat = 0) {
  if (state.currentView === "game") {
    drawGameView(canvas, ctx, state.score, beat, state.currentPitch, state.pitchTrail);
  } else {
    drawNotationView(canvas, ctx, state.score, beat, state.totalBeats);
  }
}

function updateTargetDisplay(beat) {
  const note = getCurrentNote(state.score, beat);
  targetNoteEl.textContent = note ? midiToName(note.midi) : "--";
}

function updatePitchDisplay() {
  if (state.currentPitch && state.currentPitch.hz > 0) {
    freqEl.textContent = `${state.currentPitch.hz.toFixed(1)} Hz`;
    const midi = Math.round(state.currentPitch.midi_float);
    currentNoteEl.textContent = midiToName(midi);
  } else {
    freqEl.textContent = "--";
    currentNoteEl.textContent = "--";
  }
}

function drawPitchMeter(beat) {
  if (!pitchMeterCtx) return;
  const w = pitchMeterCanvas.width;
  const h = pitchMeterCanvas.height;
  pitchMeterCtx.clearRect(0, 0, w, h);

  // Draw gauge background: green center, red edges
  const grad = pitchMeterCtx.createLinearGradient(0, 0, w, 0);
  grad.addColorStop(0, "#d94040");
  grad.addColorStop(0.3, "#e4674b");
  grad.addColorStop(0.45, "#3daa5f");
  grad.addColorStop(0.55, "#3daa5f");
  grad.addColorStop(0.7, "#e4674b");
  grad.addColorStop(1, "#d94040");
  pitchMeterCtx.fillStyle = grad;
  pitchMeterCtx.globalAlpha = 0.15;
  roundRect(pitchMeterCtx, 0, h / 2 - 6, w, 12, 6);
  pitchMeterCtx.fill();
  pitchMeterCtx.globalAlpha = 1.0;

  // Center tick
  pitchMeterCtx.strokeStyle = "rgba(0,0,0,0.2)";
  pitchMeterCtx.lineWidth = 1;
  pitchMeterCtx.beginPath();
  pitchMeterCtx.moveTo(w / 2, h / 2 - 10);
  pitchMeterCtx.lineTo(w / 2, h / 2 + 10);
  pitchMeterCtx.stroke();

  // Scale labels
  pitchMeterCtx.fillStyle = "rgba(0,0,0,0.3)";
  pitchMeterCtx.font = "9px Space Grotesk, sans-serif";
  pitchMeterCtx.fillText("-50c", 2, h - 2);
  pitchMeterCtx.fillText("+50c", w - 26, h - 2);
  pitchMeterCtx.fillText("0", w / 2 - 3, h - 2);

  // Needle: show pitch deviation from target
  const target = getCurrentNote(state.score, beat);
  if (!target || !state.currentPitch || state.currentPitch.hz <= 0 || state.currentPitch.confidence < 0.5) {
    return;
  }

  const centsOff = (state.currentPitch.midi_float - target.midi) * 100;
  const clampedCents = Math.max(-50, Math.min(50, centsOff));
  const needleX = w / 2 + (clampedCents / 50) * (w / 2);

  // Needle color: green when close, orange/red when far
  const absCents = Math.abs(clampedCents);
  let needleColor;
  if (absCents < 10) needleColor = "#3daa5f";
  else if (absCents < 25) needleColor = "#e4674b";
  else needleColor = "#d94040";

  pitchMeterCtx.beginPath();
  pitchMeterCtx.arc(needleX, h / 2, 7, 0, Math.PI * 2);
  pitchMeterCtx.fillStyle = needleColor;
  pitchMeterCtx.fill();

  // Glow
  pitchMeterCtx.beginPath();
  pitchMeterCtx.arc(needleX, h / 2, 10, 0, Math.PI * 2);
  pitchMeterCtx.globalAlpha = 0.3;
  pitchMeterCtx.strokeStyle = needleColor;
  pitchMeterCtx.lineWidth = 2;
  pitchMeterCtx.stroke();
  pitchMeterCtx.globalAlpha = 1.0;
}

// Note onset detection: detect when the player starts a new note
function trackPerformance(beat) {
  if (!state.micActive || !state.currentPitch) return;

  const p = state.currentPitch;
  if (!p || p.hz <= 0 || p.confidence < 0.5) {
    // Silence - reset onset tracking
    state.lastDetectedMidi = null;
    return;
  }

  const currentMidi = Math.round(p.midi_float);

  // Record pitch trail for visualization
  state.pitchTrail.push({ beat, midi_float: p.midi_float });
  if (state.pitchTrail.length > PITCH_TRAIL_MAX) {
    state.pitchTrail.shift();
  }

  // Detect note onset (new note or first note after silence)
  if (currentMidi !== state.lastDetectedMidi) {
    state.playedNotes.push({
      onset_beat: beat,
      midi_float: p.midi_float,
      midi_rounded: currentMidi,
      confidence: p.confidence,
    });
    state.lastDetectedMidi = currentMidi;
  }
}

function renderTechniqueMetrics(analysis) {
  let html = "";
  const metrics = [
    { label: "Pitch Stability", value: analysis.pitch_stability, invert: true },
    { label: "Attack Quality", value: analysis.attack_quality },
    { label: "Breath Support", value: analysis.breath_support },
  ];

  const hasAny = metrics.some((m) => m.value != null);
  if (!hasAny) return "";

  html += `<div class="results-technique"><h4>Technique</h4>`;
  for (const m of metrics) {
    if (m.value == null) continue;
    // For pitch stability, lower is better; convert to 0-1 where 1 is best
    let pct;
    if (m.invert) {
      pct = Math.max(0, Math.min(100, (1 - m.value / 30) * 100));
    } else {
      pct = Math.max(0, Math.min(100, m.value * 100));
    }
    const color = pct >= 70 ? "#3daa5f" : pct >= 40 ? "#e4674b" : "#d94040";
    html += `<div class="technique-bar">
      <span class="tech-label">${m.label}</span>
      <div class="tech-track"><div class="tech-fill" style="width: ${pct}%; background: ${color};"></div></div>
      <span class="tech-value" style="color: ${color};">${Math.round(pct)}%</span>
    </div>`;
  }

  if (analysis.endurance_delta != null && analysis.endurance_delta > 5) {
    html += `<div class="technique-bar">
      <span class="tech-label">Endurance</span>
      <div class="tech-track"><div class="tech-fill" style="width: ${Math.max(0, 100 - analysis.endurance_delta)}%; background: #e4674b;"></div></div>
      <span class="tech-value" style="color: #e4674b;">-${Math.round(analysis.endurance_delta)}%</span>
    </div>`;
  }

  html += `</div>`;
  return html;
}

function showResults() {
  if (!resultsPanel || !state.score || state.playedNotes.length === 0) return;

  try {
    // Pass pitch trail for technique analysis
    const pitchTrail = state.pitchTrail.length > 0 ? state.pitchTrail : null;
    const analysis = analyze_performance(state.score, state.playedNotes, 50.0, 0.3, pitchTrail);
    state.analysisResult = analysis;

    // Save progress in practice mode
    if (state.mode === "practice" && currentExerciseKey) {
      const progress = saveProgress(currentExerciseKey, Math.round(analysis.overall_score));
      // Re-render curriculum tree with updated progress
      if (curriculumData) renderCurriculumTree(curriculumData);
    }

    const scoreColor =
      analysis.overall_score >= 70 ? "#3daa5f" : analysis.overall_score >= 40 ? "#e4674b" : "#d94040";

    const techniqueHtml = renderTechniqueMetrics(analysis);

    let techniqueFeedbackHtml = "";
    if (analysis.technique_feedback && analysis.technique_feedback.length > 0) {
      techniqueFeedbackHtml = analysis.technique_feedback.map((msg) => `<p>${msg}</p>`).join("");
    }

    let html = `<div class="results-card">
      <div class="results-header">
        <div class="results-score" style="color: ${scoreColor}">
          ${Math.round(analysis.overall_score)}
        </div>
        <div class="results-label">Score</div>
      </div>
      <div class="results-stats">
        <div class="stat"><span class="stat-num">${analysis.notes_correct}</span><span class="stat-label">Correct</span></div>
        <div class="stat"><span class="stat-num">${analysis.notes_wrong_pitch}</span><span class="stat-label">Wrong pitch</span></div>
        <div class="stat"><span class="stat-num">${analysis.notes_missed}</span><span class="stat-label">Missed</span></div>
      </div>
      <div class="results-meters">
        <div class="meter">
          <span class="meter-label">Pitch</span>
          <span class="meter-value ${analysis.pitch_tendency}">${analysis.pitch_tendency} (${analysis.avg_pitch_error_cents >= 0 ? "+" : ""}${analysis.avg_pitch_error_cents.toFixed(0)}c)</span>
        </div>
        <div class="meter">
          <span class="meter-label">Timing</span>
          <span class="meter-value ${analysis.timing_tendency}">${analysis.timing_tendency.replace("_", " ")} (${analysis.avg_timing_error_beats >= 0 ? "+" : ""}${analysis.avg_timing_error_beats.toFixed(2)} beats)</span>
        </div>
      </div>
      ${techniqueHtml}
      <div class="results-feedback">
        ${analysis.feedback.map((msg) => `<p>${msg}</p>`).join("")}
        ${techniqueFeedbackHtml}
      </div>
      <button id="results-close" onclick="document.getElementById('results-panel').classList.remove('visible')">Try Again</button>
    </div>`;

    resultsPanel.innerHTML = html;
    resultsPanel.classList.add("visible");
  } catch (err) {
    console.error("Analysis error:", err);
  }
}

function tick(now) {
  if (!state.playing || !state.score) return;
  const elapsed = (now - state.playStart) / 1000;
  const beat = elapsed / state.secondsPerBeat;
  state.lastBeat = beat;

  if (beat > state.totalBeats) {
    state.playing = false;
    setStatus("Finished. Press Play to restart.");
    draw(state.totalBeats);
    updateTargetDisplay(0);
    showResults();
    return;
  }

  trackPerformance(beat);
  draw(beat);
  updateTargetDisplay(beat);
  updatePitchDisplay();
  drawPitchMeter(beat);
  requestAnimationFrame(tick);
}

function startPlayback() {
  if (!state.score) {
    setStatus("Load a MusicXML file before playing.");
    return;
  }
  resetPerformance();
  if (resultsPanel) resultsPanel.classList.remove("visible");
  state.playing = true;
  state.playStart = performance.now();
  setStatus("Playing.");
  requestAnimationFrame(tick);
}

function pausePlayback() {
  state.playing = false;
  setStatus("Paused.");
  draw(state.lastBeat);
  updateTargetDisplay(state.lastBeat);
}

function populateBuiltinSelector() {
  builtinSelect.innerHTML = "";
  for (const piece of BUILTIN_PIECES) {
    const option = document.createElement("option");
    option.value = piece.id;
    option.textContent = piece.label;
    builtinSelect.appendChild(option);
  }
}

async function handleScoreLoad(result, sourceName) {
  if (result.ok) {
    tempoEl.textContent = `${result.tempo.toFixed(0)} bpm`;
    setStatus(`${sourceName} loaded. Press Play.`);
    resetPerformance();
    if (resultsPanel) resultsPanel.classList.remove("visible");
    draw(0);
    updateTargetDisplay(0);
  } else {
    setStatus(result.error);
  }
  return result.ok;
}

function setActiveView(view) {
  state.currentView = view;
  if (viewGameBtn) viewGameBtn.classList.toggle("active", view === "game");
  if (viewNotationBtn) viewNotationBtn.classList.toggle("active", view === "notation");
  draw(state.lastBeat);
}

// Event listeners
fileInput.addEventListener("change", async (event) => {
  const file = event.target.files[0];
  if (!file) return;
  setStatus(`Loading file: ${file.name}...`);
  const result = await loadFileUpload(file);
  await handleScoreLoad(result, file.name);
});

loadBuiltinBtn.addEventListener("click", async () => {
  const result = await loadBuiltinPiece(builtinSelect.value);
  const piece = BUILTIN_PIECES.find((c) => c.id === builtinSelect.value);
  await handleScoreLoad(result, piece ? piece.label : "piece");
});

playBtn.addEventListener("click", startPlayback);
pauseBtn.addEventListener("click", pausePlayback);

micBtn.addEventListener("click", async () => {
  await startMic(setStatus);
  if (state.micActive) {
    micBtn.textContent = "Mic Active";
  }
});

if (viewGameBtn) {
  viewGameBtn.addEventListener("click", () => setActiveView("game"));
}
if (viewNotationBtn) {
  viewNotationBtn.addEventListener("click", () => setActiveView("notation"));
}

// Mode tab listeners
if (modePlayAlongBtn) {
  modePlayAlongBtn.addEventListener("click", () => setActiveMode("play_along"));
}
if (modePracticeBtn) {
  modePracticeBtn.addEventListener("click", () => {
    setActiveMode("practice");
    if (curriculumData && curriculumTree) {
      renderCurriculumTree(curriculumData);
    }
  });
}

window.addEventListener("resize", () => {
  sizeCanvas(canvas);
  draw(state.lastBeat);
});

// Boot
async function boot() {
  sizeCanvas(canvas);
  setStatus("Loading WASM...");
  await init();
  state.wasmReady = true;

  // Wire up WASM functions to modules
  setDetectPitch(detect_pitch);
  setParseMusicxml(parse_musicxml);

  // Load curriculum
  try {
    curriculumData = get_curriculum();
  } catch (err) {
    console.error("Failed to load curriculum:", err);
  }

  populateBuiltinSelector();
  builtinSelect.value = DEFAULT_BUILTIN_ID;

  const result = await loadBuiltinPiece(DEFAULT_BUILTIN_ID);
  const loaded = await handleScoreLoad(result, "Happy Birthday");
  if (!loaded) {
    setStatus("Ready. Choose a built-in piece or load a MusicXML file.");
    draw(0);
  }
}

boot();
