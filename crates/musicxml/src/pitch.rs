use std::io::Write;

use passacaglia_common::rational_value;
use passacaglia_core::std_hept::Pitch;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;

const STEPS: [&str; 7] = ["C", "D", "E", "F", "G", "A", "B"];

/// Write a `MusicXML` `<pitch>` element for a standard-heptatonic pitch.
///
/// # Errors
///
/// Returns an [`std::io::Error`] if the underlying writer fails.
pub fn pitch<W: Write>(writer: &mut Writer<W>, p: &Pitch) -> std::io::Result<()> {
    writer
        .create_element("pitch")
        .write_inner_content(|w| {
            w.create_element("step")
                .write_text_content(BytesText::new(STEPS[p.index]))?;
            w.create_element("alter")
                .write_text_content(BytesText::new(&rational_value(p.acci).to_string()))?;
            w.create_element("octave")
                .write_text_content(BytesText::new(&p.period.to_string()))?;
            Ok(())
        })?;
    Ok(())
}
