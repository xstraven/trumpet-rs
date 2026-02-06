export const state = {
  wasmReady: false,
  score: null,
  totalBeats: 0,
  secondsPerBeat: 0.5,
  playing: false,
  playStart: 0,
  lastBeat: 0,
  currentView: "game", // "game" or "notation"

  // Audio
  audioCtx: null,
  analyser: null,
  micActive: false,
  micBuffer: null,

  // Current pitch detection result
  currentPitch: null, // {hz, confidence, midi_float} or null

  // Performance tracking
  playedNotes: [], // Array of {onset_beat, midi_float, midi_rounded, confidence}
  lastDetectedMidi: null,
  pitchTrail: [], // Array of {beat, midi_float} for visualization

  // Analysis result
  analysisResult: null,
};

export function resetPerformance() {
  state.playedNotes = [];
  state.lastDetectedMidi = null;
  state.analysisResult = null;
  state.pitchTrail = [];
}
