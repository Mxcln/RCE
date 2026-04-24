use rubiks_core::CubeState;

pub fn encode_kociemba_facelets(cube: &CubeState) -> String {
    let facelets = cube.to_facelets();
    let mut encoded = String::with_capacity(54);

    for face in facelets {
        for color in face {
            encoded.push(color.as_char());
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use rubiks_core::{parse_canonical_notation, CubeState};

    use super::encode_kociemba_facelets;

    #[test]
    fn solved_state_matches_standard_cubestring() {
        let encoded = encode_kociemba_facelets(&CubeState::solved());
        assert_eq!(
            encoded,
            "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB"
        );
    }

    #[test]
    fn encoding_preserves_kociemba_face_order() {
        let mut cube = CubeState::solved();
        let seq = parse_canonical_notation("R U").unwrap();
        cube.apply_sequence(&seq);

        let encoded = encode_kociemba_facelets(&cube);
        assert_eq!(encoded.len(), 54);
        assert!(encoded
            .chars()
            .all(|ch| matches!(ch, 'U' | 'R' | 'F' | 'D' | 'L' | 'B')));
    }
}
