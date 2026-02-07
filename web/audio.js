import { state } from "./state.js";

let detectPitchFn = null;

export function setDetectPitch(fn) {
  detectPitchFn = fn;
}

function parseResult(result) {
  // Handle Float64Array [hz, confidence, midi_float]
  if (result instanceof Float64Array || (result && result.length === 3 && typeof result[0] === "number")) {
    return { hz: result[0], confidence: result[1], midi_float: result[2] };
  }
  // Fallback: serde-style object
  return result;
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

    // Try AudioWorklet first, fall back to AnalyserNode
    let workletActive = false;
    try {
      await state.audioCtx.audioWorklet.addModule("./pitch-processor.js");
      const workletNode = new AudioWorkletNode(state.audioCtx, "pitch-processor");
      source.connect(workletNode);
      workletNode.connect(state.audioCtx.destination);

      workletNode.port.onmessage = (event) => {
        if (!detectPitchFn) return;
        const samples = event.data;
        const raw = detectPitchFn(samples, state.audioCtx.sampleRate);
        const result = parseResult(raw);
        if (result && result.hz > 0 && result.confidence > 0.5) {
          state.currentPitch = result;
        } else {
          state.currentPitch = null;
        }
      };

      workletActive = true;
    } catch (_e) {
      // AudioWorklet not supported or failed, fall back
    }

    if (!workletActive) {
      state.analyser = state.audioCtx.createAnalyser();
      state.analyser.fftSize = 2048;
      state.micBuffer = new Float32Array(state.analyser.fftSize);
      source.connect(state.analyser);
      pollMic();
    }

    state.micActive = true;
    onStatusChange("Mic active. Play a note.");
  } catch (err) {
    onStatusChange(`Mic error: ${err}`);
  }
}

function pollMic() {
  if (!state.micActive || !state.analyser || !detectPitchFn) return;
  state.analyser.getFloatTimeDomainData(state.micBuffer);
  const raw = detectPitchFn(state.micBuffer, state.audioCtx.sampleRate);
  const result = parseResult(raw);

  if (result && result.hz > 0 && result.confidence > 0.5) {
    state.currentPitch = result;
  } else {
    state.currentPitch = null;
  }

  requestAnimationFrame(pollMic);
}
