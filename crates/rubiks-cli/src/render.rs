use rubiks_core::{Color, Cube};

pub fn ascii_plain(cube: &Cube) -> String {
    let facelets = cube.to_facelets();
    let mut lines = Vec::with_capacity(9);

    for row in 0..3 {
        lines.push(format!(
            "        {} {} {}",
            cell(facelets[0][row * 3]),
            cell(facelets[0][row * 3 + 1]),
            cell(facelets[0][row * 3 + 2]),
        ));
    }

    for row in 0..3 {
        lines.push(format!(
            "{}   {}   {}   {}",
            face_row(&facelets[4], row),
            face_row(&facelets[2], row),
            face_row(&facelets[1], row),
            face_row(&facelets[5], row),
        ));
    }

    for row in 0..3 {
        lines.push(format!(
            "        {} {} {}",
            cell(facelets[3][row * 3]),
            cell(facelets[3][row * 3 + 1]),
            cell(facelets[3][row * 3 + 2]),
        ));
    }

    lines.join("\n")
}

pub fn ascii(cube: &Cube) -> String {
    ascii_plain(cube)
}

fn face_row(face: &[Color; 9], row: usize) -> String {
    format!(
        "{} {} {}",
        cell(face[row * 3]),
        cell(face[row * 3 + 1]),
        cell(face[row * 3 + 2]),
    )
}

fn cell(color: Color) -> char {
    color.as_char()
}

#[cfg(test)]
mod tests {
    use rubiks_core::Cube;

    use super::*;

    #[test]
    fn solved_ascii_layout_is_stable() {
        let rendered = ascii_plain(&Cube::solved());
        assert_eq!(
            rendered,
            "        U U U\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}U U U\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}U U U\n\
             L L L   F F F   R R R   B B B\n\
             L L L   F F F   R R R   B B B\n\
             L L L   F F F   R R R   B B B\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}D D D\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}D D D\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}D D D"
        );
    }
}
