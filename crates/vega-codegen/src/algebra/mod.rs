use analyze::analyze;
use codegen::codegen;
use input::Algebra;
use lower::lower;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::parse_macro_input;

mod analyze;
mod codegen;
mod input;
mod ir;
mod lower;
mod model;

/// The pipeline function decouples the actual implementation from the macro definition
pub fn pipeline(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Algebra);
    let model = analyze(&input);
    let ir = lower(&model);
    let rust = codegen(&ir);

    TokenStream::from(rust.to_token_stream())
}
