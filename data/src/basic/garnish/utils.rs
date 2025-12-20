use garnish_lang_traits::Extents;

use crate::BasicNumber;

pub(crate) fn extents_to_start_end(extents: Extents<BasicNumber>, base: usize, len: usize) -> (usize, usize) {
    let start: usize = base + 1 + usize::from(extents.start()).min(len);
    let end: usize = base + 1 + (usize::from(extents.end())).min(len);
    (start, end)
}