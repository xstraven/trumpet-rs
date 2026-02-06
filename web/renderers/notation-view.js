import { COLORS } from "../constants.js";
import { roundRect, midiToName, getCurrentNote } from "./base.js";

const LAYOUT = {
  leftPad: 40,
  rightPad: 40,
  topPad: 40,
  lineSpacing: 14,
  noteHeight: 10,
  semitoneStep: 4,
  refMidi: 71, // B4, middle line of treble clef
};

export function drawNotationView(canvas, ctx, score, currentBeat, totalBeats) {
  const rect = canvas.getBoundingClientRect();
  const w = rect.width;
  const h = rect.height;
  ctx.clearRect(0, 0, w, h);

  // Draw staff lines
  const staffTop = h / 2 - LAYOUT.lineSpacing * 2;
  ctx.strokeStyle = COLORS.staffLine;
  ctx.lineWidth = 1.2;
  for (let i = 0; i < 5; i++) {
    const y = staffTop + i * LAYOUT.lineSpacing;
    ctx.beginPath();
    ctx.moveTo(LAYOUT.leftPad, y);
    ctx.lineTo(w - LAYOUT.rightPad, y);
    ctx.stroke();
  }

  if (!score) return;

  function beatToX(beat) {
    const usable = w - LAYOUT.leftPad - LAYOUT.rightPad;
    if (totalBeats <= 0) return LAYOUT.leftPad;
    return LAYOUT.leftPad + (beat / totalBeats) * usable;
  }

  function midiToY(midi) {
    const staffCenter = h / 2;
    return staffCenter - (midi - LAYOUT.refMidi) * LAYOUT.semitoneStep;
  }

  // Draw barlines
  if (score.measures) {
    ctx.strokeStyle = "rgba(0,0,0,0.15)";
    ctx.lineWidth = 1;
    for (const measure of score.measures) {
      if (measure.start_beat > 0) {
        const x = beatToX(measure.start_beat);
        ctx.beginPath();
        ctx.moveTo(x, staffTop);
        ctx.lineTo(x, staffTop + LAYOUT.lineSpacing * 4);
        ctx.stroke();
      }
    }
  }

  const activeNote = getCurrentNote(score, currentBeat);

  // Draw notes
  for (const note of score.notes) {
    const x = beatToX(note.start_beat);
    const noteWidth = Math.max(6, beatToX(note.start_beat + note.duration_beats) - x);

    if (note.is_rest) {
      ctx.fillStyle = COLORS.rest;
      const restY = h / 2 - LAYOUT.noteHeight / 2;
      ctx.fillRect(x, restY, noteWidth, LAYOUT.noteHeight * 0.8);
      continue;
    }

    const y = midiToY(note.midi) - LAYOUT.noteHeight / 2;
    ctx.fillStyle = activeNote === note ? COLORS.noteActive : COLORS.noteUpcoming;
    roundRect(ctx, x, y, noteWidth, LAYOUT.noteHeight, 6);
    ctx.fill();

    ctx.fillStyle = COLORS.ink;
    ctx.font = "12px Space Grotesk, sans-serif";
    ctx.fillText(midiToName(note.midi), x + 4, y - 6);
  }

  // Draw playhead
  const playheadX = beatToX(currentBeat);
  ctx.strokeStyle = COLORS.ink;
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(playheadX, LAYOUT.topPad / 2);
  ctx.lineTo(playheadX, h - LAYOUT.topPad / 2);
  ctx.stroke();
}
