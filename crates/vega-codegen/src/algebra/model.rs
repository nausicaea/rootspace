use syn::Ident;

/// Internal data model based on macro input
#[derive(Debug)]
pub struct Algebra {
    /// All generated code will be restricted to a module with said name at the invocation site of
    /// [`algebra`](crate::algebra!()).
    pub module_name: Ident,
    /// Defines the name of the type parameter used to denote scalars. This is the same symbol
    /// across all generated code
    pub scalar_type: Ident,
    /// Defines the names of scalar data types used for code generation when code cannot be generic
    pub signed_primitive_types: Vec<Ident>,
    /// Defines the name of the multivector type
    pub multivector_ident: Ident,
    /// Defines the name of the scalar field within the multivector struct
    pub scalar_field_name: Ident,
    /// Defines the names of all non-scalar blades within the multivector struct
    pub blade_field_names: Vec<Ident>,
    /// Defines the names of all non-scalar blade types
    pub blade_types: Vec<Ident>,
    /// Defines a multiplicative correspondence of two input types, one output type and a constant
    /// signum value (the result of anti-commute or blade squaring operations)
    pub blade_mul_table: Vec<Cm>,
    /// Defines the indices that identify each blade that squares to one
    pub positive_vector_blade_indices: Vec<usize>,
    /// Defines the indices that identify each blade that squares to minus one
    pub negative_vector_blade_indices: Vec<usize>,
    /// Defines the indices that identify each blade that squares to zero
    pub zero_vector_blade_indices: Vec<usize>,
    /// Defines the indices of all k-vector blades (k > 1)
    pub higher_order_blade_indices: Vec<Vec<usize>>,
    /// Defines the indices of all blades (k >= 1)
    pub blade_indices: Vec<Vec<usize>>,
    /// Defines the corresponding signum values of each higher order blade
    pub higher_order_unit_square_values: Vec<i8>,
}

/// Defines a multiplicative correspondence of two input types, one output type and a constant
/// signum value (the result of anti-commute or blade squaring operations)
#[derive(Debug)]
pub struct Cm {
    /// Defines the indices of the left-hand-side
    pub lhs: Vec<usize>,
    /// Defines the indices of the right-hand-side
    pub rhs: Vec<usize>,
    /// Defines the indices of the output
    pub output: Vec<usize>,
    /// Defines the signum value of the output
    pub signum: i8,
}

impl std::fmt::Display for Cm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} * {:?} -> {}*{:?}",
            self.lhs, self.rhs, self.signum, self.output
        )
    }
}
