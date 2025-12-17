use std::collections::HashMap;

use crate::helpers::{is_sorted_asc, product_in_place};

use super::{
    input::Algebra as AlgebraInput,
    model::{Algebra, Cm},
};
use itertools::Itertools;
use proc_macro_error::abort;
use quote::format_ident;
use syn::Ident;
use syn::parse_quote as pq;

const MAX_DIMENSIONS: usize = 9;

/// Converts the structured macro input to an expanded internal data model. [`analyze`] is fallible
/// and may cause compile-time errors if input validation fails.
#[allow(clippy::too_many_lines)]
pub fn analyze(input: &AlgebraInput) -> Algebra {
    let positive_dims: usize = input
        .positive
        .base10_parse()
        .unwrap_or_else(|e| abort!(e.span(), "{}", e));

    if positive_dims == 0 {
        abort! { input.positive,
            "Expected at least one dimension that squares to one";
            note = "A geometric algebra must have at least one regular dimension";
        }
    }

    let negative_dims: usize = input
        .negative
        .base10_parse()
        .unwrap_or_else(|e| abort!(e.span(), "{}", e));

    let zero_dims: usize = input.zero.base10_parse().unwrap_or_else(|e| abort!(e.span(), "{}", e));

    let dims = zero_dims + positive_dims + negative_dims;
    if dims > MAX_DIMENSIONS {
        abort! {input.span,
            "This library can generate geometric algebras for at most {} dimensions, you specified: {}", MAX_DIMENSIONS, dims,
        }
    }

    // Generate the blade indices
    let zero_vector_blade_indices: Vec<_> = (0..zero_dims).collect();
    let offset = usize::from(zero_dims == 0);
    let positive_vector_blade_indices: Vec<_> = (0..positive_dims).map(|d| d + offset + zero_dims).collect();
    let negative_vector_blade_indices: Vec<_> = (0..negative_dims)
        .map(|d| d + offset + zero_dims + positive_dims)
        .collect();

    let vector_blade_indices: Vec<_> = zero_vector_blade_indices
        .iter()
        .chain(positive_vector_blade_indices.iter())
        .chain(negative_vector_blade_indices.iter())
        .copied()
        .collect();

    let vector_unit_square_values: HashMap<usize, i8> = zero_vector_blade_indices
        .iter()
        .map(|zi| (*zi, 0))
        .chain(positive_vector_blade_indices.iter().map(|pi| (*pi, 1)))
        .chain(negative_vector_blade_indices.iter().map(|ni| (*ni, -1)))
        .collect();

    let mut higher_order_blade_indices = vec![];
    for d in 2..=dims {
        let comp = vector_blade_indices
            .iter()
            .copied()
            .permutations(d)
            .filter(|p| is_sorted_asc(p));
        higher_order_blade_indices.extend(comp);
    }

    let higher_order_unit_square_values: Vec<i8> = higher_order_blade_indices
        .iter()
        .map(|idxs| {
            let mut operands: Vec<_> = idxs.iter().chain(idxs.iter()).copied().collect();
            let signum = product_in_place(&mut operands, &vector_unit_square_values);
            debug_assert!(operands.is_empty());
            signum
        })
        .collect();

    let blade_indices: Vec<_> = vector_blade_indices
        .iter()
        .map(|vi| vec![*vi])
        .chain(higher_order_blade_indices.iter().cloned())
        .collect();

    let blade_field_names: Vec<_> = blade_indices
        .iter()
        .map(|idxs| format_ident!("e{}", idxs.iter().join("")))
        .collect::<Vec<_>>();
    let blade_types: Vec<_> = blade_indices
        .iter()
        .map(|idxs| format_ident!("E{}", idxs.iter().join("")))
        .collect::<Vec<_>>();

    let blade_mul_table: Vec<Cm> = blade_indices
        .iter()
        .permutations(2)
        .map(|pair| {
            let mut operands: Vec<usize> = pair[0].iter().chain(pair[1].iter()).copied().collect();

            let signum = product_in_place(&mut operands, &vector_unit_square_values);

            Cm {
                lhs: pair[0].clone(),
                rhs: pair[1].clone(),
                output: operands,
                signum,
            }
        })
        .collect();

    let module_name = Ident::new(
        &format!("algebra_{positive_dims}_{negative_dims}_{zero_dims}"),
        proc_macro2::Span::call_site(),
    );

    Algebra {
        module_name,
        scalar_type: format_ident!("R"),
        signed_primitive_types: vec![
            pq!(i8),
            pq!(i16),
            pq!(i32),
            pq!(i64),
            pq!(i128),
            pq!(isize),
            pq!(f32),
            pq!(f64),
        ],
        multivector_ident: format_ident!("Multivector"),
        scalar_field_name: format_ident!("s"),
        blade_field_names,
        blade_types,
        blade_mul_table,
        positive_vector_blade_indices,
        negative_vector_blade_indices,
        zero_vector_blade_indices,
        higher_order_blade_indices,
        higher_order_unit_square_values,
        blade_indices,
    }
}
