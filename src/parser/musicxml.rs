use quick_xml::events::Event;
use quick_xml::Reader;

use crate::scoring::types::{MeasureInfo, NoteEvent, Score, TransposeInfo};

pub fn midi_from_pitch(step: char, alter: i32, octave: i32) -> i32 {
    let base = match step {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => 0,
    };
    (octave + 1) * 12 + base + alter
}

pub fn parse_musicxml(xml: &str) -> Result<Score, String> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut buf = Vec::new();

    let mut divisions: f64 = 1.0;
    let mut tempo: f64 = 120.0;
    let mut notes: Vec<NoteEvent> = Vec::new();
    let mut measures: Vec<MeasureInfo> = Vec::new();

    let mut current_beat: f64 = 0.0;
    let mut last_note_start: f64 = 0.0;
    let mut last_note_duration: f64 = 0.0;

    let mut current_tag: Option<&'static str> = None;

    // Note state
    let mut in_note = false;
    let mut note_is_rest = false;
    let mut note_is_chord = false;
    let mut note_duration_divs: Option<f64> = None;
    let mut note_type_str: String = String::new();
    let mut step: Option<char> = None;
    let mut alter: i32 = 0;
    let mut octave: Option<i32> = None;

    // Measure state
    let mut current_measure_number: u32 = 0;
    let mut measure_start_beat: f64 = 0.0;

    // Score-level metadata
    let mut key_fifths: i32 = 0;
    let mut time_sig_num: u8 = 4;
    let mut time_sig_den: u8 = 4;
    let mut title: Option<String> = None;

    // Transpose state
    let mut transpose: Option<TransposeInfo> = None;
    let mut in_transpose = false;
    let mut transpose_chromatic: i32 = 0;
    let mut transpose_diatonic: i32 = 0;

    // Tag context tracking
    let mut in_type_tag = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"measure" => {
                        // Finalize previous measure if any
                        if current_measure_number > 0 {
                            measures.push(MeasureInfo {
                                number: current_measure_number,
                                start_beat: measure_start_beat,
                                duration_beats: current_beat - measure_start_beat,
                                time_sig_num,
                                time_sig_den,
                            });
                        }
                        // Parse measure number attribute
                        if let Some(attr) = e
                            .attributes()
                            .flatten()
                            .find(|a| a.key.as_ref() == b"number")
                        {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                if let Ok(n) = val.parse::<u32>() {
                                    current_measure_number = n;
                                }
                            }
                        }
                        measure_start_beat = current_beat;
                    }
                    b"note" => {
                        in_note = true;
                        note_is_rest = false;
                        note_is_chord = false;
                        note_duration_divs = None;
                        note_type_str.clear();
                        step = None;
                        alter = 0;
                        octave = None;
                    }
                    b"rest" => {
                        if in_note {
                            note_is_rest = true;
                        }
                    }
                    b"chord" => {
                        if in_note {
                            note_is_chord = true;
                        }
                    }
                    b"transpose" => {
                        in_transpose = true;
                        transpose_chromatic = 0;
                        transpose_diatonic = 0;
                    }
                    b"divisions" => current_tag = Some("divisions"),
                    b"duration" => current_tag = Some("duration"),
                    b"step" => current_tag = Some("step"),
                    b"alter" => current_tag = Some("alter"),
                    b"octave" => current_tag = Some("octave"),
                    b"per-minute" => current_tag = Some("per-minute"),
                    b"fifths" => current_tag = Some("fifths"),
                    b"beats" => current_tag = Some("beats"),
                    b"beat-type" => current_tag = Some("beat-type"),
                    b"chromatic" => current_tag = Some("chromatic"),
                    b"diatonic" => current_tag = Some("diatonic"),
                    b"movement-title" => current_tag = Some("movement-title"),
                    b"work-title" => current_tag = Some("work-title"),
                    b"type" => {
                        if in_note {
                            in_type_tag = true;
                            current_tag = Some("type");
                        }
                    }
                    b"sound" => {
                        if let Some(attr) =
                            e.attributes().flatten().find(|a| a.key.as_ref() == b"tempo")
                        {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                if let Ok(t) = val.parse::<f64>() {
                                    tempo = t;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"sound" {
                    if let Some(attr) =
                        e.attributes().flatten().find(|a| a.key.as_ref() == b"tempo")
                    {
                        if let Ok(val) = std::str::from_utf8(&attr.value) {
                            if let Ok(t) = val.parse::<f64>() {
                                tempo = t;
                            }
                        }
                    }
                }
                if name.as_ref() == b"rest" && in_note {
                    note_is_rest = true;
                }
                if name.as_ref() == b"chord" && in_note {
                    note_is_chord = true;
                }
            }
            Ok(Event::Text(e)) => {
                if let Some(tag) = current_tag.take() {
                    let text = e.unescape().map_err(|e| e.to_string())?;
                    match tag {
                        "divisions" => {
                            if let Ok(v) = text.parse::<f64>() {
                                if v > 0.0 {
                                    divisions = v;
                                }
                            }
                        }
                        "per-minute" => {
                            if let Ok(v) = text.parse::<f64>() {
                                tempo = v;
                            }
                        }
                        "duration" => {
                            if let Ok(v) = text.parse::<f64>() {
                                note_duration_divs = Some(v);
                            }
                        }
                        "step" => {
                            step = text.chars().next();
                        }
                        "alter" => {
                            if let Ok(v) = text.parse::<i32>() {
                                alter = v;
                            }
                        }
                        "octave" => {
                            if let Ok(v) = text.parse::<i32>() {
                                octave = Some(v);
                            }
                        }
                        "fifths" => {
                            if let Ok(v) = text.parse::<i32>() {
                                key_fifths = v;
                            }
                        }
                        "beats" => {
                            if let Ok(v) = text.parse::<u8>() {
                                time_sig_num = v;
                            }
                        }
                        "beat-type" => {
                            if let Ok(v) = text.parse::<u8>() {
                                time_sig_den = v;
                            }
                        }
                        "chromatic" => {
                            if in_transpose {
                                if let Ok(v) = text.parse::<i32>() {
                                    transpose_chromatic = v;
                                }
                            }
                        }
                        "diatonic" => {
                            if in_transpose {
                                if let Ok(v) = text.parse::<i32>() {
                                    transpose_diatonic = v;
                                }
                            }
                        }
                        "type" => {
                            if in_type_tag {
                                note_type_str = text.to_string();
                                in_type_tag = false;
                            }
                        }
                        "movement-title" | "work-title" => {
                            if title.is_none() {
                                title = Some(text.to_string());
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"note" if in_note => {
                        let duration_divs = note_duration_divs.unwrap_or(0.0);
                        let duration_beats = if divisions > 0.0 {
                            duration_divs / divisions
                        } else {
                            0.0
                        };

                        let start_beat = if note_is_chord {
                            last_note_start
                        } else {
                            current_beat
                        };

                        let midi = if note_is_rest {
                            -1
                        } else {
                            let s = step.ok_or("Missing pitch step")?;
                            let o = octave.ok_or("Missing pitch octave")?;
                            midi_from_pitch(s, alter, o)
                        };

                        notes.push(NoteEvent {
                            start_beat,
                            duration_beats,
                            midi,
                            is_rest: note_is_rest,
                            measure_number: current_measure_number,
                            note_type: if note_type_str.is_empty() {
                                "quarter".to_string()
                            } else {
                                note_type_str.clone()
                            },
                        });

                        if !note_is_chord {
                            last_note_start = start_beat;
                            last_note_duration = duration_beats;
                            current_beat += duration_beats;
                        } else if last_note_duration == 0.0 {
                            last_note_duration = duration_beats;
                        }

                        in_note = false;
                    }
                    b"transpose" => {
                        in_transpose = false;
                        transpose = Some(TransposeInfo {
                            chromatic: transpose_chromatic,
                            diatonic: transpose_diatonic,
                        });
                    }
                    b"type" => {
                        in_type_tag = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    // Finalize the last measure
    if current_measure_number > 0 {
        measures.push(MeasureInfo {
            number: current_measure_number,
            start_beat: measure_start_beat,
            duration_beats: current_beat - measure_start_beat,
            time_sig_num,
            time_sig_den,
        });
    }

    let total_beats = current_beat;

    Ok(Score {
        tempo,
        notes,
        measures,
        key_fifths,
        transpose,
        title,
        total_beats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_from_pitch() {
        assert_eq!(midi_from_pitch('C', 0, 4), 60);
        assert_eq!(midi_from_pitch('A', 0, 4), 69);
        assert_eq!(midi_from_pitch('C', 1, 4), 61);
        assert_eq!(midi_from_pitch('B', -1, 4), 70);
        assert_eq!(midi_from_pitch('G', 0, 3), 55);
    }

    #[test]
    fn test_parse_simple_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 3.1 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">
<score-partwise version="3.1">
  <part-list><score-part id="P1"><part-name>Trumpet</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
      </attributes>
      <direction>
        <direction-type><metronome><beat-unit>quarter</beat-unit><per-minute>120</per-minute></metronome></direction-type>
      </direction>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <pitch><step>D</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <rest/>
        <duration>1</duration>
        <type>quarter</type>
      </note>
      <note>
        <pitch><step>E</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).unwrap();
        assert_eq!(score.tempo, 120.0);
        assert_eq!(score.total_beats, 4.0);
        assert_eq!(score.notes.len(), 4);
        assert_eq!(score.measures.len(), 1);
        assert_eq!(score.key_fifths, 0);
        assert!(score.transpose.is_none());

        // Measure info
        assert_eq!(score.measures[0].number, 1);
        assert_eq!(score.measures[0].start_beat, 0.0);
        assert_eq!(score.measures[0].duration_beats, 4.0);
        assert_eq!(score.measures[0].time_sig_num, 4);
        assert_eq!(score.measures[0].time_sig_den, 4);

        // C4
        assert_eq!(score.notes[0].midi, 60);
        assert_eq!(score.notes[0].start_beat, 0.0);
        assert_eq!(score.notes[0].duration_beats, 1.0);
        assert_eq!(score.notes[0].measure_number, 1);
        assert_eq!(score.notes[0].note_type, "quarter");
        assert!(!score.notes[0].is_rest);

        // D4
        assert_eq!(score.notes[1].midi, 62);
        assert_eq!(score.notes[1].start_beat, 1.0);

        // Rest
        assert!(score.notes[2].is_rest);
        assert_eq!(score.notes[2].midi, -1);

        // E4
        assert_eq!(score.notes[3].midi, 64);
        assert_eq!(score.notes[3].note_type, "quarter");
    }

    #[test]
    fn test_parse_transpose() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="3.1">
  <part-list><score-part id="P1"><part-name>Trumpet</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <key><fifths>0</fifths></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <transpose>
          <diatonic>-1</diatonic>
          <chromatic>-2</chromatic>
        </transpose>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>1</duration>
        <type>quarter</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).unwrap();
        assert!(score.transpose.is_some());
        let t = score.transpose.unwrap();
        assert_eq!(t.chromatic, -2);
        assert_eq!(t.diatonic, -1);
    }

    #[test]
    fn test_parse_multiple_measures() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="3.1">
  <part-list><score-part id="P1"><part-name>Trumpet</part-name></score-part></part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>1</divisions>
        <time><beats>4</beats><beat-type>4</beat-type></time>
      </attributes>
      <note>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>4</duration>
        <type>whole</type>
      </note>
    </measure>
    <measure number="2">
      <note>
        <pitch><step>D</step><octave>4</octave></pitch>
        <duration>2</duration>
        <type>half</type>
      </note>
      <note>
        <pitch><step>E</step><octave>4</octave></pitch>
        <duration>2</duration>
        <type>half</type>
      </note>
    </measure>
  </part>
</score-partwise>"#;

        let score = parse_musicxml(xml).unwrap();
        assert_eq!(score.measures.len(), 2);
        assert_eq!(score.measures[0].number, 1);
        assert_eq!(score.measures[0].duration_beats, 4.0);
        assert_eq!(score.measures[1].number, 2);
        assert_eq!(score.measures[1].start_beat, 4.0);
        assert_eq!(score.measures[1].duration_beats, 4.0);
        assert_eq!(score.total_beats, 8.0);

        assert_eq!(score.notes[0].note_type, "whole");
        assert_eq!(score.notes[0].measure_number, 1);
        assert_eq!(score.notes[1].note_type, "half");
        assert_eq!(score.notes[1].measure_number, 2);
    }

    #[test]
    fn test_parse_happy_birthday() {
        let xml = include_str!("../../web/assets/happy_birthday.musicxml");
        let score = parse_musicxml(xml).unwrap();
        assert_eq!(score.tempo, 92.0);
        assert_eq!(score.key_fifths, 0);
        assert_eq!(score.measures.len(), 8);
        assert_eq!(score.total_beats, 32.0);
        // First note is G3
        assert_eq!(score.notes[0].midi, midi_from_pitch('G', 0, 3));
        assert_eq!(score.notes[0].measure_number, 1);
    }

    #[test]
    fn test_parse_ode_to_joy() {
        let xml = include_str!("../../web/assets/ode_to_joy.musicxml");
        let score = parse_musicxml(xml).unwrap();
        assert_eq!(score.tempo, 96.0);
        assert_eq!(score.measures.len(), 8);
        assert_eq!(score.total_beats, 32.0);
        // First note is E4
        assert_eq!(score.notes[0].midi, midi_from_pitch('E', 0, 4));
    }
}
