use std::io::Write;

use passacaglia_common::{rational, rational_value};
use passacaglia_core::structure::Container;
use passacaglia_species_counterpoint::clef::Clef;
use passacaglia_species_counterpoint::score::Score;
use passacaglia_species_counterpoint::voice::{MeasureCursor, NonHarmonicType, Note, Voice};
use quick_xml::events::{BytesDecl, BytesText, Event};
use quick_xml::writer::Writer;

use crate::pitch::pitch;

/// Serialize a domain value to a `MusicXML` document.
pub trait ToMxl {
    fn to_mxl(&self) -> String;
}

fn non_harmonic_mark(n: &Note) -> Option<&'static str> {
    match n.non_harmonic {
        Some(NonHarmonicType::Neighbor) => Some("N"),
        Some(NonHarmonicType::PassingTone) => Some("P"),
        Some(NonHarmonicType::Suspension) => Some("S"),
        None => None,
    }
}

/// Write a `MusicXML` `<note>` element.
///
/// # Errors
///
/// Returns an [`std::io::Error`] if the underlying writer fails.
pub fn note<W: Write>(writer: &mut Writer<W>, n: &Note, tie_start: bool) -> std::io::Result<()> {
    writer
        .create_element("note")
        .write_inner_content(|w| {
            if let Some(p) = n.pitch {
                pitch(w, &p)?;
            } else {
                w.create_element("rest").write_empty()?;
            }

            let duration = rational_value(n.duration * rational(2)).to_string();
            w.create_element("duration")
                .write_text_content(BytesText::new(&duration))?;

            if n.is_tied() || tie_start {
                w.create_element("notations").write_inner_content(|w| {
                    let ty = if tie_start { "start" } else { "stop" };
                    w.create_element("tied")
                        .with_attribute(("type", ty))
                        .write_empty()?;
                    Ok(())
                })?;
            }

            if let Some(mark) = non_harmonic_mark(n) {
                w.create_element("lyric").write_inner_content(|w| {
                    w.create_element("text")
                        .write_text_content(BytesText::new(mark))?;
                    Ok(())
                })?;
            }
            Ok(())
        })?;
    Ok(())
}

fn clef<W: Write>(writer: &mut Writer<W>, clef: Clef) -> std::io::Result<()> {
    writer
        .create_element("clef")
        .write_inner_content(|w| {
            w.create_element("sign")
                .write_text_content(BytesText::new(clef.sign.as_str()))?;
            w.create_element("line")
                .write_text_content(BytesText::new(&clef.line.to_string()))?;
            if let Some(octave) = clef.octave {
                w.create_element("clef-octave-change")
                    .write_text_content(BytesText::new(&octave.to_string()))?;
            }
            Ok(())
        })?;
    Ok(())
}

fn measure<W: Write>(writer: &mut Writer<W>, s: &Score, m: MeasureCursor<'_>) -> std::io::Result<()> {
    let v = m.container();
    let number = (m.index() + 1).to_string();
    let chord = s
        .harmony
        .elements
        .get(m.index())
        .and_then(|ce| ce.chord.as_ref());
    let is_last = v.index() == s.voices.len() - 1;

    writer
        .create_element("measure")
        .with_attribute(("number", number.as_str()))
        .write_inner_content(|w| {
            if is_last {
                if let Some(chord) = chord {
                    let words = chord.to_string();
                    if !words.is_empty() {
                        w.create_element("direction")
                            .with_attribute(("placement", "below"))
                            .write_inner_content(|w| {
                                w.create_element("direction-type").write_inner_content(|w| {
                                    w.create_element("words")
                                        .write_text_content(BytesText::new(&words))?;
                                    Ok(())
                                })?;
                                Ok(())
                            })?;
                    }
                }
            }

            if m.index() == 0 {
                w.create_element("attributes").write_inner_content(|w| {
                    w.create_element("divisions")
                        .write_text_content(BytesText::new("2"))?;
                    clef(w, v.clef())?;
                    Ok(())
                })?;
            }

            let mut n = m.first_child();
            while let Some(nc) = n {
                let tie_start = nc.next_global().is_some_and(|nn| nn.is_tied());
                note(w, &nc, tie_start)?;
                n = nc.next();
            }
            Ok(())
        })?;
    Ok(())
}

fn part<W: Write>(writer: &mut Writer<W>, s: &Score, v: &Voice, voice_id: &str) -> std::io::Result<()> {
    writer
        .create_element("part")
        .with_attribute(("id", voice_id))
        .write_inner_content(|w| {
            for i in 0..v.measures().len() {
                let m = v.cursor(i).expect("measure cursor");
                measure(w, s, m)?;
            }
            Ok(())
        })?;
    Ok(())
}

impl ToMxl for Score {
    fn to_mxl(&self) -> String {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
        writer
            .write_event(Event::Decl(BytesDecl::new("1.0", None, None)))
            .expect("write declaration");
        writer
            .create_element("score-partwise")
            .with_attribute(("version", "4.0"))
            .write_inner_content(|w| {
                w.create_element("part-list").write_inner_content(|w| {
                    for (i, v) in self.voices.iter().enumerate() {
                        let id = i.to_string();
                        w.create_element("score-part")
                            .with_attribute(("id", id.as_str()))
                            .write_inner_content(|w| {
                                w.create_element("part-name")
                                    .write_text_content(BytesText::new(v.name()))?;
                                Ok(())
                            })?;
                    }
                    Ok(())
                })?;

                for (i, v) in self.voices.iter().enumerate() {
                    let id = i.to_string();
                    part(w, self, v, &id)?;
                }
                Ok(())
            })
            .expect("write score-partwise");
        String::from_utf8(writer.into_inner()).expect("valid UTF-8")
    }
}
