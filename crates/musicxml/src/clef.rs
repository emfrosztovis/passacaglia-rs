/// A clef sign (G, C, or F).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClefType {
    G,
    C,
    F,
}

impl ClefType {
    /// The `MusicXML` `<sign>` value for this clef type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            ClefType::G => "G",
            ClefType::C => "C",
            ClefType::F => "F",
        }
    }
}

/// A clef: a sign plus the staff line and an optional octave transposition.
///
/// `octave` maps to `MusicXML`'s `clef-octave-change` (`None` = no change).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Clef {
    pub sign: ClefType,
    pub line: i32,
    pub octave: Option<i32>,
}

impl Clef {
    pub const TREBLE: Clef = Clef {
        sign: ClefType::G,
        line: 2,
        octave: None,
    };
    pub const TREBLE_8VB: Clef = Clef {
        sign: ClefType::G,
        line: 2,
        octave: Some(-1),
    };
    pub const TREBLE_8VA: Clef = Clef {
        sign: ClefType::G,
        line: 2,
        octave: Some(1),
    };
    pub const ALTO: Clef = Clef {
        sign: ClefType::C,
        line: 3,
        octave: None,
    };
    pub const BASS: Clef = Clef {
        sign: ClefType::F,
        line: 4,
        octave: None,
    };
}
