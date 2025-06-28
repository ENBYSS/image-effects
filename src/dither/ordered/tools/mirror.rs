use crate::dither::ordered::tools::Matrix;

#[derive(Debug, Clone, Copy)]
pub enum MirrorDirection {
    Horizontal,
    Vertical,
    Downright,
    Upright,
}

#[derive(Debug, Clone)]
pub struct MirrorLine {
    pub direction: MirrorDirection,
    pub flip: bool,
    pub thorough: bool,
}

impl MirrorLine {
    pub fn mirror(&self, matrix: &mut Matrix) {
        _mirror(matrix, self.direction, self.flip, self.thorough);
    }
}

fn _mirror(matrix: &mut Matrix, direction: MirrorDirection, flip: bool, thorough: bool) {
    let (x, y) = matrix.dim();

    if x < 2 {
        return;
    }

    if x != y {
        panic!("Tried to mirror a malformed ordered pattern")
    }

    fn check_flip(
        side1: (usize, usize),
        side2: (usize, usize),
        flip: bool,
    ) -> ((usize, usize), (usize, usize)) {
        if flip {
            (side1, side2)
        } else {
            (side2, side1)
        }
    }

    fn mirror_map_reflect(
        matrix: &mut Matrix,
        side1: (usize, usize),
        side2: (usize, usize),
        flip: bool,
    ) {
        let (source, target) = check_flip(side1, side2, flip);
        let mirrored = { *matrix.get(source).unwrap() };
        let pix = matrix.get_mut(target).unwrap();
        *pix = mirrored;
    }

    fn mirror_map_through(
        matrix: &mut Matrix,
        side1: (usize, usize),
        side2: (usize, usize),
        flip: bool,
    ) {
        let (source, target) = check_flip(side1, side2, flip);
        let mirror_value = (*matrix.get(source).unwrap() + *matrix.get(target).unwrap()) / 2.;
        {
            let source_p = matrix.get_mut(source).unwrap();
            *source_p = mirror_value;
        }
        {
            let target_p = matrix.get_mut(target).unwrap();
            *target_p = mirror_value;
        }
    }

    fn mirror_map(
        matrix: &mut Matrix,
        side1: (usize, usize),
        side2: (usize, usize),
        flip: bool,
        through: bool,
    ) {
        if through {
            mirror_map_through(matrix, side1, side2, flip);
        } else {
            mirror_map_reflect(matrix, side1, side2, flip);
        }
    }

    match direction {
        MirrorDirection::Horizontal => {
            for cy in 0..y {
                for cx in 0..x / 2 {
                    let side1 = (cy, cx);
                    let side2 = (cy, x - cx - 1);

                    mirror_map(matrix, side1, side2, flip, thorough);
                }
            }
        }
        MirrorDirection::Vertical => {
            for cy in 0..y / 2 {
                for cx in 0..x {
                    let side1 = (cy, cx);
                    let side2 = (y - cy - 1, cx);

                    mirror_map(matrix, side1, side2, flip, thorough);
                }
            }
        }
        MirrorDirection::Downright => {
            for cy in 0..y {
                for cx in 0..x {
                    let side1 = (cy, cx);
                    let side2 = (cx, cy);

                    mirror_map(matrix, side1, side2, flip, thorough);
                }
            }
        }
        MirrorDirection::Upright => {
            for cy in 0..y {
                for cx in 0..x {
                    let side1 = (cy, cx);
                    let side2 = (y - cy - 1, x - cx - 1);

                    mirror_map(matrix, side1, side2, flip, thorough);
                }
            }
        }
    }
}
