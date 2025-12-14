use super::{ir::Algebra, model};

mod blades;
mod multivector;

/// Converts the internal data model to an intermediate representation for code generation
pub fn lower(model: &model::Algebra) -> Algebra {
    let model::Algebra { module_name, .. } = model;

    Algebra {
        module_name,
        blades: blades::lower(model),
        multivector: multivector::lower(model),
    }
}
