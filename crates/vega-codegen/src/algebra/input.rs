use proc_macro2::Span;
use syn::{LitInt, parse_quote};

/// Restricts the input [`algebra`](crate::algebra!()) to between one and three literal integers
#[derive(Debug)]
pub struct Algebra {
    /// Refers to the location of the input to the [`algebra`](crate::algebra!()) macro
    pub span: Span,
    /// Holds the number of dimensions that square to one
    pub positive: LitInt,
    /// Holds the number of dimensions that square to minus one
    pub negative: LitInt,
    /// Holds the number of dimensions that square to zero
    pub zero: LitInt,
}

impl syn::parse::Parse for Algebra {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let punct = syn::punctuated::Punctuated::<LitInt, syn::token::Comma>::parse_separated_nonempty(input)?;

        match punct.len() {
            1 => Ok(Algebra {
                span,
                positive: punct[0].clone(),
                negative: parse_quote!(0),
                zero: parse_quote!(0),
            }),
            2 => Ok(Algebra {
                span,
                positive: punct[0].clone(),
                negative: punct[1].clone(),
                zero: parse_quote!(0),
            }),
            3 => Ok(Algebra {
                span,
                positive: punct[0].clone(),
                negative: punct[1].clone(),
                zero: punct[2].clone(),
            }),
            _ => Err(input.error("expected between one and three literal integer arguments")),
        }
    }
}
