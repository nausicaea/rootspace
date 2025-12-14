use itertools::Itertools;
use quote::format_ident;
use syn::{parse_quote as pq, Path};

use crate::algebra::{
    ir::multivector::{
        IrMultivectorAddTraitImpl, IrMultivectorAddTraitImplWithScalarLhs,
        IrMultivectorAddTraitImplWithScalarRhs, IrMultivectorAddTraitImplsWithBlade,
        IrMultivectorCloneTraitImpl, IrMultivectorCopyTraitImpl, IrMultivectorDebugTraitImpl,
        IrMultivectorEqTraitImpl, IrMultivectorHashTraitImpl, IrMultivectorMulTraitImpl,
        IrMultivectorMulTraitImplWithScalarLhs, IrMultivectorMulTraitImplWithScalarRhs,
        IrMultivectorMulTraitImplsWithBladeLhs, IrMultivectorMulTraitImplsWithBladeRhs,
        IrMultivectorMulTraitImplsWithBladeRhsTable, IrMultivectorNegTraitImpl,
        IrMultivectorPartialEqTraitImpl, IrMultivectorSubTraitImpl,
        IrMultivectorSubTraitImplWithScalarLhs, IrMultivectorSubTraitImplWithScalarRhs,
        IrMultivectorSubTraitImplsWithBlade, IrMultivectorTraitImpl, IrMultivectorZeroTraitImpl,
        IrNamedStruct, Multivector,
    },
    model,
};
use crate::helpers::to_where_clause;

#[allow(clippy::too_many_lines)]
pub fn lower(model: &model::Algebra) -> Multivector {
    let model::Algebra {
        scalar_type,
        signed_primitive_types,
        multivector_ident,
        scalar_field_name,
        blade_field_names,
        blade_types,
        blade_indices,
        ..
    } = model;

    let copy: Path = pq!(::core::marker::Copy);
    let add: Path = pq!(::core::ops::Add);
    let sub: Path = pq!(::core::ops::Sub);
    let neg: Path = pq!(::core::ops::Neg);
    let mul: Path = pq!(::core::ops::Mul);
    let zero: Path = pq!(::num_traits::Zero);
    let const_signum: Path = pq!(::vega_internal::ConstSignum);

    Multivector {
        struct_def: IrNamedStruct {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            field_types: blade_types,
        },
        multivector_trait_impl: IrMultivectorTraitImpl {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            grade_exprs: {
                let groups = blade_indices.iter()
                    .group_by(|idxs| idxs.len());

                let mut ge = vec![];

                for (_, group) in &groups {
                    let (grouped_field_names, grouped_field_types): (Vec<_>, Vec<_>) = group.map(|idxs| (format_ident!("e{}", idxs.iter().join("")), format_ident!("E{}", idxs.iter().join("")))).unzip();
                    ge.push(pq!(#(self.#grouped_field_names != #grouped_field_types::zero())||*));
                }

                ge
            },
            gproj_exprs: {
                let groups = blade_indices.iter()
                    .group_by(|idxs| idxs.len());

                let mut ge = vec![];

                for (_, group) in &groups {
                    let grouped_field_names: Vec<_> = group.map(|idxs| format_ident!("e{}", idxs.iter().join(""))).collect();
                    ge.push(pq!(Self { #(#grouped_field_names: self.#grouped_field_names),*, ..::num_traits::Zero::zero()}));
                }

                ge
            },
            grades: {
                let groups = blade_indices.iter()
                    .group_by(|idxs| idxs.len());

                let mut g = vec![];

                for (grade, _) in &groups {
                    g.push(grade);
                }

                g
            },
            where_clause: to_where_clause([pq!(#scalar_type: #copy + #zero + ::core::cmp::PartialEq)]),
        },
        zero_trait_impl: IrMultivectorZeroTraitImpl {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            field_types: blade_types,
            where_clause: to_where_clause([pq!(#scalar_type: #zero)]),
        },
        debug_trait_impl: IrMultivectorDebugTraitImpl {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            where_clause: to_where_clause([pq!(#scalar_type: ::core::fmt::Debug)]),
        },
        clone_trait_impl: IrMultivectorCloneTraitImpl {
            ident: multivector_ident,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #copy)]),
        },
        copy_trait_impl: IrMultivectorCopyTraitImpl {
            ident: multivector_ident,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: #copy)]),
        },
        partial_eq_trait_impl: IrMultivectorPartialEqTraitImpl {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            where_clause: to_where_clause([pq!(#scalar_type: ::core::cmp::PartialEq)]),
        },
        eq_trait_impl: IrMultivectorEqTraitImpl {
            ident: multivector_ident,
            scalar_type,
            where_clause: to_where_clause([pq!(#scalar_type: ::core::cmp::Eq)]),
        },
        hash_trait_impl: IrMultivectorHashTraitImpl {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            where_clause: to_where_clause([pq!(#scalar_type: ::core::hash::Hash)]),
        },
        add_trait_impl: IrMultivectorAddTraitImpl {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            where_clause: to_where_clause([pq!(#scalar_type: #add<Output = #scalar_type>)]),
        },
        sub_trait_impl: IrMultivectorSubTraitImpl {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            where_clause: to_where_clause([pq!(#scalar_type: #sub<Output = #scalar_type>)]),
        },
        neg_trait_impl: IrMultivectorNegTraitImpl {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            where_clause: to_where_clause([pq!(#scalar_type: #neg<Output = #scalar_type>)]),
        },
        mul_trait_impl: IrMultivectorMulTraitImpl {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            where_clause: to_where_clause([
                pq!(#scalar_type: #copy + #zero + #const_signum + #add<Output = #scalar_type> + #mul<Output = #scalar_type> + #(#mul<#blade_types<#scalar_type>, Output = #blade_types<#scalar_type>>)+*),
            ]),
        },
        add_trait_impl_with_scalar_rhs: IrMultivectorAddTraitImplWithScalarRhs {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            where_clause: to_where_clause([pq!(#scalar_type: #add<Output = #scalar_type>)]),
        },
        add_trait_impl_with_scalar_lhs: IrMultivectorAddTraitImplWithScalarLhs {
            ident: multivector_ident,
            scalar_field_name,
            signed_primitive_types,
        },
        sub_trait_impl_with_scalar_rhs: IrMultivectorSubTraitImplWithScalarRhs {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            where_clause: to_where_clause([pq!(#scalar_type: #sub<Output = #scalar_type>)]),
        },
        sub_trait_impl_with_scalar_lhs: IrMultivectorSubTraitImplWithScalarLhs {
            ident: multivector_ident,
            scalar_field_name,
            signed_primitive_types,
        },
        mul_trait_impl_with_scalar_rhs: IrMultivectorMulTraitImplWithScalarRhs {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            where_clause: to_where_clause([pq!(#scalar_type: #mul<Output = #scalar_type> + #copy)]),
        },
        mul_trait_impl_with_scalar_lhs: IrMultivectorMulTraitImplWithScalarLhs {
            ident: multivector_ident,
            scalar_field_name,
            signed_primitive_types,
            field_names: blade_field_names,
        },
        add_trait_impls_with_blade: IrMultivectorAddTraitImplsWithBlade {
            ident: multivector_ident,
            scalar_type,
            field_names: blade_field_names,
            field_types: blade_types,
            where_clause: to_where_clause([pq!(#scalar_type: #add<Output = #scalar_type>)]),
        },
        sub_trait_impls_with_blade: IrMultivectorSubTraitImplsWithBlade {
            ident: multivector_ident,
            scalar_type,
            field_names: blade_field_names,
            field_types: blade_types,
            where_clause: to_where_clause([
                pq!(#scalar_type: #copy + #sub<Output = #scalar_type> + #neg<Output = #scalar_type>),
            ]),
        },
        mul_trait_impls_with_blade_rhs: IrMultivectorMulTraitImplsWithBladeRhs {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            operation_table: {
                let st = &scalar_type;
                blade_types.iter()
                    .map(|rhs| {
                        IrMultivectorMulTraitImplsWithBladeRhsTable {
                            rhs_type: rhs,
                            where_clause:
                                to_where_clause([pq!(#st: #copy + #zero + #const_signum + #add<Output = #st> + #mul<Output = #st> + #mul<#rhs<#st>, Output = #rhs<#st>>),]),
                        }
                    })
                .collect()
            },
        },
        mul_trait_impls_with_blade_lhs: IrMultivectorMulTraitImplsWithBladeLhs {
            ident: multivector_ident,
            scalar_type,
            scalar_field_name,
            field_names: blade_field_names,
            field_types: blade_types,
            where_clause: to_where_clause([
                pq!(#scalar_type: #copy + #zero + #const_signum + #add<Output = #scalar_type> + #mul<Output = #scalar_type>),
            ]),
        },
    }
}
