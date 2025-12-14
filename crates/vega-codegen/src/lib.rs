//! Provides a procedural function-like macro [`algebra!`] that generates code for a Geometric
//! Algebra. See the [macro documentation](crate::algebra!()) for more details.

#![warn(clippy::pedantic)]
#![warn(missing_docs, unused_imports, unsafe_code)]

use algebra::pipeline;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod algebra;
mod helpers;

/// Generates a geometric algebra from three positional arguments:
/// 1. The number of dimensions whose unit vectors square to one.
/// 2. The number of dimensions whose unit vectors square to minus one.
/// 3. The number of dimensions whose unit vectors square to zero.
///
/// The second and third arguments are optional, and assumed zero if missing.
///
/// # Example
///
/// ```rust
/// use vega_codegen::algebra;
///
/// algebra!(2);
///
/// let multivector = 1 + E1(2) + E2(3) + E12(4);
/// assert_eq!(Multivector { s: 1, e1: E1(2), e2: E2(3), e12: E12(4) }, multivector);
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn algebra(input: TokenStream) -> TokenStream {
    pipeline(input)
}
