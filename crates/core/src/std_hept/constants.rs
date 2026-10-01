use std::sync::LazyLock;

use passacaglia_common::rational;

use crate::std_hept::system::StandardHeptatonic;

type Pitch = crate::pitch::Pitch<StandardHeptatonic>;
type Interval = crate::interval::Interval<StandardHeptatonic>;
type Scale = crate::scale::Scale<StandardHeptatonic>;

/// The natural pitch classes (period 0).
pub struct PitchClasses {
    pub c: Pitch,
    pub d: Pitch,
    pub e: Pitch,
    pub f: Pitch,
    pub g: Pitch,
    pub a: Pitch,
    pub b: Pitch,
}

pub static PITCH_CLASSES: PitchClasses = PitchClasses {
    c: Pitch::new(0, rational(0), 0),
    d: Pitch::new(1, rational(0), 0),
    e: Pitch::new(2, rational(0), 0),
    f: Pitch::new(3, rational(0), 0),
    g: Pitch::new(4, rational(0), 0),
    a: Pitch::new(5, rational(0), 0),
    b: Pitch::new(6, rational(0), 0),
};

fn parse_ints(abbrs: &[&str]) -> Vec<Interval> {
    abbrs
        .iter()
        .map(|s| Interval::parse(s).expect("valid interval literal"))
        .collect()
}

fn parse_pitches(abbrs: &[&str]) -> Vec<Pitch> {
    abbrs
        .iter()
        .map(|s| Pitch::parse(s).expect("valid pitch literal"))
        .collect()
}

pub static C_MAJOR: LazyLock<Scale> = LazyLock::new(|| {
    let ints = parse_ints(&["M2", "M2", "m2", "M2", "M2", "M2", "m2"]);
    Scale::from_intervals(Pitch::new(0, rational(0), 0), &ints)
});

pub static C_HARMONIC_MINOR: LazyLock<Scale> = LazyLock::new(|| {
    let ints = parse_ints(&["M2", "m2", "M2", "M2", "m2", "A2", "m2"]);
    Scale::from_intervals(Pitch::new(0, rational(0), 0), &ints)
});

pub static C_CHROMATIC: LazyLock<Scale> = LazyLock::new(|| {
    let degs = parse_pitches(&[
        "c", "cs", "df", "d", "ds", "ef", "e", "f", "fs", "gf", "g", "gs", "af", "a", "as", "bf",
        "b",
    ]);
    Scale::from_pitches(&degs)
});

pub static C_COMPLETE_MINOR: LazyLock<Scale> = LazyLock::new(|| {
    let degs = parse_pitches(&["c", "d", "ef", "f", "g", "af", "a", "bf", "b"]);
    Scale::from_pitches(&degs)
});

pub static C_ASCENDING_MINOR: LazyLock<Scale> = LazyLock::new(|| {
    let degs = parse_pitches(&["c", "d", "ef", "f", "g", "a", "b"]);
    Scale::from_pitches(&degs)
});

pub static C_DESCENDING_MINOR: LazyLock<Scale> = LazyLock::new(|| {
    let degs = parse_pitches(&["c", "d", "ef", "f", "gf", "af", "b"]);
    Scale::from_pitches(&degs)
});

pub mod scales {
    use std::sync::LazyLock;

    use super::{
        Pitch, Scale, C_ASCENDING_MINOR, C_CHROMATIC, C_COMPLETE_MINOR, C_DESCENDING_MINOR,
        C_HARMONIC_MINOR, C_MAJOR,
    };

    pub mod c {
        use super::*;

        pub static IONIAN: LazyLock<Scale> = LazyLock::new(|| (*C_MAJOR).clone());
        pub static DORIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(1, false));
        pub static PHRYGIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(2, false));
        pub static LYDIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(3, false));
        pub static MIXOLYDIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(4, false));
        pub static AEOLIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(5, false));
        pub static LOCRIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(6, false));

        pub static MAJOR: LazyLock<Scale> = LazyLock::new(|| (*C_MAJOR).clone());
        pub static HARMONIC_MINOR: LazyLock<Scale> = LazyLock::new(|| (*C_HARMONIC_MINOR).clone());
        pub static CHROMATIC: LazyLock<Scale> = LazyLock::new(|| (*C_CHROMATIC).clone());
        pub static COMPLETE_MINOR: LazyLock<Scale> = LazyLock::new(|| (*C_COMPLETE_MINOR).clone());
        pub static ASCENDING_MINOR: LazyLock<Scale> =
            LazyLock::new(|| (*C_ASCENDING_MINOR).clone());
        pub static DESCENDING_MINOR: LazyLock<Scale> =
            LazyLock::new(|| (*C_DESCENDING_MINOR).clone());
    }

    pub fn major(root: Pitch) -> Scale {
        C_MAJOR.transpose_to(&root)
    }
    pub fn harmonic_minor(root: Pitch) -> Scale {
        C_HARMONIC_MINOR.transpose_to(&root)
    }
    pub fn complete_minor(root: Pitch) -> Scale {
        C_COMPLETE_MINOR.transpose_to(&root)
    }

    pub fn ionian(root: Pitch) -> Scale {
        c::IONIAN.transpose_to(&root)
    }
    pub fn dorian(root: Pitch) -> Scale {
        c::DORIAN.transpose_to(&root)
    }
    pub fn phrygian(root: Pitch) -> Scale {
        c::PHRYGIAN.transpose_to(&root)
    }
    pub fn lydian(root: Pitch) -> Scale {
        c::LYDIAN.transpose_to(&root)
    }
    pub fn mixolydian(root: Pitch) -> Scale {
        c::MIXOLYDIAN.transpose_to(&root)
    }
    pub fn aeolian(root: Pitch) -> Scale {
        c::AEOLIAN.transpose_to(&root)
    }
    pub fn locrian(root: Pitch) -> Scale {
        c::LOCRIAN.transpose_to(&root)
    }
}
