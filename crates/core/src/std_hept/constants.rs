use std::sync::LazyLock;

use passacaglia_common::{Rational, rational};

use crate::{std_hept::system::StdHeptatonic};

type Pitch = crate::pitch::Pitch<StdHeptatonic>;
type Interval = crate::interval::Interval<StdHeptatonic>;
type Scale = crate::scale::Scale<StdHeptatonic>;
type DegreeDefinition = crate::scale::DegreeDefinition<StdHeptatonic>;

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

fn parse_degree_defs(abbrs: &[(&str, &[Rational])]) -> Vec<DegreeDefinition> {
    abbrs
        .iter()
        .map(|(s, alts)| DegreeDefinition {
            pitch: Pitch::parse(s).expect("valid pitch literal"),
            alterations: alts.to_vec()
        })
        .collect()
}

pub static C_MAJOR: LazyLock<Scale> = LazyLock::new(|| {
    let ints = parse_ints(&["M2", "M2", "m2", "M2", "M2", "M2", "m2"]);
    Scale::from_intervals(Pitch::new(0, rational(0), 0), &ints)
});

pub static C_CHROMATIC: LazyLock<Scale> = LazyLock::new(|| {
    let defs = parse_degree_defs(&[
        ("c", &[rational(-1), rational(1)]),
        ("d", &[rational(-1), rational(1)]),
        ("e", &[rational(-1), rational(1)]),
        ("f", &[rational(-1), rational(1)]),
        ("g", &[rational(-1), rational(1)]),
        ("a", &[rational(-1), rational(1)]),
        ("b", &[rational(-1), rational(1)]),
    ]);
    Scale::from_definitions(&defs)
});

pub static C_MINOR: LazyLock<Scale> = LazyLock::new(|| {
    let defs = parse_degree_defs(&[
        ("c", &[]),
        ("d", &[]),
        ("ef", &[]),
        ("f", &[]),
        ("g", &[]),
        ("af", &[rational(1)]),
        ("bf", &[rational(1)]),
    ]);
    Scale::from_definitions(&defs)
});

pub mod scales {
    use std::sync::LazyLock;

    use super::{
        Pitch, Scale, C_CHROMATIC, C_MAJOR, C_MINOR,
    };

    /// Predefined scale constants starting at C.
    pub mod c {
        use super::*;

        /// Ionian mode starting at C. Equivalent to [`MAJOR`].
        pub static IONIAN: LazyLock<Scale> = LazyLock::new(|| (*C_MAJOR).clone());
        /// Dorian mode starting at C.
        pub static DORIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(1, false));
        /// Phrygian mode starting at C.
        pub static PHRYGIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(2, false));
        /// Lydian mode starting at C.
        pub static LYDIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(3, false));
        /// Mixolydian mode starting at C.
        pub static MIXOLYDIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(4, false));
        /// Aeolian mode starting at C. Equivalent to [`MINOR`] without the altered tones.
        pub static AEOLIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(5, false));
        /// Locrian mode starting at C.
        pub static LOCRIAN: LazyLock<Scale> = LazyLock::new(|| C_MAJOR.rotate(6, false));

        /// C major scale.
        pub static MAJOR: LazyLock<Scale> = LazyLock::new(|| (*C_MAJOR).clone());

        /// Chromatic scale.
        pub static CHROMATIC: LazyLock<Scale> = LazyLock::new(|| (*C_CHROMATIC).clone());

        /// C minor scale, defined as the Aeolian mode with allowed altered tones of VI-sharp and 
        /// VII-sharp (see Schoenberg, *Theory of Harmony*, Ch. 5). Note that many systems treat
        /// unaltered minor degrees as referring to the harmonic or melodic minor (hence the 
        /// existence of VII-flat etc.), which is *different* from our approach.
        pub static MINOR: LazyLock<Scale> = LazyLock::new(|| (*C_MINOR).clone());
    }

    /// Get the major scale starting at a given pitch.
    pub fn major(root: Pitch) -> Scale {
        C_MAJOR.transpose_to(&root)
    }

    /// Get the minor scale starting at a given pitch.
    /// 
    /// The minor scale is defined here as the Aeolian mode with allowed altered tones of VI-sharp 
    /// and VII-sharp (see Schoenberg, *Theory of Harmony*, Ch. 5). Note that many systems treat
    /// unaltered minor degrees as referring to the harmonic or melodic minor (hence the 
    /// existence of VII-flat etc.), which is *different* from our approach.
    pub fn minor(root: Pitch) -> Scale {
        C_MINOR.transpose_to(&root)
    }

    /// Get the Ionian mode scale starting at a given pitch. Equivalent to [`major`].
    pub fn ionian(root: Pitch) -> Scale {
        c::IONIAN.transpose_to(&root)
    }
    /// Get the Dorian mode scale starting at a given pitch.
    pub fn dorian(root: Pitch) -> Scale {
        c::DORIAN.transpose_to(&root)
    }
    /// Get the Phrygian mode scale starting at a given pitch.
    pub fn phrygian(root: Pitch) -> Scale {
        c::PHRYGIAN.transpose_to(&root)
    }
    /// Get the Lydian mode scale starting at a given pitch.
    pub fn lydian(root: Pitch) -> Scale {
        c::LYDIAN.transpose_to(&root)
    }
    /// Get the Mixolydian mode scale starting at a given pitch.
    pub fn mixolydian(root: Pitch) -> Scale {
        c::MIXOLYDIAN.transpose_to(&root)
    }
    /// Get the Aeolian mode scale starting at a given pitch. Equivalent to [`minor`] without the altered tones.
    pub fn aeolian(root: Pitch) -> Scale {
        c::AEOLIAN.transpose_to(&root)
    }
    /// Get the Locrian mode scale starting at a given pitch.
    pub fn locrian(root: Pitch) -> Scale {
        c::LOCRIAN.transpose_to(&root)
    }
}
