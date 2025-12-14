use quote::{quote, ToTokens};

use crate::algebra::ir::multivector::{
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
    Multivector,
};

pub fn codegen(ir: &Multivector) -> impl ToTokens {
    let multivector_trait_impl = codegen_multivector_trait_impl(&ir.multivector_trait_impl);

    let zero_trait_impl = codegen_zero_trait_impl(&ir.zero_trait_impl);

    let debug_trait_impl = codegen_debug_trait_impl(&ir.debug_trait_impl);
    let clone_trait_impl = codegen_clone_trait_impl(&ir.clone_trait_impl);
    let copy_trait_impl = codegen_copy_trait_impl(&ir.copy_trait_impl);
    let partial_eq_trait_impl = codegen_partial_eq_trait_impl(&ir.partial_eq_trait_impl);
    let eq_trait_impl = codegen_eq_trait_impl(&ir.eq_trait_impl);
    let hash_trait_impl = codegen_hash_trait_impl(&ir.hash_trait_impl);

    let add_trait_impl = codegen_add_trait_impl(&ir.add_trait_impl);
    let sub_trait_impl = codegen_sub_trait_impl(&ir.sub_trait_impl);
    let neg_trait_impl = codegen_neg_trait_impl(&ir.neg_trait_impl);
    let mul_trait_impl = codegen_mul_trait_impl(&ir.mul_trait_impl);
    let add_trait_impl_with_scalar_rhs =
        codegen_add_trait_impl_with_scalar_rhs(&ir.add_trait_impl_with_scalar_rhs);
    let add_trait_impl_with_scalar_lhs =
        codegen_add_trait_impl_with_scalar_lhs(&ir.add_trait_impl_with_scalar_lhs);
    let sub_trait_impl_with_scalar_rhs =
        codegen_sub_trait_impl_with_scalar_rhs(&ir.sub_trait_impl_with_scalar_rhs);
    let sub_trait_impl_with_scalar_lhs =
        codegen_sub_trait_impl_with_scalar_lhs(&ir.sub_trait_impl_with_scalar_lhs);
    let mul_trait_impl_with_scalar_rhs =
        codegen_mul_trait_impl_with_scalar_rhs(&ir.mul_trait_impl_with_scalar_rhs);
    let mul_trait_impl_with_scalar_lhs =
        codegen_mul_trait_impl_with_scalar_lhs(&ir.mul_trait_impl_with_scalar_lhs);
    let add_trait_impls_with_blade =
        codegen_add_trait_impls_with_blade(&ir.add_trait_impls_with_blade);
    let sub_trait_impls_with_blade =
        codegen_sub_trait_impls_with_blade(&ir.sub_trait_impls_with_blade);
    let mul_trait_impls_with_blade_rhs =
        codegen_mul_trait_impls_with_blade_rhs(&ir.mul_trait_impls_with_blade_rhs);

    let mul_trait_impls_with_blade_lhs =
        codegen_mul_trait_impls_with_blade_lhs(&ir.mul_trait_impls_with_blade_lhs);

    quote! {
        #multivector_trait_impl

        #zero_trait_impl

        #debug_trait_impl
        #clone_trait_impl
        #copy_trait_impl
        #partial_eq_trait_impl
        #eq_trait_impl
        #hash_trait_impl

        #add_trait_impl
        #sub_trait_impl
        #neg_trait_impl
        #mul_trait_impl
        #add_trait_impl_with_scalar_rhs
        #add_trait_impl_with_scalar_lhs
        #sub_trait_impl_with_scalar_rhs
        #sub_trait_impl_with_scalar_lhs
        #mul_trait_impl_with_scalar_rhs
        #mul_trait_impl_with_scalar_lhs
        #add_trait_impls_with_blade
        #sub_trait_impls_with_blade
        #mul_trait_impls_with_blade_rhs
        #mul_trait_impls_with_blade_lhs
    }
}

fn codegen_multivector_trait_impl(ir: &IrMultivectorTraitImpl) -> impl ToTokens {
    let IrMultivectorTraitImpl { ident, scalar_type, scalar_field_name, grade_exprs, gproj_exprs, grades, where_clause } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::vega_internal::Multivector for #ident<#scalar_type> #where_clause {
            #[inline]
            fn grade(&self) -> Option<usize> {
                use ::num_traits::Zero;

                let blade_grades = [
                    #(
                        ::core::convert::Into::<usize>::into(#grade_exprs) * #grades,
                    )*
                ].into_iter().filter(|g| *g != 0).chain((self.#scalar_field_name != #scalar_type::zero()).then_some(0).into_iter()).collect::<Vec<_>>();

                match blade_grades.len() {
                    1 => Some(blade_grades[0]),
                    _ => None,
                }
            }

            #[inline]
            fn gproj(&self, k: usize) -> Self {
                match k {
                    #(
                        #grades => #gproj_exprs,
                    )*
                    0 => Self {
                        #scalar_field_name: self.#scalar_field_name,
                        ..::num_traits::Zero::zero()
                    },
                    _ => ::num_traits::Zero::zero(),
                }
            }
        }
    }
}

fn codegen_zero_trait_impl(ir: &IrMultivectorZeroTraitImpl) -> impl ToTokens {
    let IrMultivectorZeroTraitImpl {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        field_types,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::num_traits::Zero for #ident<#scalar_type> #where_clause {
            #[inline]
            fn zero() -> Self {
                Self {
                    #scalar_field_name: #scalar_type::zero(),
                    #(
                        #field_names: #field_types::zero(),
                    )*
                }
            }

            #[inline]
            fn is_zero(&self) -> bool {
                self.#scalar_field_name.is_zero() && #( self.#field_names.is_zero() )&&*
            }
        }
    }
}

fn codegen_debug_trait_impl(ir: &IrMultivectorDebugTraitImpl) -> impl ToTokens {
    let IrMultivectorDebugTraitImpl {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::fmt::Debug for #ident<#scalar_type> #where_clause {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let fields: Vec<_> = vec![#( format!("{}: {:?}", stringify!(#field_names), &self.#field_names) ),*];
                ::core::write!(f, "{} {{ {}: {:?}, {} }}", stringify!(#ident), stringify!(#scalar_field_name), &self.#scalar_field_name, fields.join(", "))
            }
        }
    }
}

fn codegen_clone_trait_impl(ir: &IrMultivectorCloneTraitImpl) -> impl ToTokens {
    let IrMultivectorCloneTraitImpl {
        ident,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::clone::Clone for #ident<#scalar_type> #where_clause {
            #[inline]
            fn clone(&self) -> Self {
                *self
            }
        }
    }
}

fn codegen_copy_trait_impl(ir: &IrMultivectorCopyTraitImpl) -> impl ToTokens {
    let IrMultivectorCopyTraitImpl {
        ident,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::marker::Copy for #ident<#scalar_type> #where_clause {}
    }
}

fn codegen_partial_eq_trait_impl(ir: &IrMultivectorPartialEqTraitImpl) -> impl ToTokens {
    let IrMultivectorPartialEqTraitImpl {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::cmp::PartialEq for #ident<#scalar_type> #where_clause {
            #[inline]
            fn eq(&self, rhs: &Self) -> bool {
                self.#scalar_field_name.eq(&rhs.#scalar_field_name) && #( self.#field_names.eq(&rhs.#field_names) )&&*
            }
        }
    }
}

fn codegen_eq_trait_impl(ir: &IrMultivectorEqTraitImpl) -> impl ToTokens {
    let IrMultivectorEqTraitImpl {
        ident,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::cmp::Eq for #ident<#scalar_type> #where_clause {}
    }
}

fn codegen_hash_trait_impl(ir: &IrMultivectorHashTraitImpl) -> impl ToTokens {
    let IrMultivectorHashTraitImpl {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::hash::Hash for #ident<#scalar_type> #where_clause {
            #[inline]
            fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                self.#scalar_field_name.hash(state);
                #(
                    self.#field_names.hash(state);
                )*
            }
        }
    }
}

fn codegen_add_trait_impl(ir: &IrMultivectorAddTraitImpl) -> impl ToTokens {
    let IrMultivectorAddTraitImpl {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::ops::Add for #ident<#scalar_type> #where_clause {
            type Output = Self;

            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self {
                    #scalar_field_name: self.#scalar_field_name + rhs.#scalar_field_name,
                    #(
                        #field_names: self.#field_names + rhs.#field_names,
                        )*
                }
            }
        }
    }
}

fn codegen_sub_trait_impl(ir: &IrMultivectorSubTraitImpl) -> impl ToTokens {
    let IrMultivectorSubTraitImpl {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::ops::Sub for #ident<#scalar_type> #where_clause {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self {
                Self {
                    #scalar_field_name: self.#scalar_field_name - rhs.#scalar_field_name,
                    #(
                        #field_names: self.#field_names - rhs.#field_names,
                        )*
                }
            }
        }
    }
}

fn codegen_neg_trait_impl(ir: &IrMultivectorNegTraitImpl) -> impl ToTokens {
    let IrMultivectorNegTraitImpl {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::ops::Neg for #ident<#scalar_type> #where_clause {
            type Output = Self;

            #[inline]
            fn neg(self) -> Self {
                Self {
                    #scalar_field_name: -self.#scalar_field_name,
                    #(
                        #field_names: -self.#field_names,
                        )*
                }
            }
        }
    }
}

fn codegen_mul_trait_impl(ir: &IrMultivectorMulTraitImpl) -> impl ToTokens {
    let IrMultivectorMulTraitImpl {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::ops::Mul for #ident<#scalar_type> #where_clause {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: Self) -> Self {
                self * rhs.#scalar_field_name + #(self * rhs.#field_names)+*
            }
        }
    }
}

fn codegen_add_trait_impl_with_scalar_rhs(
    ir: &IrMultivectorAddTraitImplWithScalarRhs,
) -> impl ToTokens {
    let IrMultivectorAddTraitImplWithScalarRhs {
        ident,
        scalar_type,
        scalar_field_name,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::ops::Add<#scalar_type> for #ident<#scalar_type> #where_clause {
            type Output = Self;

            #[inline]
            fn add(self, rhs: #scalar_type) -> Self {
                Self {
                    #scalar_field_name: self.#scalar_field_name + rhs,
                    ..self
                }
            }
        }
    }
}

fn codegen_add_trait_impl_with_scalar_lhs(
    ir: &IrMultivectorAddTraitImplWithScalarLhs,
) -> impl ToTokens {
    let IrMultivectorAddTraitImplWithScalarLhs {
        ident,
        signed_primitive_types,
        scalar_field_name,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl ::core::ops::Add<#ident<#signed_primitive_types>> for #signed_primitive_types {
                type Output = #ident<#signed_primitive_types>;

                #[inline]
                fn add(self, rhs: #ident<#signed_primitive_types>) -> Self::Output {
                    #ident {
                        #scalar_field_name: self + rhs.#scalar_field_name,
                        ..rhs
                    }
                }
            }
        )*
    }
}

fn codegen_sub_trait_impl_with_scalar_rhs(
    ir: &IrMultivectorSubTraitImplWithScalarRhs,
) -> impl ToTokens {
    let IrMultivectorSubTraitImplWithScalarRhs {
        ident,
        scalar_type,
        scalar_field_name,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::ops::Sub<#scalar_type> for #ident<#scalar_type> #where_clause {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: #scalar_type) -> Self {
                Self {
                    #scalar_field_name: self.#scalar_field_name - rhs,
                    ..self
                }
            }
        }
    }
}

fn codegen_sub_trait_impl_with_scalar_lhs(
    ir: &IrMultivectorSubTraitImplWithScalarLhs,
) -> impl ToTokens {
    let IrMultivectorSubTraitImplWithScalarLhs {
        ident,
        signed_primitive_types,
        scalar_field_name,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl ::core::ops::Sub<#ident<#signed_primitive_types>> for #signed_primitive_types {
                type Output = #ident<#signed_primitive_types>;

                #[inline]
                fn sub(self, rhs: #ident<#signed_primitive_types>) -> Self::Output {
                    #ident {
                        #scalar_field_name: self - rhs.#scalar_field_name,
                        ..(-rhs)
                    }
                }
            }
        )*
    }
}

fn codegen_mul_trait_impl_with_scalar_rhs(
    ir: &IrMultivectorMulTraitImplWithScalarRhs,
) -> impl ToTokens {
    let IrMultivectorMulTraitImplWithScalarRhs {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        where_clause,
    } = ir;

    quote! {
        #[automatically_derived]
        impl<#scalar_type> ::core::ops::Mul<#scalar_type> for #ident<#scalar_type> #where_clause {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: #scalar_type) -> Self {
                Self {
                    #scalar_field_name: self.#scalar_field_name * rhs,
                    #(
                        #field_names: self.#field_names * rhs,
                    )*
                }
            }
        }
    }
}

fn codegen_mul_trait_impl_with_scalar_lhs(
    ir: &IrMultivectorMulTraitImplWithScalarLhs,
) -> impl ToTokens {
    let IrMultivectorMulTraitImplWithScalarLhs {
        ident,
        signed_primitive_types,
        scalar_field_name,
        field_names,
    } = ir;

    let mut impls = vec![];
    for scalar_type in *signed_primitive_types {
        impls.push(quote! {
            #[automatically_derived]
            impl ::core::ops::Mul<#ident<#scalar_type>> for #scalar_type {
                type Output = #ident<#scalar_type>;

                #[inline]
                fn mul(self, rhs: #ident<#scalar_type>) -> Self::Output {
                    #ident {
                        #scalar_field_name: self * rhs.#scalar_field_name,
                        #(
                            #field_names: self * rhs.#field_names,
                        )*
                    }
                }
            }
        });
    }

    quote!(#(#impls)*)
}

fn codegen_add_trait_impls_with_blade(ir: &IrMultivectorAddTraitImplsWithBlade) -> impl ToTokens {
    let IrMultivectorAddTraitImplsWithBlade {
        ident,
        scalar_type,
        field_names,
        field_types,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Add<#field_types<#scalar_type>> for #ident<#scalar_type> #where_clause {
                type Output = Self;

                #[inline]
                fn add(self, rhs: #field_types<#scalar_type>) -> Self {
                    Self {
                        #field_names: self.#field_names + rhs,
                        ..self
                    }
                }
            }

            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Add<#ident<#scalar_type>> for #field_types<#scalar_type> #where_clause {
                type Output = #ident<#scalar_type>;

                #[inline]
                fn add(self, rhs: #ident<#scalar_type>) -> Self::Output {
                    #ident {
                        #field_names: self + rhs.#field_names,
                        ..rhs
                    }
                }
            }
        )*
    }
}

fn codegen_sub_trait_impls_with_blade(ir: &IrMultivectorSubTraitImplsWithBlade) -> impl ToTokens {
    let IrMultivectorSubTraitImplsWithBlade {
        ident,
        scalar_type,
        field_names,
        field_types,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Sub<#field_types<#scalar_type>> for #ident<#scalar_type> #where_clause {
                type Output = Self;

                #[inline]
                fn sub(self, rhs: #field_types<#scalar_type>) -> Self {
                    Self {
                        #field_names: self.#field_names - rhs,
                        ..self
                    }
                }
            }

            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Sub<#ident<#scalar_type>> for #field_types<#scalar_type> #where_clause {
                type Output = #ident<#scalar_type>;

                #[inline]
                fn sub(self, rhs: #ident<#scalar_type>) -> Self::Output {
                    #ident {
                        #field_names: self - rhs.#field_names,
                        ..(-rhs)
                    }
                }
            }
        )*
    }
}

fn codegen_mul_trait_impls_with_blade_rhs(
    ir: &IrMultivectorMulTraitImplsWithBladeRhs,
) -> impl ToTokens {
    let IrMultivectorMulTraitImplsWithBladeRhs {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        operation_table,
    } = ir;

    let mut impls = vec![];
    for IrMultivectorMulTraitImplsWithBladeRhsTable {
        rhs_type,
        where_clause,
    } in operation_table
    {
        impls.push(quote! {
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Mul<#rhs_type<#scalar_type>> for #ident<#scalar_type> #where_clause {
                type Output = Self;

                #[inline]
                fn mul(self, rhs: #rhs_type<#scalar_type>) -> Self {
                    self.#scalar_field_name * rhs + #( self.#field_names * rhs )+*
                }
            }
        });
    }

    quote!(#(#impls)*)
}

fn codegen_mul_trait_impls_with_blade_lhs(
    ir: &IrMultivectorMulTraitImplsWithBladeLhs,
) -> impl ToTokens {
    let IrMultivectorMulTraitImplsWithBladeLhs {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        field_types,
        where_clause,
    } = ir;

    let mut impls = vec![];

    for lhs_type in *field_types {
        impls.push(quote! {
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Mul<#ident<#scalar_type>> for #lhs_type<#scalar_type> #where_clause {
                type Output = #ident<#scalar_type>;

                #[inline]
                fn mul(self, rhs: #ident<#scalar_type>) -> Self::Output {
                    self * rhs.#scalar_field_name + #( self * rhs.#field_names )+*
                }
            }
        });
    }

    quote!(#(#impls)*)
}
