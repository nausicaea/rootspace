use itertools::Itertools;
use quote::format_ident;
use syn::{parse_quote as pq, Path};

use crate::algebra::{
    ir::blades::{
        Blades, IrBladeAddTraitImpls, IrBladeAddTraitImplsWithOtherBlade,
        IrBladeAddTraitImplsWithOtherBladeTable, IrBladeAddTraitImplsWithScalarLhs,
        IrBladeAddTraitImplsWithScalarRhs, IrBladeCloneTraitImpls, IrBladeCopyTraitImpls,
        IrBladeDebugTraitImpls, IrBladeEqTraitImpls, IrBladeHashTraitImpls, IrBladeMulTraitImpls,
        IrBladeMulTraitImplsWithOtherBlade, IrBladeMulTraitImplsWithOtherBladeTable,
        IrBladeMulTraitImplsWithScalarLhs, IrBladeMulTraitImplsWithScalarRhs, IrBladeNegTraitImpls,
        IrBladeOrdTraitImpls, IrBladePartialEqTraitImpls, IrBladePartialOrdTraitImpls,
        IrBladeSubTraitImpls, IrBladeSubTraitImplsWithOtherBlade,
        IrBladeSubTraitImplsWithOtherBladeTable, IrBladeSubTraitImplsWithScalarLhs,
        IrBladeSubTraitImplsWithScalarRhs, IrBladeTraitImpls, IrBladeZeroTraitImpls,
        IrNewtypeStruct,
    },
    model::{self, Cm},
};
use crate::helpers::to_where_clause;

#[allow(clippy::too_many_lines)]
pub fn lower(model: &model::Algebra) -> Blades<'_> {
    let model::Algebra {
        scalar_type,
        signed_primitive_types,
        multivector_ident,
        scalar_field_name,
        blade_field_names,
        blade_types,
        blade_mul_table,
        positive_vector_blade_indices,
        negative_vector_blade_indices,
        zero_vector_blade_indices,
        higher_order_blade_indices,
        higher_order_unit_square_values,
        ..
    } = model;

    let copy: Path = pq!(::core::marker::Copy);
    let add: Path = pq!(::core::ops::Add);
    let sub: Path = pq!(::core::ops::Sub);
    let neg: Path = pq!(::core::ops::Neg);
    let mul: Path = pq!(::core::ops::Mul);
    let zero: Path = pq!(::num_traits::Zero);
    let const_signum: Path = pq!(::vega_internal::ConstSignum);

    Blades {
        struct_defs: IrNewtypeStruct {
            idents: blade_types,
            scalar_type,
        },
        positive_blade_trait_impls: IrBladeTraitImpls {
            idents: positive_vector_blade_indices
                .iter()
                .map(|idx| format_ident!("E{idx}"))
                .collect::<Vec<_>>(),
            unit_square_values: vec![pq!(#const_signum::ONE); positive_vector_blade_indices.len()],
            grades: vec![1; positive_vector_blade_indices.len()],
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #const_signum)]),
        },
        negative_blade_trait_impls: IrBladeTraitImpls {
            idents: negative_vector_blade_indices
                .iter()
                .map(|idx| format_ident!("E{idx}"))
                .collect::<Vec<_>>(),
            unit_square_values: vec![
                pq!(#const_signum::MINUS_ONE);
                negative_vector_blade_indices.len()
            ],
            grades: vec![1; negative_vector_blade_indices.len()],
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #const_signum)]),
        },
        zero_blade_trait_impls: IrBladeTraitImpls {
            idents: zero_vector_blade_indices
                .iter()
                .map(|idx| format_ident!("E{idx}"))
                .collect::<Vec<_>>(),
            unit_square_values: vec![pq!(#const_signum::ZERO); zero_vector_blade_indices.len()],
            grades: vec![1; zero_vector_blade_indices.len()],
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #const_signum)]),
        },
        higher_order_blade_trait_impls: IrBladeTraitImpls {
            idents: higher_order_blade_indices
                .iter()
                .map(|idxs| format_ident!("E{}", idxs.iter().join("")))
                .collect::<Vec<_>>(),
            unit_square_values: higher_order_unit_square_values
                .iter()
                .map(|usv| match usv {
                    0 => pq!(::vega_internal::ConstSignum::ZERO),
                    1 => pq!(::vega_internal::ConstSignum::ONE),
                    -1 => pq!(::vega_internal::ConstSignum::MINUS_ONE),
                    _ => unreachable!(),
                })
                .collect(),
            grades: higher_order_blade_indices
                .iter()
                .map(std::vec::Vec::len)
                .collect(),
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #const_signum)]),
        },
        zero_trait_impls: IrBladeZeroTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #zero)]),
        },
        debug_trait_impls: IrBladeDebugTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: ::core::fmt::Debug)]),
        },
        clone_trait_impls: IrBladeCloneTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #copy)]),
        },
        copy_trait_impls: IrBladeCopyTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #copy)]),
        },
        partial_eq_trait_impls: IrBladePartialEqTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: ::core::cmp::PartialEq)]),
        },
        eq_trait_impls: IrBladeEqTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: ::core::cmp::Eq)]),
        },
        partial_ord_trait_impls: IrBladePartialOrdTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: ::core::cmp::PartialOrd)]),
        },
        ord_trait_impls: IrBladeOrdTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: ::core::cmp::Ord)]),
        },
        hash_trait_impls: IrBladeHashTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: ::core::hash::Hash)]),
        },
        add_trait_impls: IrBladeAddTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #add<Output = #scalar_type>)]),
        },
        sub_trait_impls: IrBladeSubTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #sub<Output = #scalar_type>)]),
        },
        neg_trait_impls: IrBladeNegTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #neg<Output = #scalar_type>)]),
        },
        mul_trait_impls: IrBladeMulTraitImpls {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([
                pq!(#scalar_type: #const_signum + #mul<Output = #scalar_type>),
            ]),
        },
        add_trait_impls_with_scalar_rhs: IrBladeAddTraitImplsWithScalarRhs {
            multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            field_types: blade_types,
            where_clause: to_where_clause([pq!(#scalar_type: #zero)]),
        },
        add_trait_impls_with_scalar_lhs: IrBladeAddTraitImplsWithScalarLhs {
            multivector_ident,
            scalar_field_name,
            signed_primitive_types,
            field_names: blade_field_names,
            field_types: blade_types,
        },
        sub_trait_impls_with_scalar_rhs: IrBladeSubTraitImplsWithScalarRhs {
            multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            field_types: blade_types,
            where_clause: to_where_clause([pq!(#scalar_type: #zero + #neg<Output = #scalar_type>)]),
        },
        sub_trait_impls_with_scalar_lhs: IrBladeSubTraitImplsWithScalarLhs {
            multivector_ident,
            scalar_field_name,
            signed_primitive_types,
            field_names: blade_field_names,
            field_types: blade_types,
        },
        mul_trait_impls_with_scalar_rhs: IrBladeMulTraitImplsWithScalarRhs {
            idents: blade_types,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #mul<Output = #scalar_type>)]),
        },
        mul_trait_impls_with_scalar_lhs: IrBladeMulTraitImplsWithScalarLhs {
            idents: blade_types,
            signed_primitive_types,
        },
        add_trait_impls_with_other_blade: IrBladeAddTraitImplsWithOtherBlade {
            multivector_ident,
            scalar_type,
            operation_table: blade_field_names
                .iter()
                .zip(blade_types.iter())
                .permutations(2)
                .map(|pairs| IrBladeAddTraitImplsWithOtherBladeTable {
                    lhs_field_name: pairs[0].0,
                    lhs_field_type: pairs[0].1,
                    rhs_field_name: pairs[1].0,
                    rhs_field_type: pairs[1].1,
                })
                .collect(),
            where_clause: to_where_clause([pq!(#scalar_type: #copy + #zero)]),
        },
        sub_trait_impls_with_other_blade: IrBladeSubTraitImplsWithOtherBlade {
            multivector_ident,
            scalar_type,
            operation_table: blade_field_names
                .iter()
                .zip(blade_types.iter())
                .permutations(2)
                .map(|pairs| IrBladeSubTraitImplsWithOtherBladeTable {
                    lhs_field_name: pairs[0].0,
                    lhs_field_type: pairs[0].1,
                    rhs_field_name: pairs[1].0,
                    rhs_field_type: pairs[1].1,
                })
                .collect(),
            where_clause: to_where_clause([
                pq!(#scalar_type: #copy + #zero + #neg<Output = #scalar_type>),
            ]),
        },
        mul_trait_impls_with_other_blade: IrBladeMulTraitImplsWithOtherBlade {
            scalar_type,
            operation_table: blade_mul_table
                .iter()
                .map(
                    |Cm {
                         lhs,
                         rhs,
                         output,
                         signum,
                     }| {
                        IrBladeMulTraitImplsWithOtherBladeTable {
                            lhs_field_type: format_ident!("E{}", lhs.iter().join("")),
                            rhs_field_type: format_ident!("E{}", rhs.iter().join("")),
                            output_type: format_ident!("E{}", output.iter().join("")),
                            output_signum: match signum {
                                0 => pq!(<#scalar_type as #const_signum>::ZERO),
                                1 => pq!(<#scalar_type as #const_signum>::ONE),
                                -1 => pq!(<#scalar_type as #const_signum>::MINUS_ONE),
                                _ => unreachable!(),
                            },
                        }
                    },
                )
                .collect(),
            where_clause: to_where_clause([
                pq!(#scalar_type: #const_signum + #mul<Output = #scalar_type>),
            ]),
        },
    }
}
