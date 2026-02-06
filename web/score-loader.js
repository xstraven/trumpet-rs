import { state } from "./state.js";
import { BUILTIN_PIECES } from "./constants.js";

let parseMusicxmlFn = null;

export function setParseMusicxml(fn) {
  parseMusicxmlFn = fn;
}

export async function parseAndLoad(xml, sourceName = "MusicXML") {
  if (!state.wasmReady || !parseMusicxmlFn) {
    return { ok: false, error: "WASM not ready yet." };
  }
  try {
    const parsed = parseMusicxmlFn(xml);
    state.playing = false;
    state.lastBeat = 0;
    state.score = parsed;
    state.totalBeats = parsed.total_beats || 0;
    if (state.totalBeats === 0) {
      for (const note of parsed.notes) {
        state.totalBeats = Math.max(state.totalBeats, note.start_beat + note.duration_beats);
      }
    }
    state.secondsPerBeat = 60 / parsed.tempo;
    return { ok: true, tempo: parsed.tempo };
  } catch (err) {
    return { ok: false, error: `Error parsing ${sourceName}: ${err}` };
  }
}

export async function loadBuiltinPiece(pieceId) {
  const piece = BUILTIN_PIECES.find((c) => c.id === pieceId);
  if (!piece) {
    return { ok: false, error: "Unknown built-in piece selected." };
  }

  try {
    const response = await fetch(piece.path);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const xml = await response.text();
    return await parseAndLoad(xml, piece.label);
  } catch (err) {
    return { ok: false, error: `Error loading "${piece.label}": ${err}` };
  }
}

async function extractMxl(arrayBuffer) {
  if (typeof JSZip === "undefined") {
    throw new Error("JSZip not loaded. Cannot open .mxl files.");
  }
  const zip = await JSZip.loadAsync(arrayBuffer);

  // Try META-INF/container.xml to find the root file
  const containerFile = zip.file("META-INF/container.xml");
  if (containerFile) {
    const containerXml = await containerFile.async("string");
    const match = containerXml.match(/full-path\s*=\s*"([^"]+)"/);
    if (match) {
      const rootFile = zip.file(match[1]);
      if (rootFile) {
        return await rootFile.async("string");
      }
    }
  }

  // Fallback: find the first .xml file that isn't in META-INF
  for (const [path, entry] of Object.entries(zip.files)) {
    if (!entry.dir && path.endsWith(".xml") && !path.startsWith("META-INF")) {
      return await entry.async("string");
    }
  }

  throw new Error("No MusicXML file found inside the .mxl archive.");
}

export async function loadFileUpload(file) {
  if (file.name.endsWith(".pdf")) {
    return {
      ok: false,
      error:
        "PDF files need to be converted to MusicXML first. Try musescore.com/import or the free Audiveris desktop app.",
    };
  }

  if (file.name.endsWith(".mxl")) {
    try {
      const buf = await file.arrayBuffer();
      const xml = await extractMxl(buf);
      return await parseAndLoad(xml, file.name);
    } catch (err) {
      return { ok: false, error: `Error reading .mxl file: ${err.message}` };
    }
  }

  const xml = await file.text();
  return await parseAndLoad(xml, file.name);
}
