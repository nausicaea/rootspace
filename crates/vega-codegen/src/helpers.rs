use itertools::Itertools;
use proc_macro2::Span;
use std::{collections::HashMap, hash::Hash};

use syn::WhereClause;
use syn::WherePredicate;

/// Returns `true` if a sequence of `usize` is sorted with ascending order (low numbers are first)
#[must_use]
pub fn is_sorted_asc(indices: &[usize]) -> bool {
    let sorted: Vec<_> = indices.iter().copied().sorted().collect();
    sorted == indices
}

/// Creates a where clause from a sequence of where predicates
#[must_use]
pub fn to_where_clause<I>(predicates: I) -> WhereClause
where
    I: IntoIterator<Item = WherePredicate>,
{
    WhereClause {
        where_token: syn::token::Where {
            span: Span::call_site(),
        },
        predicates: predicates.into_iter().collect(),
    }
}

/// Calculates the product of a list of operands under the following rules:
///
/// 1. Operands may be swapped with adjacent operands under anti-commutativity
/// 2. Equal neighbouring pairs square to a real value
#[must_use]
pub fn product_in_place<T: Eq + PartialOrd + Hash>(
    operands: &mut Vec<T>,
    unit_square_values: &HashMap<T, i8>,
) -> i8 {
    if operands.is_empty() {
        return 0;
    }

    // Bubble-sort the operand indices and record each swap
    let carry_anticommute = anticommute_new(operands);

    // Contract the operand indices, squaring those that are equal
    let carry_square = square(operands, unit_square_values);

    if carry_anticommute {
        -carry_square
    } else {
        carry_square
    }
}

/// Bubble-sort the operand indices and record each swap
#[must_use]
fn anticommute_new<T: PartialOrd>(operands: &mut [T]) -> bool {
    let mut carry = false;
    loop {
        let mut swapped = false;
        for i in 0..(operands.len() - 1) {
            if operands[i] > operands[i + 1] {
                operands.swap(i, i + 1);
                carry ^= true;
                swapped = true;
            }
        }
        if !swapped {
            break;
        }
    }
    carry
}

/// Contract the operand indices, squaring those that are equal
#[must_use]
fn square<T: Eq + Hash>(operands: &mut Vec<T>, unit_square_values: &HashMap<T, i8>) -> i8 {
    let mut carry: Vec<i8> = vec![];

    // Iterate over all operands in pairs
    let mut iter = operands
        .iter()
        .enumerate()
        .tuple_windows::<((usize, &T), (usize, &T))>();

    while let Some(((lhs_i, lhs), (rhs_i, rhs))) = iter.next() {
        // Skip any operands that aren't equal
        if lhs != rhs {
            continue;
        }

        // Record the result of multiplying lhs with rhs (i.e. squaring lhs)
        carry.push(unit_square_values[lhs]);

        // Remove both elements, making sure to remove the higher index first.
        operands.remove(rhs_i);
        operands.remove(lhs_i);

        // Update the iterator, since we just removed two elements from the operands
        iter = operands
            .iter()
            .enumerate()
            .tuple_windows::<((usize, &T), (usize, &T))>();
    }

    carry.into_iter().product()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_signum() {
        let operand_indices = vec![1, 1];
        let unit_square_values: HashMap<_, _> = [(1, 1)].into_iter().collect();

        let mut final_operands = operand_indices.clone();
        let signum = product_in_place(&mut final_operands, &unit_square_values);

        assert_eq!(1, signum);
        assert_eq!(Vec::<usize>::new(), final_operands);
    }
}
