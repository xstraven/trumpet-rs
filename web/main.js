import init, { detect_pitch, parse_musicxml } from "./pkg/trumpet_rs.js";

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

let wasmReady = false;
let score = null;
let totalBeats = 0;
let secondsPerBeat = 0.5;
let playStart = 0;
let playing = false;
let lastBeat = 0;

let audioCtx = null;
let analyser = null;
let micActive = false;
let micBuffer = null;

const DEFAULT_BUILTIN_ID = "happy-birthday";
const BUILTIN_PIECES = [
  {
    id: "happy-birthday",
    label: "Happy Birthday",
    path: "./assets/happy_birthday.musicxml",
  },
  {
    id: "hot-cross-buns",
    label: "Hot Cross Buns",
    path: "./assets/hot_cross_buns.musicxml",
  },
  {
    id: "ode-to-joy",
    label: "Ode to Joy",
    path: "./assets/ode_to_joy.musicxml",
  },
];

const layout = {
  leftPad: 40,
  rightPad: 40,
  topPad: 40,
  lineSpacing: 14,
  noteHeight: 10,
  semitoneStep: 4,
  refMidi: 71,
};

function setStatus(text) {
  statusEl.textContent = text;
}

function sizeCanvas() {
  const rect = canvas.getBoundingClientRect();
  const ratio = window.devicePixelRatio || 1;
  canvas.width = rect.width * ratio;
  canvas.height = rect.height * ratio;
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.scale(ratio, ratio);
}

function beatToX(beat) {
  const rect = canvas.getBoundingClientRect();
  const usable = rect.width - layout.leftPad - layout.rightPad;
  if (totalBeats <= 0) return layout.leftPad;
  return layout.leftPad + (beat / totalBeats) * usable;
}

function midiToY(midi) {
  const rect = canvas.getBoundingClientRect();
  const staffCenter = rect.height / 2;
  return staffCenter - (midi - layout.refMidi) * layout.semitoneStep;
}

function midiToName(midi) {
  const names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
  const name = names[(midi + 1200) % 12];
  const octave = Math.floor(midi / 12) - 1;
  return `${name}${octave}`;
}

function drawScore(currentBeat = 0) {
  const rect = canvas.getBoundingClientRect();
  ctx.clearRect(0, 0, rect.width, rect.height);

  const staffTop = rect.height / 2 - layout.lineSpacing * 2;
  ctx.strokeStyle = "#c9bfb3";
  ctx.lineWidth = 1.2;
  for (let i = 0; i < 5; i += 1) {
    const y = staffTop + i * layout.lineSpacing;
    ctx.beginPath();
    ctx.moveTo(layout.leftPad, y);
    ctx.lineTo(rect.width - layout.rightPad, y);
    ctx.stroke();
  }

  if (!score) {
    return;
  }

  const currentNote = getCurrentNote(currentBeat);

  for (const note of score.notes) {
    const x = beatToX(note.start_beat);
    const width = Math.max(6, beatToX(note.start_beat + note.duration_beats) - x);

    if (note.is_rest) {
      ctx.fillStyle = "#9a8f84";
      const restY = rect.height / 2 - layout.noteHeight / 2;
      ctx.fillRect(x, restY, width, layout.noteHeight * 0.8);
      continue;
    }

    const y = midiToY(note.midi) - layout.noteHeight / 2;
    ctx.fillStyle = currentNote && currentNote === note ? "#e4674b" : "#2e8e8b";
    roundRect(ctx, x, y, width, layout.noteHeight, 6);
    ctx.fill();

    ctx.fillStyle = "#2f2a26";
    ctx.font = "12px Space Grotesk, sans-serif";
    ctx.fillText(midiToName(note.midi), x + 4, y - 6);
  }

  const playheadX = beatToX(currentBeat);
  ctx.strokeStyle = "#181411";
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(playheadX, layout.topPad / 2);
  ctx.lineTo(playheadX, rect.height - layout.topPad / 2);
  ctx.stroke();
}

function roundRect(context, x, y, width, height, radius) {
  const r = Math.min(radius, width / 2, height / 2);
  context.beginPath();
  context.moveTo(x + r, y);
  context.lineTo(x + width - r, y);
  context.quadraticCurveTo(x + width, y, x + width, y + r);
  context.lineTo(x + width, y + height - r);
  context.quadraticCurveTo(x + width, y + height, x + width - r, y + height);
  context.lineTo(x + r, y + height);
  context.quadraticCurveTo(x, y + height, x, y + height - r);
  context.lineTo(x, y + r);
  context.quadraticCurveTo(x, y, x + r, y);
  context.closePath();
}

function getCurrentNote(beat) {
  if (!score) return null;
  for (const note of score.notes) {
    if (note.is_rest) continue;
    if (beat >= note.start_beat && beat < note.start_beat + note.duration_beats) {
      return note;
    }
  }
  return null;
}

function updateTargetDisplay(beat) {
  const note = getCurrentNote(beat);
  if (!note) {
    targetNoteEl.textContent = "--";
    return;
  }
  targetNoteEl.textContent = midiToName(note.midi);
}

function tick(now) {
  if (!playing || !score) return;
  const elapsed = (now - playStart) / 1000;
  const beat = elapsed / secondsPerBeat;
  lastBeat = beat;

  if (beat > totalBeats) {
    playing = false;
    setStatus("Finished. Press Play to restart.");
    drawScore(0);
    updateTargetDisplay(0);
    return;
  }

  drawScore(beat);
  updateTargetDisplay(beat);
  requestAnimationFrame(tick);
}

function startPlayback() {
  if (!score) {
    setStatus("Load a MusicXML file before playing.");
    return;
  }
  playing = true;
  playStart = performance.now();
  setStatus("Playing.");
  requestAnimationFrame(tick);
}

function pausePlayback() {
  playing = false;
  setStatus("Paused.");
  drawScore(lastBeat);
  updateTargetDisplay(lastBeat);
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

async function parseAndLoad(xml, sourceName = "MusicXML") {
  if (!wasmReady) {
    setStatus("WASM not ready yet.");
    return false;
  }
  try {
    const parsed = parse_musicxml(xml);
    playing = false;
    lastBeat = 0;
    score = parsed;
    totalBeats = 0;
    for (const note of score.notes) {
      totalBeats = Math.max(totalBeats, note.start_beat + note.duration_beats);
    }
    secondsPerBeat = 60 / score.tempo;
    tempoEl.textContent = `${score.tempo.toFixed(0)} bpm`;
    setStatus(`${sourceName} loaded. Press Play.`);
    drawScore(0);
    updateTargetDisplay(0);
    return true;
  } catch (err) {
    setStatus(`Error parsing ${sourceName}: ${err}`);
    return false;
  }
}

async function loadBuiltinPiece(pieceId) {
  const piece = BUILTIN_PIECES.find((candidate) => candidate.id === pieceId);
  if (!piece) {
    setStatus("Unknown built-in piece selected.");
    return false;
  }

  setStatus(`Loading built-in piece: ${piece.label}...`);
  try {
    const response = await fetch(piece.path);
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    const xml = await response.text();
    return await parseAndLoad(xml, piece.label);
  } catch (err) {
    setStatus(`Error loading built-in piece "${piece.label}": ${err}`);
    return false;
  }
}

async function startMic() {
  if (!wasmReady) {
    setStatus("WASM not ready yet.");
    return;
  }
  if (micActive) {
    return;
  }

  try {
    audioCtx = new AudioContext();
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    const source = audioCtx.createMediaStreamSource(stream);
    analyser = audioCtx.createAnalyser();
    analyser.fftSize = 2048;
    micBuffer = new Float32Array(analyser.fftSize);
    source.connect(analyser);
    micActive = true;
    micBtn.textContent = "Mic Active";
    setStatus("Mic active. Play a note.");
    requestAnimationFrame(updateMic);
  } catch (err) {
    setStatus(`Mic error: ${err}`);
  }
}

function updateMic() {
  if (!micActive || !analyser) return;
  analyser.getFloatTimeDomainData(micBuffer);
  const freq = detect_pitch(micBuffer, audioCtx.sampleRate);

  if (freq > 0) {
    freqEl.textContent = `${freq.toFixed(1)} Hz`;
    const midi = Math.round(69 + 12 * Math.log2(freq / 440));
    currentNoteEl.textContent = midiToName(midi);
  } else {
    freqEl.textContent = "--";
    currentNoteEl.textContent = "--";
  }

  requestAnimationFrame(updateMic);
}

fileInput.addEventListener("change", async (event) => {
  const file = event.target.files[0];
  if (!file) return;
  setStatus(`Loading file: ${file.name}...`);
  const xml = await file.text();
  await parseAndLoad(xml, file.name);
});

loadBuiltinBtn.addEventListener("click", async () => {
  await loadBuiltinPiece(builtinSelect.value);
});

playBtn.addEventListener("click", startPlayback);

pauseBtn.addEventListener("click", pausePlayback);

micBtn.addEventListener("click", startMic);

window.addEventListener("resize", () => {
  sizeCanvas();
  drawScore(lastBeat);
});

async function boot() {
  sizeCanvas();
  setStatus("Loading WASM...");
  await init();
  wasmReady = true;
  populateBuiltinSelector();
  builtinSelect.value = DEFAULT_BUILTIN_ID;
  const loaded = await loadBuiltinPiece(DEFAULT_BUILTIN_ID);
  if (!loaded) {
    setStatus("Ready. Choose a built-in piece or load a MusicXML file.");
    drawScore(0);
  }
}

boot();
