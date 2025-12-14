use syn::Ident;

use self::{blades::Blades, multivector::Multivector};

pub mod blades;
pub mod multivector;

#[derive(Debug)]
pub struct Algebra<'a> {
    pub module_name: &'a Ident,
    pub blades: Blades<'a>,
    pub multivector: Multivector<'a>,
}
