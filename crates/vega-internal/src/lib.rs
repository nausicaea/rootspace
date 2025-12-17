//! `vega-internal` provides traits and functionality that enables creating and using geometric
//! algebras.

#![warn(clippy::pedantic)]
#![warn(unused_imports, unsafe_code)]

pub mod ops;

/// Trait for a geometric algebra multivector
pub trait Multivector: Sized {
    /// Return the grade of the multivector
    fn grade(&self) -> Option<usize>;

    /// Grade projection: obtain the k-vector blades of a Multivector, where k is the selected
    /// grade.
    #[must_use]
    fn gproj(&self, k: usize) -> Self;
}

/// Trait for a geometric algebra k-vector blade
pub trait Blade<R: ConstSignum>: Sized {
    /// Defines the value of the square of a unit blade, i.e. `e1 * e1 = 1`.
    const UNIT_SQUARE_VALUE: R;
    /// Defines the grade of the blade
    const GRADE: usize;
}

/// Trait for a type that supports minus one, zero, and one values.
pub trait ConstSignum: Sized {
    /// Defines the value of `zero`
    const ZERO: Self;
    /// Defines the value of `one`
    const ONE: Self;
    /// Defines the value of `one`
    const MINUS_ONE: Self;
}

macro_rules! impl_const_signum {
    ($($type:ty => ($zero:literal, $one:literal, $minus_one:literal));+ $(;)*) => {
        $(
            impl ConstSignum for $type {
                const ZERO: Self = $zero;
                const ONE: Self = $one;
                const MINUS_ONE: Self = $minus_one;
            }
        )*
    };
}

impl_const_signum! {
    i8 => (0, 1, -1);
    i16 => (0, 1, -1);
    i32 => (0, 1, -1);
    i64 => (0, 1, -1);
    i128 => (0, 1, -1);
    isize => (0, 1, -1);
    f32 => (0.0, 1.0, -1.0);
    f64 => (0.0, 1.0, -1.0);
}
