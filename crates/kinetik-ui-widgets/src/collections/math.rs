pub(crate) fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

pub(crate) fn finite_positive(value: f32) -> Option<f32> {
    value
        .is_finite()
        .then_some(value)
        .filter(|value| *value > 0.0)
}

pub(crate) fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

pub(crate) fn finite_sum(lhs: f32, rhs: f32) -> f32 {
    let sum = lhs + rhs;
    if sum.is_finite() {
        sum
    } else if sum.is_sign_negative() {
        f32::MIN
    } else {
        f32::MAX
    }
}

#[allow(clippy::cast_precision_loss)]
pub(crate) fn finite_index_extent(index: usize, extent: f32) -> f32 {
    let offset = index as f32 * extent;
    if offset.is_finite() { offset } else { f32::MAX }
}
