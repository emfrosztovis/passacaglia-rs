use passacaglia_core::std_hept::Pitch;
use passacaglia_musicxml::pitch;
use quick_xml::Writer;

fn render(p: &Pitch) -> String {
    let mut writer = Writer::new(Vec::new());
    pitch(&mut writer, p).unwrap();
    String::from_utf8(writer.into_inner()).unwrap()
}

#[test]
fn natural() {
    assert_eq!(
        render(&Pitch::parse("c4").unwrap()),
        "<pitch><step>C</step><alter>0</alter><octave>4</octave></pitch>"
    );
}

#[test]
fn sharp() {
    assert_eq!(
        render(&Pitch::parse("fs5").unwrap()),
        "<pitch><step>F</step><alter>1</alter><octave>5</octave></pitch>"
    );
}

#[test]
fn flat() {
    assert_eq!(
        render(&Pitch::parse("bf3").unwrap()),
        "<pitch><step>B</step><alter>-1</alter><octave>3</octave></pitch>"
    );
}
