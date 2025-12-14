//! Custom operators.
//!
//! Implementing these traits signals to users the availability to perform corresponding
//! mathematical operations. None of these traits provide infix notation equivalents, so they must
//! be called with associated functions.

pub trait InnerMul<R, Rhs = Self> {
    type Output;

    fn imul(self, rhs: Rhs) -> Self::Output;
}

pub trait OuterMul<R, Rhs = Self> {
    type Output;

    fn omul(self, rhs: Rhs) -> Self::Output;
}

pub trait RegressiveMul<Rhs = Self> {
    type Output;

    fn rmul(self, rhs: Rhs) -> Self::Output;
}

/// Reverse flips the signs of every two grades:
///
/// 0. +
/// 1. +
/// 2. -
/// 3. -
/// 4. +
/// 5. +
/// 6. -
/// 7. -
pub trait Reverse {
    type Output;

    fn rev(self) -> Self::Output;
}

/// Grade involution flips the signs of every other grade:
///
/// 0. +
/// 1. -
/// 2. +
/// 3. -
/// 4. +
/// 5. -
/// 6. +
/// 7. -
pub trait GradeInvolution {
    type Output;

    fn ginv(self) -> Self::Output;
}

/// The dual operation calculates the perpendicular correspondent blade
pub trait Dual {
    type Output;

    fn dual(self) -> Self::Output;
}

/// The magnitude calculates the metric of length generalized to multivectors
pub trait Magnitude {
    type Output;

    fn mag_sq(self) -> Self::Output;
}
