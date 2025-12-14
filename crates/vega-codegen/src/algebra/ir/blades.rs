use syn::Ident;

#[derive(Debug)]
pub struct Blades<'a> {
    pub struct_defs: IrNewtypeStruct<'a>,

    pub positive_blade_trait_impls: IrBladeTraitImpls<'a>,
    pub negative_blade_trait_impls: IrBladeTraitImpls<'a>,
    pub zero_blade_trait_impls: IrBladeTraitImpls<'a>,
    pub higher_order_blade_trait_impls: IrBladeTraitImpls<'a>,

    pub zero_trait_impls: IrBladeZeroTraitImpls<'a>,

    pub debug_trait_impls: IrBladeDebugTraitImpls<'a>,
    pub clone_trait_impls: IrBladeCloneTraitImpls<'a>,
    pub copy_trait_impls: IrBladeCopyTraitImpls<'a>,
    pub partial_eq_trait_impls: IrBladePartialEqTraitImpls<'a>,
    pub eq_trait_impls: IrBladeEqTraitImpls<'a>,
    pub partial_ord_trait_impls: IrBladePartialOrdTraitImpls<'a>,
    pub ord_trait_impls: IrBladeOrdTraitImpls<'a>,
    pub hash_trait_impls: IrBladeHashTraitImpls<'a>,

    pub add_trait_impls: IrBladeAddTraitImpls<'a>,
    pub sub_trait_impls: IrBladeSubTraitImpls<'a>,
    pub neg_trait_impls: IrBladeNegTraitImpls<'a>,
    pub mul_trait_impls: IrBladeMulTraitImpls<'a>,
    pub add_trait_impls_with_scalar_rhs: IrBladeAddTraitImplsWithScalarRhs<'a>,
    pub add_trait_impls_with_scalar_lhs: IrBladeAddTraitImplsWithScalarLhs<'a>,
    pub sub_trait_impls_with_scalar_rhs: IrBladeSubTraitImplsWithScalarRhs<'a>,
    pub sub_trait_impls_with_scalar_lhs: IrBladeSubTraitImplsWithScalarLhs<'a>,
    pub mul_trait_impls_with_scalar_rhs: IrBladeMulTraitImplsWithScalarRhs<'a>,
    pub mul_trait_impls_with_scalar_lhs: IrBladeMulTraitImplsWithScalarLhs<'a>,
    pub add_trait_impls_with_other_blade: IrBladeAddTraitImplsWithOtherBlade<'a>,
    pub sub_trait_impls_with_other_blade: IrBladeSubTraitImplsWithOtherBlade<'a>,
    pub mul_trait_impls_with_other_blade: IrBladeMulTraitImplsWithOtherBlade<'a>,
}

#[derive(Debug)]
pub struct IrNewtypeStruct<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
}

#[derive(Debug)]
pub struct IrBladeTraitImpls<'a> {
    pub idents: Vec<Ident>,
    pub unit_square_values: Vec<syn::ExprPath>,
    pub grades: Vec<usize>,
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeZeroTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeDebugTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeCloneTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeCopyTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladePartialEqTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeEqTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladePartialOrdTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeOrdTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeHashTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeAddTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeSubTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeNegTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeMulTraitImpls<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeAddTraitImplsWithScalarRhs<'a> {
    pub multivector_ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub field_types: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeAddTraitImplsWithScalarLhs<'a> {
    pub multivector_ident: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub signed_primitive_types: &'a [Ident],
    pub field_names: &'a [Ident],
    pub field_types: &'a [Ident],
}

#[derive(Debug)]
pub struct IrBladeSubTraitImplsWithScalarRhs<'a> {
    pub multivector_ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub field_names: &'a [Ident],
    pub field_types: &'a [Ident],
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeSubTraitImplsWithScalarLhs<'a> {
    pub multivector_ident: &'a Ident,
    pub scalar_field_name: &'a Ident,
    pub signed_primitive_types: &'a [Ident],
    pub field_names: &'a [Ident],
    pub field_types: &'a [Ident],
}

#[derive(Debug)]
pub struct IrBladeMulTraitImplsWithScalarRhs<'a> {
    pub idents: &'a [Ident],
    pub scalar_type: &'a Ident,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeMulTraitImplsWithScalarLhs<'a> {
    pub idents: &'a [Ident],
    pub signed_primitive_types: &'a [Ident],
}

#[derive(Debug)]
pub struct IrBladeAddTraitImplsWithOtherBlade<'a> {
    pub multivector_ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub operation_table: Vec<IrBladeAddTraitImplsWithOtherBladeTable<'a>>,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeAddTraitImplsWithOtherBladeTable<'a> {
    pub lhs_field_name: &'a Ident,
    pub lhs_field_type: &'a Ident,
    pub rhs_field_name: &'a Ident,
    pub rhs_field_type: &'a Ident,
}

#[derive(Debug)]
pub struct IrBladeSubTraitImplsWithOtherBlade<'a> {
    pub multivector_ident: &'a Ident,
    pub scalar_type: &'a Ident,
    pub operation_table: Vec<IrBladeSubTraitImplsWithOtherBladeTable<'a>>,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeSubTraitImplsWithOtherBladeTable<'a> {
    pub lhs_field_name: &'a Ident,
    pub lhs_field_type: &'a Ident,
    pub rhs_field_name: &'a Ident,
    pub rhs_field_type: &'a Ident,
}

#[derive(Debug)]
pub struct IrBladeMulTraitImplsWithOtherBlade<'a> {
    pub scalar_type: &'a Ident,
    pub operation_table: Vec<IrBladeMulTraitImplsWithOtherBladeTable>,
    pub where_clause: syn::WhereClause,
}

#[derive(Debug)]
pub struct IrBladeMulTraitImplsWithOtherBladeTable {
    pub lhs_field_type: Ident,
    pub rhs_field_type: Ident,
    pub output_type: Ident,
    pub output_signum: syn::ExprPath,
}
