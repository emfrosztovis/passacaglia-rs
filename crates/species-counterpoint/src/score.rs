use im::Vector;

use passacaglia_common::Rational;

use crate::chord::Harmony;
use crate::voice::Voice;

/// Score-level parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Parameters {
    pub measure_length: Rational,
}

/// A score: parameters, a list of voices, and a harmonic background.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Score {
    pub parameters: Parameters,
    pub voices: Vector<Voice>,
    pub harmony: Harmony,
}

impl Score {
    #[must_use]
    pub fn new(parameters: Parameters, voices: Vec<Voice>, harmony: Harmony) -> Score {
        Score {
            parameters,
            voices: voices.into_iter().collect(),
            harmony,
        }
    }

    #[must_use]
    pub fn replace_harmony(&self, h: Harmony) -> Score {
        Score {
            parameters: self.parameters,
            voices: self.voices.clone(),
            harmony: h,
        }
    }

    #[must_use]
    pub fn replace_voice(&self, i: usize, v: Voice) -> Score {
        Score {
            parameters: self.parameters,
            voices: self.voices.update(i, v),
            harmony: self.harmony.clone(),
        }
    }
}
