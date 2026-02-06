import { COLORS } from "../constants.js";
import { roundRect, midiToName, getCurrentNote } from "./base.js";

const PIXELS_PER_BEAT = 80;
const SEMITONE_HEIGHT = 18;
const PLAYHEAD_X_FRACTION = 0.25;
const NOTE_HEIGHT = 16;
const NOTE_RADIUS = 6;

export function drawGameView(canvas, ctx, score, currentBeat, currentPitch, pitchTrail) {
  const rect = canvas.getBoundingClientRect();
  const w = rect.width;
  const h = rect.height;
  ctx.clearRect(0, 0, w, h);

  if (!score || !score.notes.length) {
    ctx.fillStyle = COLORS.muted;
    ctx.font = "16px Space Grotesk, sans-serif";
    ctx.fillText("Load a piece to begin", w / 2 - 80, h / 2);
    return;
  }

  // Compute MIDI range from score
  let midiMin = 127;
  let midiMax = 0;
  for (const note of score.notes) {
    if (!note.is_rest) {
      midiMin = Math.min(midiMin, note.midi);
      midiMax = Math.max(midiMax, note.midi);
    }
  }
  // Pad range by 3 semitones on each side
  midiMin = Math.max(0, midiMin - 3);
  midiMax = Math.min(127, midiMax + 3);

  const playheadX = w * PLAYHEAD_X_FRACTION;
  const midiRange = midiMax - midiMin + 1;
  const totalMidiHeight = midiRange * SEMITONE_HEIGHT;
  const yOffset = (h - totalMidiHeight) / 2;

  function midiToY(midi) {
    // Higher notes at top
    return yOffset + (midiMax - midi) * SEMITONE_HEIGHT + SEMITONE_HEIGHT / 2;
  }

  function beatToX(beat) {
    return playheadX + (beat - currentBeat) * PIXELS_PER_BEAT;
  }

  // Draw horizontal note lanes (alternating bands)
  for (let midi = midiMin; midi <= midiMax; midi++) {
    const y = yOffset + (midiMax - midi) * SEMITONE_HEIGHT;
    const pitchClass = midi % 12;
    // Natural notes get lighter background
    const isNatural = [0, 2, 4, 5, 7, 9, 11].includes(pitchClass);
    ctx.fillStyle = isNatural ? "rgba(0,0,0,0.02)" : "rgba(0,0,0,0.06)";
    ctx.fillRect(0, y, w, SEMITONE_HEIGHT);

    // C notes get a subtle line
    if (pitchClass === 0) {
      ctx.strokeStyle = "rgba(0,0,0,0.08)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, y + SEMITONE_HEIGHT);
      ctx.lineTo(w, y + SEMITONE_HEIGHT);
      ctx.stroke();
    }
  }

  // Draw note name labels on left edge
  ctx.font = "11px Space Grotesk, sans-serif";
  ctx.fillStyle = COLORS.muted;
  for (let midi = midiMin; midi <= midiMax; midi++) {
    const pitchClass = midi % 12;
    if ([0, 2, 4, 5, 7, 9, 11].includes(pitchClass)) {
      const y = midiToY(midi);
      ctx.fillText(midiToName(midi), 4, y + 4);
    }
  }

  // Draw notes
  const activeNote = getCurrentNote(score, currentBeat);

  for (const note of score.notes) {
    const x = beatToX(note.start_beat);
    const noteEndX = beatToX(note.start_beat + note.duration_beats);

    // Skip off-screen notes
    if (noteEndX < 0 || x > w) continue;

    const drawX = Math.max(0, x);
    const drawWidth = Math.max(4, Math.min(noteEndX, w) - drawX);

    if (note.is_rest) {
      continue; // Don't draw rests in game view
    }

    const y = midiToY(note.midi) - NOTE_HEIGHT / 2;

    // Color based on state
    let color;
    if (activeNote === note) {
      color = COLORS.noteActive;
    } else if (note.start_beat + note.duration_beats <= currentBeat) {
      color = COLORS.noteUpcoming; // past
      ctx.globalAlpha = 0.4;
    } else {
      color = COLORS.noteUpcoming; // upcoming
    }

    ctx.fillStyle = color;
    roundRect(ctx, drawX, y, drawWidth, NOTE_HEIGHT, NOTE_RADIUS);
    ctx.fill();
    ctx.globalAlpha = 1.0;

    // Draw pitch accuracy band around active note
    if (activeNote === note) {
      ctx.fillStyle = COLORS.pitchBand;
      const bandHeight = NOTE_HEIGHT + SEMITONE_HEIGHT;
      ctx.fillRect(drawX, y - (bandHeight - NOTE_HEIGHT) / 2, drawWidth, bandHeight);
    }
  }

  // Draw playhead
  ctx.strokeStyle = COLORS.ink;
  ctx.lineWidth = 2.5;
  ctx.beginPath();
  ctx.moveTo(playheadX, 10);
  ctx.lineTo(playheadX, h - 10);
  ctx.stroke();

  // Draw playhead glow
  ctx.strokeStyle = "rgba(228, 103, 75, 0.3)";
  ctx.lineWidth = 8;
  ctx.beginPath();
  ctx.moveTo(playheadX, 10);
  ctx.lineTo(playheadX, h - 10);
  ctx.stroke();

  // Draw pitch trail (history of player's pitch)
  if (pitchTrail && pitchTrail.length > 1) {
    ctx.beginPath();
    ctx.strokeStyle = "rgba(228, 103, 75, 0.35)";
    ctx.lineWidth = 2;
    let started = false;
    for (const pt of pitchTrail) {
      const px = beatToX(pt.beat);
      if (px < 0 || px > playheadX) continue;
      const py = midiToY(pt.midi_float);
      if (py < yOffset || py > yOffset + totalMidiHeight) continue;
      if (!started) {
        ctx.moveTo(px, py);
        started = true;
      } else {
        ctx.lineTo(px, py);
      }
    }
    if (started) ctx.stroke();
  }

  // Draw player's pitch indicator
  if (currentPitch && currentPitch.hz > 0 && currentPitch.confidence > 0.5) {
    const pitchMidi = currentPitch.midi_float;
    if (pitchMidi >= midiMin && pitchMidi <= midiMax) {
      const pitchY = midiToY(pitchMidi);
      // Glowing circle
      ctx.beginPath();
      ctx.arc(playheadX, pitchY, 8, 0, Math.PI * 2);
      ctx.fillStyle = COLORS.accent;
      ctx.fill();
      ctx.beginPath();
      ctx.arc(playheadX, pitchY, 12, 0, Math.PI * 2);
      ctx.strokeStyle = "rgba(228, 103, 75, 0.4)";
      ctx.lineWidth = 3;
      ctx.stroke();
    }
  }

  // Beat markers along bottom
  ctx.fillStyle = COLORS.muted;
  ctx.font = "10px Space Grotesk, sans-serif";
  const firstVisibleBeat = Math.max(0, Math.floor(currentBeat - playheadX / PIXELS_PER_BEAT));
  const lastVisibleBeat = Math.ceil(currentBeat + (w - playheadX) / PIXELS_PER_BEAT);
  for (let b = firstVisibleBeat; b <= lastVisibleBeat && b <= score.total_beats; b++) {
    const x = beatToX(b);
    if (x < 0 || x > w) continue;
    // Draw measure lines (using measures data if available)
    const isMeasureStart = score.measures && score.measures.some((m) => Math.abs(m.start_beat - b) < 0.01);
    if (isMeasureStart) {
      ctx.strokeStyle = "rgba(0,0,0,0.12)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(x, yOffset);
      ctx.lineTo(x, yOffset + totalMidiHeight);
      ctx.stroke();
    }
    // Beat numbers at bottom
    if (b % 4 === 0) {
      ctx.fillText(b.toString(), x - 3, h - 4);
    }
  }
}
