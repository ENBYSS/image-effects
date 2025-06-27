use std::ops::RangeBounds;

use rand::{Rng, seq::SliceRandom};

use crate::utils::process_range;

// Structure

pub trait BendLocal {
    fn bend(&mut self, affect: &mut [u8]);
}

pub trait BendGlobal<R: RangeBounds<usize>> {
    fn bend(&mut self, whole: &mut [u8], chunk: R);
}

// Bend effects

pub struct Reverse;

impl BendLocal for Reverse {
    fn bend(&mut self, affect: &mut [u8]) {
        affect.reverse();
    }
}

pub struct Shuffle<'a, R: Rng>(&'a mut R);

impl<'a, R: Rng> BendLocal for Shuffle<'a, R> {
    fn bend(&mut self, affect: &mut [u8]) {
        affect.shuffle(&mut self.0)
    }
}

pub struct Multiply {
    pub multiply_by: usize,
}

impl BendLocal for Multiply {
    fn bend(&mut self, affect: &mut [u8]) {
        for byte in affect.iter_mut() {
            *byte = (*byte as usize).wrapping_mul(self.multiply_by) as u8;
        }
    }
}

pub struct Increment {
    pub increment_by: usize,
}

impl BendLocal for Increment {
    fn bend(&mut self, affect: &mut [u8]) {
        for byte in affect.iter_mut() {
            *byte = byte.wrapping_add(self.increment_by as u8);
        }
    }
}

pub struct Accelerate {
    pub accelerate_by: usize,
    pub accelerate_in: usize,
}

impl BendLocal for Accelerate {
    fn bend(&mut self, affect: &mut [u8]) {
        for (i, byte) in affect.iter_mut().enumerate() {
            let increment = (i as f32 / self.accelerate_in as f32) * self.accelerate_by as f32;
            *byte = byte.wrapping_add(increment as u8);
        }
    }
}

pub struct Compress {
    pub factor: usize,
}

impl BendLocal for Compress {
    fn bend(&mut self, affect: &mut [u8]) {
        let compressed_len = (affect.len() as f32 / self.factor as f32).floor() as u8;

        for i in 0..compressed_len {
            affect[i as usize] = affect[i as usize * self.factor];
        }
    }
}

pub struct Noise<'a, R: Rng>(&'a mut R);

impl<'a, R: Rng> BendLocal for Noise<'a, R> {
    fn bend(&mut self, affect: &mut [u8]) {
        for byte in affect.iter_mut() {
            *byte = self.0.gen_range(u8::MIN..=u8::MAX);
        }
    }
}

pub struct Blackout;

impl BendLocal for Blackout {
    fn bend(&mut self, affect: &mut [u8]) {
        for byte in affect.iter_mut() {
            *byte = 0;
        }
    }
}

pub struct Shift {
    pub shift_by: isize,
}

impl<R: RangeBounds<usize>> BendGlobal<R> for Shift {
    fn bend(&mut self, whole: &mut [u8], chunk: R) {
        let (start, end) = process_range(chunk, whole.len());

        if start as isize == 0 && self.shift_by < 0 {
            panic!("at start - shift_by smaller than 0");
        } else if end as isize == whole.len() as isize && self.shift_by > 0 {
            panic!("at end - shift by bigger than 0");
        } else if end as isize > whole.len() as isize {
            panic!("end is beyond limits");
        } else if start as isize + self.shift_by < 0 {
            panic!("start would be shifted beyond limits");
        } else if end as isize + self.shift_by > whole.len() as isize {
            panic!("end would be shifted beyond limits");
        }

        let mut shifted = Vec::new();
        whole[start..end].clone_into(&mut shifted);

        let chunksize = end - start;

        if self.shift_by < 0 {
            // shift left
            let mut affected = Vec::new();
            let affected_start = start - self.shift_by.unsigned_abs();
            whole[affected_start .. start].clone_into(&mut affected);

            whole[affected_start .. affected_start + chunksize].copy_from_slice(&shifted);
            whole[affected_start + chunksize .. affected_start + chunksize + self.shift_by.unsigned_abs()].copy_from_slice(&affected);
        } else {
            // shift right
            let mut affected = Vec::new();
            let affected_end = end + self.shift_by as usize;
            whole[end .. affected_end].clone_into(&mut affected);

            whole[affected_end - chunksize .. affected_end].copy_from_slice(&shifted);
            whole[affected_end - chunksize - self.shift_by as usize .. affected_end - chunksize].copy_from_slice(&affected);
        }
    }
}

pub struct Repeat {
    pub n: usize,
}

impl<R: RangeBounds<usize>> BendGlobal<R> for Repeat {
    fn bend(&mut self, whole: &mut [u8], chunk: R) {
        let (start, end) = process_range(chunk, whole.len());
        let chunksize = end - start;
        if end + chunksize * self.n >= whole.len() {
            panic!("nope");
        }

        let mut byte_chunk = Vec::new();
        whole[start..end].clone_into(&mut byte_chunk);

        for i in 0..self.n {
            whole[end * i .. end * i + chunksize].copy_from_slice(&byte_chunk);
        }
    }
}

pub struct Swap {
    pub block_at: usize,
}

impl<R: RangeBounds<usize>> BendGlobal<R> for Swap {
    fn bend(&mut self, whole: &mut [u8], chunk: R) {
        let (start, end) = process_range(chunk, whole.len());
        let chunksize = end - start;

        if self.block_at + chunksize > whole.len() {
            panic!("tried to swap beyond end");
        } else if self.block_at < end && self.block_at >= start {
            panic!("can't swap intersecting blocks");
        }

        if self.block_at < start {
            let (left, right) = whole.split_at_mut(self.block_at + chunksize);
            let left_len = left.len();
            left[self.block_at..].swap_with_slice(&mut right[start - left_len .. end - left_len]);
        } else {
            let (left, right) = whole.split_at_mut(end);
            let left_len = left.len();
            left[start..].swap_with_slice(&mut right[self.block_at - left_len .. self.block_at + chunksize - left_len]);
        }
    }
}