//! Provides a procedural function-like macro [`algebra!`] that generates code for a Geometric
//! Algebra. See the [macro documentation](crate::algebra!()) for more details.

#![warn(clippy::pedantic)]
#![warn(missing_docs, unused_imports, unsafe_code)]

pub use vega_codegen::algebra;
pub use vega_internal::{Blade, ConstSignum, Multivector};

/// Supplies implementations of Projective Geometric Algebra (PGA)
pub mod pga {
    /// Supplies an implementation for 3D PGA
    pub mod three {
        use vega_codegen::algebra;

        algebra!(3, 0, 1);
    }

    /// Supplies an implementation for 2D PGA
    pub mod two {
        use vega_codegen::algebra;

        algebra!(2, 0, 1);
    }
}

/// Supplies implementations of Vanilla Geometric Algebra (VGA)
pub mod vga {
    /// Supplies an implementation for 3D VGA
    pub mod three {
        use vega_codegen::algebra;

        algebra!(3, 0, 0);
    }

    /// Supplies an implementation for 2D VGA
    pub mod two {
        use vega_codegen::algebra;

        algebra!(2, 0, 0);
    }
}
