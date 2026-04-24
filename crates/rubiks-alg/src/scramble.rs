use rand::prelude::*;
use rubiks_core::{Direction, Face, Move, MoveSequence};

use crate::error::ScrambleError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScrambleMode {
    RandomState3x3,
    TrainingFaceTurn { length: usize },
}

pub trait ScrambleGenerator {
    fn generate(&self, mode: ScrambleMode) -> Result<MoveSequence, ScrambleError>;

    fn generate_with_rng<R: Rng + ?Sized>(
        &self,
        mode: ScrambleMode,
        rng: &mut R,
    ) -> Result<MoveSequence, ScrambleError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TrainingScrambleGenerator;

impl ScrambleGenerator for TrainingScrambleGenerator {
    fn generate(&self, mode: ScrambleMode) -> Result<MoveSequence, ScrambleError> {
        let mut rng = rand::thread_rng();
        self.generate_with_rng(mode, &mut rng)
    }

    fn generate_with_rng<R: Rng + ?Sized>(
        &self,
        mode: ScrambleMode,
        rng: &mut R,
    ) -> Result<MoveSequence, ScrambleError> {
        match mode {
            ScrambleMode::TrainingFaceTurn { length } => {
                if length == 0 {
                    return Err(ScrambleError::InvalidLength { length });
                }

                let mut moves = Vec::with_capacity(length);
                let mut previous_face = None;

                while moves.len() < length {
                    let face = Face::ALL[rng.gen_range(0..Face::ALL.len())];
                    if previous_face == Some(face) {
                        continue;
                    }
                    let direction = match rng.gen_range(0..3) {
                        0 => Direction::CW,
                        1 => Direction::CCW,
                        _ => Direction::Double,
                    };
                    moves.push(Move::new(face, direction));
                    previous_face = Some(face);
                }

                Ok(MoveSequence(moves))
            }
            other => Err(ScrambleError::UnsupportedMode { mode: other }),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn zero_length_is_rejected() {
        let generator = TrainingScrambleGenerator;
        assert_eq!(
            generator.generate(ScrambleMode::TrainingFaceTurn { length: 0 }),
            Err(ScrambleError::InvalidLength { length: 0 })
        );
    }

    #[test]
    fn random_state_is_not_implemented_yet() {
        let generator = TrainingScrambleGenerator;
        assert!(matches!(
            generator.generate(ScrambleMode::RandomState3x3),
            Err(ScrambleError::UnsupportedMode { .. })
        ));
    }

    #[test]
    fn training_scramble_has_requested_length_and_no_consecutive_face_repeat() {
        let generator = TrainingScrambleGenerator;
        let mut rng = StdRng::seed_from_u64(42);
        let sequence = generator
            .generate_with_rng(ScrambleMode::TrainingFaceTurn { length: 25 }, &mut rng)
            .unwrap();
        assert_eq!(sequence.len(), 25);
        for pair in sequence.0.windows(2) {
            assert_ne!(pair[0].face, pair[1].face);
        }
    }
}
