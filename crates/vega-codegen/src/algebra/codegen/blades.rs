use quote::{ToTokens, quote};

use crate::algebra::ir::blades::{
    Blades, IrBladeAddTraitImpls, IrBladeAddTraitImplsWithOtherBlade, IrBladeAddTraitImplsWithOtherBladeTable,
    IrBladeAddTraitImplsWithScalarLhs, IrBladeAddTraitImplsWithScalarRhs, IrBladeCloneTraitImpls,
    IrBladeCopyTraitImpls, IrBladeDebugTraitImpls, IrBladeEqTraitImpls, IrBladeHashTraitImpls, IrBladeMulTraitImpls,
    IrBladeMulTraitImplsWithOtherBlade, IrBladeMulTraitImplsWithOtherBladeTable, IrBladeMulTraitImplsWithScalarLhs,
    IrBladeMulTraitImplsWithScalarRhs, IrBladeNegTraitImpls, IrBladeOrdTraitImpls, IrBladePartialEqTraitImpls,
    IrBladePartialOrdTraitImpls, IrBladeSubTraitImpls, IrBladeSubTraitImplsWithOtherBlade,
    IrBladeSubTraitImplsWithOtherBladeTable, IrBladeSubTraitImplsWithScalarLhs, IrBladeSubTraitImplsWithScalarRhs,
    IrBladeTraitImpls, IrBladeZeroTraitImpls,
};

pub fn codegen(ir: &Blades) -> impl ToTokens {
    let positive_blade_trait_impls = codegen_blade_trait_impls(&ir.positive_blade_trait_impls);
    let negative_blade_trait_impls = codegen_blade_trait_impls(&ir.negative_blade_trait_impls);
    let zero_blade_trait_impls = codegen_blade_trait_impls(&ir.zero_blade_trait_impls);
    let higher_order_blade_trait_impls = codegen_blade_trait_impls(&ir.higher_order_blade_trait_impls);

    let zero_trait_impls = codegen_zero_trait_impls(&ir.zero_trait_impls);

    let debug_trait_impls = codegen_debug_trait_impls(&ir.debug_trait_impls);
    let clone_trait_impls = codegen_clone_trait_impls(&ir.clone_trait_impls);
    let copy_trait_impls = codegen_copy_trait_impls(&ir.copy_trait_impls);
    let partial_eq_trait_impls = codegen_partial_eq_trait_impls(&ir.partial_eq_trait_impls);
    let eq_trait_impls = codegen_eq_trait_impls(&ir.eq_trait_impls);
    let partial_ord_trait_impls = codegen_partial_ord_trait_impls(&ir.partial_ord_trait_impls);
    let ord_trait_impls = codegen_ord_trait_impls(&ir.ord_trait_impls);
    let hash_trait_impls = codegen_hash_trait_impls(&ir.hash_trait_impls);

    let add_trait_impls = codegen_add_trait_impls(&ir.add_trait_impls);
    let sub_trait_impls = codegen_sub_trait_impls(&ir.sub_trait_impls);
    let neg_trait_impls = codegen_neg_trait_impls(&ir.neg_trait_impls);
    let mul_trait_impls = codegen_mul_trait_impls(&ir.mul_trait_impls);
    let add_trait_impls_with_scalar_rhs = codegen_add_trait_impls_with_scalar_rhs(&ir.add_trait_impls_with_scalar_rhs);
    let add_trait_impls_with_scalar_lhs = codegen_add_trait_impls_with_scalar_lhs(&ir.add_trait_impls_with_scalar_lhs);
    let sub_trait_impls_with_scalar_rhs = codegen_sub_trait_impls_with_scalar_rhs(&ir.sub_trait_impls_with_scalar_rhs);
    let sub_trait_impls_with_scalar_lhs = codegen_sub_trait_impls_with_scalar_lhs(&ir.sub_trait_impls_with_scalar_lhs);
    let mul_trait_impls_with_scalar_rhs = codegen_mul_trait_impls_with_scalar_rhs(&ir.mul_trait_impls_with_scalar_rhs);
    let mul_trait_impls_with_scalar_lhs = codegen_mul_trait_impls_with_scalar_lhs(&ir.mul_trait_impls_with_scalar_lhs);
    let add_trait_impls_with_other_blade =
        codegen_add_trait_impls_with_other_blade(&ir.add_trait_impls_with_other_blade);
    let sub_trait_impls_with_other_blade =
        codegen_sub_trait_impls_with_other_blade(&ir.sub_trait_impls_with_other_blade);
    let mul_trait_impls_with_other_blade =
        codegen_mul_trait_impls_with_other_blade(&ir.mul_trait_impls_with_other_blade);

    quote! {
        #positive_blade_trait_impls
        #negative_blade_trait_impls
        #zero_blade_trait_impls
        #higher_order_blade_trait_impls

        #zero_trait_impls

        #debug_trait_impls
        #clone_trait_impls
        #copy_trait_impls
        #partial_eq_trait_impls
        #eq_trait_impls
        #partial_ord_trait_impls
        #ord_trait_impls
        #hash_trait_impls

        #add_trait_impls
        #sub_trait_impls
        #neg_trait_impls
        #mul_trait_impls
        #add_trait_impls_with_scalar_rhs
        #add_trait_impls_with_scalar_lhs
        #sub_trait_impls_with_scalar_rhs
        #sub_trait_impls_with_scalar_lhs
        #mul_trait_impls_with_scalar_rhs
        #mul_trait_impls_with_scalar_lhs
        #add_trait_impls_with_other_blade
        #sub_trait_impls_with_other_blade
        #mul_trait_impls_with_other_blade
    }
}

fn codegen_blade_trait_impls(ir: &IrBladeTraitImpls) -> impl ToTokens {
    let IrBladeTraitImpls {
        idents,
        unit_square_values,
        grades,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::vega_internal::Blade<#scalar_type> for #idents<#scalar_type> #where_clause {
                const UNIT_SQUARE_VALUE: #scalar_type = #unit_square_values;
                const GRADE: usize = #grades;
            }
        )*
    }
}

fn codegen_zero_trait_impls(ir: &IrBladeZeroTraitImpls) -> impl ToTokens {
    let IrBladeZeroTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::num_traits::Zero for #idents<#scalar_type> #where_clause {
                #[inline]
                fn zero() -> Self {
                    Self(#scalar_type::zero())
                }

                #[inline]
                fn is_zero(&self) -> bool {
                    self.0.is_zero()
                }
            }
        )*
    }
}

fn codegen_debug_trait_impls(ir: &IrBladeDebugTraitImpls) -> impl ToTokens {
    let IrBladeDebugTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::fmt::Debug for #idents<#scalar_type> #where_clause {
                #[inline]
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    ::core::write!(f, "{}({:?})", stringify!(#idents), self.0)
                }
            }
        )*
    }
}

fn codegen_clone_trait_impls(ir: &IrBladeCloneTraitImpls) -> impl ToTokens {
    let IrBladeCloneTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::clone::Clone for #idents<#scalar_type> #where_clause {
                #[inline]
                fn clone(&self) -> Self {
                    *self
                }
            }
        )*
    }
}

fn codegen_copy_trait_impls(ir: &IrBladeCopyTraitImpls) -> impl ToTokens {
    let IrBladeCopyTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::marker::Copy for #idents<#scalar_type> #where_clause {}
        )*
    }
}

fn codegen_partial_eq_trait_impls(ir: &IrBladePartialEqTraitImpls) -> impl ToTokens {
    let IrBladePartialEqTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::cmp::PartialEq for #idents<#scalar_type> #where_clause {
                #[inline]
                fn eq(&self, rhs: &Self) -> bool {
                    self.0.eq(&rhs.0)
                }
            }
        )*
    }
}

fn codegen_eq_trait_impls(ir: &IrBladeEqTraitImpls) -> impl ToTokens {
    let IrBladeEqTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::cmp::Eq for #idents<#scalar_type> #where_clause {}
        )*
    }
}

fn codegen_partial_ord_trait_impls(ir: &IrBladePartialOrdTraitImpls) -> impl ToTokens {
    let IrBladePartialOrdTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::cmp::PartialOrd for #idents<#scalar_type> #where_clause {
                #[inline]
                fn partial_cmp(&self, rhs: &Self) -> Option<::core::cmp::Ordering> {
                    self.0.partial_cmp(&rhs.0)
                }
            }
        )*
    }
}

fn codegen_ord_trait_impls(ir: &IrBladeOrdTraitImpls) -> impl ToTokens {
    let IrBladeOrdTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::cmp::Ord for #idents<#scalar_type> #where_clause {
                #[inline]
                fn cmp(&self, rhs: &Self) -> ::core::cmp::Ordering {
                    self.0.cmp(&rhs.0)
                }
            }
        )*
    }
}

fn codegen_hash_trait_impls(ir: &IrBladeHashTraitImpls) -> impl ToTokens {
    let IrBladeHashTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::hash::Hash for #idents<#scalar_type> #where_clause {
                #[inline]
                fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
                    self.0.hash(state)
                }
            }
        )*
    }
}

fn codegen_add_trait_impls(ir: &IrBladeAddTraitImpls) -> impl ToTokens {
    let IrBladeAddTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Add for #idents<#scalar_type> #where_clause {
                type Output = Self;

                #[inline]
                fn add(self, rhs: Self) -> Self {
                    Self(self.0 + rhs.0)
                }
            }
        )*
    }
}

fn codegen_sub_trait_impls(ir: &IrBladeSubTraitImpls) -> impl ToTokens {
    let IrBladeSubTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Sub for #idents<#scalar_type> #where_clause {
                type Output = Self;

                #[inline]
                fn sub(self, rhs: Self) -> Self {
                    Self(self.0 - rhs.0)
                }
            }
        )*
    }
}

fn codegen_neg_trait_impls(ir: &IrBladeNegTraitImpls) -> impl ToTokens {
    let IrBladeNegTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Neg for #idents<#scalar_type> #where_clause {
                type Output = Self;

                #[inline]
                fn neg(self) -> Self {
                    Self(-self.0)
                }
            }
        )*
    }
}

fn codegen_mul_trait_impls(ir: &IrBladeMulTraitImpls) -> impl ToTokens {
    let IrBladeMulTraitImpls {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Mul for #idents<#scalar_type> #where_clause {
                type Output = #scalar_type;

                #[inline]
                fn mul(self, rhs: Self) -> Self::Output {
                    <Self as ::vega_internal::Blade<#scalar_type>>::UNIT_SQUARE_VALUE * self.0 * rhs.0
                }
            }
        )*
    }
}

fn codegen_add_trait_impls_with_scalar_rhs(ir: &IrBladeAddTraitImplsWithScalarRhs) -> impl ToTokens {
    let IrBladeAddTraitImplsWithScalarRhs {
        multivector_ident: ident,
        scalar_type,
        where_clause,
        field_names,
        field_types,
        scalar_field_name,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Add<#scalar_type> for #field_types<#scalar_type> #where_clause {
                type Output = #ident<#scalar_type>;

                #[inline]
                fn add(self, rhs: #scalar_type) -> Self::Output {
                    #ident {
                        #scalar_field_name: rhs,
                        #field_names: self,
                        ..::num_traits::Zero::zero()
                    }
                }
            }
        )*
    }
}

fn codegen_add_trait_impls_with_scalar_lhs(ir: &IrBladeAddTraitImplsWithScalarLhs) -> impl ToTokens {
    let IrBladeAddTraitImplsWithScalarLhs {
        multivector_ident: ident,
        signed_primitive_types,
        field_names,
        field_types,
        scalar_field_name,
    } = ir;

    let mut impls = Vec::new();

    for (field_name, field_type) in field_names.iter().zip(field_types.iter()) {
        impls.push(quote! {
            #(
                #[automatically_derived]
                impl ::core::ops::Add<#field_type<#signed_primitive_types>> for #signed_primitive_types {
                    type Output = #ident<#signed_primitive_types>;

                    #[inline]
                    fn add(self, rhs: #field_type<#signed_primitive_types>) -> Self::Output {
                        #ident {
                            #scalar_field_name: self,
                            #field_name: rhs,
                            ..::num_traits::Zero::zero()
                        }
                    }
                }
            )*
        });
    }

    quote!(#(#impls)*)
}

fn codegen_sub_trait_impls_with_scalar_rhs(ir: &IrBladeSubTraitImplsWithScalarRhs) -> impl ToTokens {
    let IrBladeSubTraitImplsWithScalarRhs {
        multivector_ident: ident,
        scalar_type,
        where_clause,
        field_names,
        field_types,
        scalar_field_name,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Sub<#scalar_type> for #field_types<#scalar_type> #where_clause {
                type Output = #ident<#scalar_type>;

                #[inline]
                fn sub(self, rhs: #scalar_type) -> Self::Output {
                    #ident {
                        #scalar_field_name: -rhs,
                        #field_names: self,
                        ..::num_traits::Zero::zero()
                    }
                }
            }
        )*
    }
}

fn codegen_sub_trait_impls_with_scalar_lhs(ir: &IrBladeSubTraitImplsWithScalarLhs) -> impl ToTokens {
    let IrBladeSubTraitImplsWithScalarLhs {
        multivector_ident: ident,
        signed_primitive_types,
        field_names,
        field_types,
        scalar_field_name,
    } = ir;

    let mut impls = Vec::new();

    for (field_name, field_type) in field_names.iter().zip(field_types.iter()) {
        impls.push(quote! {
            #(
                #[automatically_derived]
                impl ::core::ops::Sub<#field_type<#signed_primitive_types>> for #signed_primitive_types {
                    type Output = #ident<#signed_primitive_types>;

                    #[inline]
                    fn sub(self, rhs: #field_type<#signed_primitive_types>) -> Self::Output {
                        #ident {
                            #scalar_field_name: self,
                            #field_name: -rhs,
                            ..::num_traits::Zero::zero()
                        }
                    }
                }
            )*
        });
    }

    quote!(#(#impls)*)
}

fn codegen_mul_trait_impls_with_scalar_rhs(ir: &IrBladeMulTraitImplsWithScalarRhs) -> impl ToTokens {
    let IrBladeMulTraitImplsWithScalarRhs {
        idents,
        scalar_type,
        where_clause,
    } = ir;

    quote! {
        #(
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Mul<#scalar_type> for #idents<#scalar_type> #where_clause {
                type Output = Self;

                #[inline]
                fn mul(self, rhs: #scalar_type) -> Self {
                    Self(self.0 * rhs)
                }
            }
        )*
    }
}

fn codegen_mul_trait_impls_with_scalar_lhs(ir: &IrBladeMulTraitImplsWithScalarLhs) -> impl ToTokens {
    let IrBladeMulTraitImplsWithScalarLhs {
        idents,
        signed_primitive_types,
    } = ir;

    let mut impls = Vec::new();

    for ident in *idents {
        impls.push(quote! {
            #(
                #[automatically_derived]
                impl ::core::ops::Mul<#ident<#signed_primitive_types>> for #signed_primitive_types {
                    type Output = #ident<#signed_primitive_types>;

                    #[inline]
                    fn mul(self, rhs: #ident<#signed_primitive_types>) -> Self::Output {
                        #ident(self * rhs.0)
                    }
                }
            )*
        });
    }

    quote!(#(#impls)*)
}

fn codegen_add_trait_impls_with_other_blade(ir: &IrBladeAddTraitImplsWithOtherBlade) -> impl ToTokens {
    let IrBladeAddTraitImplsWithOtherBlade {
        multivector_ident,
        scalar_type,
        operation_table,
        where_clause,
    } = ir;

    let mut impls = vec![];

    for IrBladeAddTraitImplsWithOtherBladeTable {
        lhs_field_name,
        lhs_field_type,
        rhs_field_name,
        rhs_field_type,
    } in operation_table
    {
        impls.push(quote! {
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Add<#rhs_field_type<#scalar_type>> for #lhs_field_type<#scalar_type> #where_clause {
                type Output = #multivector_ident<#scalar_type>;

                #[inline]
                fn add(self, rhs: #rhs_field_type<#scalar_type>) -> Self::Output {
                    #multivector_ident {
                        #lhs_field_name: self,
                        #rhs_field_name: rhs,
                        ..::num_traits::Zero::zero()
                    }
                }
            }
        });
    }

    quote!(#(#impls)*)
}

fn codegen_sub_trait_impls_with_other_blade(ir: &IrBladeSubTraitImplsWithOtherBlade) -> impl ToTokens {
    let IrBladeSubTraitImplsWithOtherBlade {
        multivector_ident,
        scalar_type,
        operation_table,
        where_clause,
    } = ir;

    let mut impls = vec![];

    for IrBladeSubTraitImplsWithOtherBladeTable {
        lhs_field_name,
        lhs_field_type,
        rhs_field_name,
        rhs_field_type,
    } in operation_table
    {
        impls.push(quote! {
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Sub<#rhs_field_type<#scalar_type>> for #lhs_field_type<#scalar_type> #where_clause {
                type Output = #multivector_ident<#scalar_type>;

                #[inline]
                fn sub(self, rhs: #rhs_field_type<#scalar_type>) -> Self::Output {
                    #multivector_ident {
                        #lhs_field_name: self,
                        #rhs_field_name: -rhs,
                        ..::num_traits::Zero::zero()
                    }
                }
            }
        });
    }

    quote!(#(#impls)*)
}

fn codegen_mul_trait_impls_with_other_blade(ir: &IrBladeMulTraitImplsWithOtherBlade) -> impl ToTokens {
    let IrBladeMulTraitImplsWithOtherBlade {
        scalar_type,
        operation_table,
        where_clause,
    } = ir;

    let mut impls = vec![];

    for IrBladeMulTraitImplsWithOtherBladeTable {
        lhs_field_type,
        rhs_field_type,
        output_type,
        output_signum,
    } in operation_table
    {
        impls.push(quote! {
            #[automatically_derived]
            impl<#scalar_type> ::core::ops::Mul<#rhs_field_type<#scalar_type>> for #lhs_field_type<#scalar_type> #where_clause {
                type Output = #output_type<#scalar_type>;

                #[inline]
                fn mul(self, rhs: #rhs_field_type<#scalar_type>) -> Self::Output {
                    #output_type(#output_signum * self.0 * rhs.0)
                }
            }
        });
    }

    quote!(#(#impls)*)
}
