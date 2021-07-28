use super::{namespace::*, scope::*, *};
use crate::ast;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimpleType(pub ast::SimpleType);

impl Legalize for SimpleType {
    type Input = ast::SimpleType;
    fn legalize(
        _ns: &Namespace,
        _ss: &SubSuperGraph,
        _scope: &Scope,
        input: &Self::Input,
    ) -> Result<Self, SemanticError> {
        Ok(SimpleType(input.clone()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bound {}

impl Legalize for Bound {
    type Input = ast::Bound;
    fn legalize(
        _ns: &Namespace,
        _ss: &SubSuperGraph,
        _scope: &Scope,
        _input: &Self::Input,
    ) -> Result<Self, SemanticError> {
        // FIXME
        Ok(Bound {})
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeRef {
    SimpleType(SimpleType),
    Named {
        name: String,

        /// Scope where the named type is declared
        scope: Scope,

        /// True if the underlying type of named type is a simple type:
        ///
        /// ```text
        /// TYPE a = INTEGER; ENDTYPE;
        /// TYPE b = a; ENDTYPE;
        /// ```
        ///
        /// Then both `a` and `b` are simple.
        ///
        is_simple: bool,
    },

    /* Declared as `ENTITY` */
    Entity {
        name: String,
        scope: Scope,
        has_supertype_decl: bool,
    },

    /* Aggregated */
    Set {
        base: Box<TypeRef>,
        bound: Option<Bound>,
    },
    List {
        base: Box<TypeRef>,
        bound: Option<Bound>,
        unique: bool,
    },
}

impl TypeRef {
    pub fn is_simple(&self) -> bool {
        match self {
            TypeRef::SimpleType(..) => true,
            TypeRef::Named { is_simple, .. } => *is_simple,
            _ => false,
        }
    }
}

impl Legalize for TypeRef {
    type Input = ast::Type;

    fn legalize(
        ns: &Namespace,
        ss: &SubSuperGraph,
        scope: &Scope,
        ty: &ast::Type,
    ) -> Result<Self, SemanticError> {
        use ast::Type::*;
        Ok(match ty {
            Simple(ty) => Self::SimpleType(SimpleType(*ty)),
            Named(name) => ns.lookup_type(scope, name)?,
            Set { base, bound } => {
                let base = TypeRef::legalize(ns, ss, scope, base.as_ref())?;
                let bound = if let Some(bound) = bound {
                    Some(Legalize::legalize(ns, ss, scope, bound)?)
                } else {
                    None
                };
                Self::Set {
                    base: Box::new(base),
                    bound,
                }
            }
            List {
                base,
                bound,
                unique,
            } => {
                let base = TypeRef::legalize(ns, ss, scope, base.as_ref())?;
                let bound = if let Some(bound) = bound {
                    Some(Legalize::legalize(ns, ss, scope, bound)?)
                } else {
                    None
                };
                Self::List {
                    base: Box::new(base),
                    bound,
                    unique: *unique,
                }
            }
            _ => todo!(),
        })
    }
}
