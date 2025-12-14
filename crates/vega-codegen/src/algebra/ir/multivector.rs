use syn::Ident;

#[derive(Debug)]
pub struct Multivector<'a> {
    pub struct_def: IrNamedStruct<'a>,

    #[allow(clippy::struct_field_names)]
    pub multivector_trait_impl: IrMultivectorTraitImpl<'a>,

    pub zero_trait_impl: IrMultivectorZeroTraitImpl<'a>,

    pub debug_trait_impl: IrMultivectorDebugTraitImpl<'a>,
    pub clone_trait_impl: IrMultivectorCloneTraitImpl<'a>,
    pub copy_trait_impl: IrMultivectorCopyTraitImpl<'a>,
    pub partial_eq_trait_impl: IrMultivectorPartialEqTraitImpl<'a>,
    pub eq_trait_impl: IrMultivectorEqTraitImpl<'a>,
    pub hash_trait_impl: IrMultivectorHashTraitImpl<'a>,

    pub add_trait_impl: IrMultivectorAddTraitImpl<'a>,
    pub sub_trait_impl: IrMultivectorSubTraitImpl<'a>,
    pub neg_trait_impl: IrMultivectorNegTraitImpl<'a>,
    pub mul_trait_impl: IrMultivectorMulTraitImpl<'a>,
    pub add_trait_impl_with_scalar_rhs: IrMultivectorAddTraitImplWithScalarRhs<'a>,
    pub add_trait_impl_with_scalar_lhs: IrMultivectorAddTraitImplWithScalarLhs<'a>,
    pub sub_trait_impl_with_scalar_rhs: IrMultivectorSubTraitImplWithScalarRhs<'a>,
    pub sub_trait_impl_with_scalar_lhs: IrMultivectorSubTraitImplWithScalarLhs<'a>,
    pub mul_trait_impl_with_scalar_rhs: IrMultivectorMulTraitImplWithScalarRhs<'a>,
    pub mul_trait_impl_with_scalar_lhs: IrMultivectorMulTraitImplWithScalarLhs<'a>,
    pub add_trait_impls_with_blade: IrMultivectorAddTraitImplsWithBlade<'a>,
    pub sub_trait_impls_with_blade: IrMultivectorSubTraitImplsWithBlade<'a>,
    pub mul_trait_impls_with_blade_rhs: IrMultivectorMulTraitImplsWithBladeRhs<'a>,
    pub mul_trait_impls_with_blade_lhs: IrMultivectorMulTraitImplsWithBladeLhs<'a>,
}

#[derive(Debug)]
pub struct IrNamedStruct<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub field_types: &'a [Ident],
}

#[derive(Debug)]
pub struct IrMultivectorTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub grade_exprs: Vec<syn::Expr>,
    pub gproj_exprs: Vec<syn::ExprStruct>,
    pub grades: Vec<usize>,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorZeroTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub field_types: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorDebugTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorCloneTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorCopyTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorPartialEqTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorEqTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorHashTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorAddTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorSubTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorNegTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorMulTraitImpl<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorAddTraitImplWithScalarRhs<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorAddTraitImplWithScalarLhs<'a> {
    pub ident: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub signed_primitive_types: &'a [Ident],
}

#[derive(Debug)]
pub struct IrMultivectorSubTraitImplWithScalarRhs<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorSubTraitImplWithScalarLhs<'a> {
    pub ident: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub signed_primitive_types: &'a [Ident],
}

#[derive(Debug)]
pub struct IrMultivectorMulTraitImplWithScalarRhs<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorMulTraitImplWithScalarLhs<'a> {
    pub ident: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub signed_primitive_types: &'a [Ident],
    pub field_names: &'a [Ident],
}

#[derive(Debug)]
pub struct IrMultivectorAddTraitImplsWithBlade<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub field_names: &'a [Ident],
    pub field_types: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorSubTraitImplsWithBlade<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub field_names: &'a [Ident],
    pub field_types: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorMulTraitImplsWithBladeRhs<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub operation_table: Vec<IrMultivectorMulTraitImplsWithBladeRhsTable<'a>>,
}

#[derive(Debug)]
pub struct IrMultivectorMulTraitImplsWithBladeRhsTable<'a> {
    pub rhs_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrMultivectorMulTraitImplsWithBladeLhs<'a> {
    pub ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub field_types: &'a [Ident],
    pub where_clause: syn::WhereClause,
}
