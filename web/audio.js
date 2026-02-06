import { state } from "./state.js";

let detectPitchFn = null;

export function setDetectPitch(fn) {
  detectPitchFn = fn;
}

export async function startMic(onStatusChange) {
  if (!state.wasmReady) {
    onStatusChange("WASM not ready yet.");
    return;
  }
  if (state.micActive) return;

  try {
    state.audioCtx = new AudioContext();
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    const source = state.audioCtx.createMediaStreamSource(stream);
    state.analyser = state.audioCtx.createAnalyser();
    state.analyser.fftSize = 2048;
    state.micBuffer = new Float32Array(state.analyser.fftSize);
    source.connect(state.analyser);
    state.micActive = true;
    onStatusChange("Mic active. Play a note.");
    pollMic();
  } catch (err) {
    onStatusChange(`Mic error: ${err}`);
  }
}

function pollMic() {
  if (!state.micActive || !state.analyser || !detectPitchFn) return;
  state.analyser.getFloatTimeDomainData(state.micBuffer);
  const result = detectPitchFn(state.micBuffer, state.audioCtx.sampleRate);

  if (result && result.hz > 0 && result.confidence > 0.5) {
    state.currentPitch = result;
  } else {
    state.currentPitch = null;
  }

  requestAnimationFrame(pollMic);
}
