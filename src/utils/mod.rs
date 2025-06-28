use std::ops::RangeBounds;

pub mod image;
pub mod numops;

pub fn process_range<R: RangeBounds<usize>>(range: R, len: usize) -> (usize, usize) {
    (
        match range.start_bound() {
            std::ops::Bound::Unbounded => 0,
            std::ops::Bound::Included(x) => *x,
            std::ops::Bound::Excluded(x) => *x,
        },
        match range.end_bound() {
            std::ops::Bound::Unbounded => len,
            std::ops::Bound::Included(x) => *x + 1,
            std::ops::Bound::Excluded(x) => *x,
        },
    )
}
