use crate::algebra::codegen::blades::codegen as codegen_blades;
use crate::algebra::codegen::multivector::codegen as codegen_multivector;

use super::ir::Algebra;
use super::ir::blades::IrNewtypeStruct;
use super::ir::multivector::IrNamedStruct;
use quote::ToTokens;
use quote::quote;

mod blades;
mod multivector;

pub fn codegen(ir: &Algebra) -> impl ToTokens {
    let module_name = &ir.module_name;

    let blades = codegen_newtype_struct(&ir.blades.struct_defs);
    let multivector = codegen_named_struct(&ir.multivector.struct_def);

    let blades_impls = codegen_blades(&ir.blades);
    let multivector_impls = codegen_multivector(&ir.multivector);

    quote! {
        pub use #module_name::*;

        #[doc = "Defines a Geometric Algebra"]
        pub mod #module_name {
            #blades
            #multivector

            mod impls {
                mod blades {
                    use super::super::*;

                    #blades_impls
                }

                mod multivector {
                    use super::super::*;

                    #multivector_impls
                }
            }
        }
    }
}

fn codegen_newtype_struct(ir: &IrNewtypeStruct) -> impl ToTokens {
    let IrNewtypeStruct { idents, scalar_type } = ir;

    quote! {
        #(
            #[doc = "A blade of a multivector in a geometric algebra. Blades may be vectors or k-vectors."]
            #[repr(transparent)]
            pub struct #idents<#scalar_type>(pub #scalar_type);
        )*
    }
}

fn codegen_named_struct(ir: &IrNamedStruct) -> impl ToTokens {
    let IrNamedStruct {
        ident,
        scalar_type,
        scalar_field_name,
        field_names,
        field_types,
    } = ir;

    quote! {
        #[doc = "A multivector type that supports addition, subtraction, negation, and multiplication with itself, blades, and scalars."]
        pub struct #ident<#scalar_type> {
            #[doc = "The scalar field"]
            pub #scalar_field_name: #scalar_type,
            #(
                #[doc = "A blade (vector or k-vector)"]
                pub #field_names: #field_types<#scalar_type>,
            )*
        }
    }
}
