//! Module provides `AstScope` - scope control
//! for parsed ast.
//!

use dbuf_core::ast::elaborated::*;

use crate::core::workspace::ElaboratedAst;
use crate::core::workspace::ElaboratedHelper;
use crate::core::workspace::Str;

/// Scope control for parsed ast. Contains current
/// type name and constructor name with their
/// representations in EAST if any.
///
/// Usage:
/// * call `enter_in_type` on entering in type.
/// * call `enter_in_constructor` on entering in constructor.
/// * call `apply_variable` on scope change.
pub struct AstScope<'a> {
    elaborated: &'a ElaboratedAst,

    /// current scope type.
    ///
    /// 0: name of type
    ///
    /// 1: representation in elaborated ast of type.
    ty: Option<(&'a str, Option<&'a Type<Str>>)>,

    /// current scope constructor.
    ///
    /// 0: name of constructor
    ///
    /// 1: representation in elaborated ast of constructor.
    constructor: Option<(&'a str, Option<&'a Constructor<Str>>)>,

    cache_ty: Option<(&'a str, Option<&'a Type<Str>>)>,
    cache_constructor: Option<(&'a str, Option<&'a Constructor<Str>>)>,
}

impl<'a> AstScope<'a> {
    pub fn new(elaborated: &ElaboratedAst) -> AstScope<'_> {
        AstScope {
            elaborated,
            ty: None,
            constructor: None,
            cache_ty: None,
            cache_constructor: None,
        }
    }

    /// Returns type name.
    pub fn get_type(&self) -> Option<&'a str> {
        self.ty.map(|a| a.0)
    }

    /// Returns constructor name.
    pub fn get_constructor(&self) -> Option<&'a str> {
        self.constructor.map(|a| a.0)
    }

    /// Enters in type.
    pub fn enter_in_type(&mut self, type_name: &'a str) {
        self.ty = (type_name, self.elaborated.get_type(type_name)).into();
        self.constructor = None;
    }

    /// Enters in constructor.
    pub fn enter_in_constructor(&mut self, constructor_name: &'a str) {
        let Some((_ty_n, ty_rep)) = self.ty else {
            return;
        };

        let c_rep = if let Some(ty) = ty_rep {
            self.elaborated.get_type_constructor(ty, constructor_name)
        } else {
            None
        };

        self.constructor = (constructor_name, c_rep).into();
    }

    /// checks variants of fields types. If found same field
    /// returns its type name.
    fn try_switch_to(
        variable: &str,
        variants: &'a [(Str, TypeExpression<Str>)],
    ) -> Option<&'a str> {
        if let Some((
            _,
            TypeExpression::TypeExpression {
                name,
                dependencies: _,
            },
        )) = variants.iter().rev().find(|v| v.0.as_ref() == variable)
        {
            Some(name.as_ref())
        } else {
            None
        }
    }

    /// resets type and constructor to None.
    fn reset(&mut self) {
        self.ty = None;
        self.constructor = None;
    }

    /// Changes scopes according to variable.
    pub fn apply_variable(&mut self, variable: &str) {
        let Some((_ty_n, Some(ty))) = self.ty else {
            self.reset();
            return;
        };

        let mut switch_to = None;
        if let Some((_ctr_n, Some(ctr))) = self.constructor {
            switch_to = switch_to.or_else(|| Self::try_switch_to(variable, &ctr.fields));
            switch_to = switch_to.or_else(|| Self::try_switch_to(variable, &ctr.implicits));
        }
        switch_to = switch_to.or_else(|| Self::try_switch_to(variable, &ty.dependencies));

        let Some(new_ty_name) = switch_to else {
            self.reset();
            return;
        };
        self.enter_in_type(new_ty_name);

        if let Some((_new_ty_n, Some(new_ty))) = self.ty
            && self.elaborated.is_message(new_ty)
        {
            self.enter_in_constructor(new_ty_name);
        }
    }

    /// Save state to local cache.
    pub fn save_state(&mut self) {
        self.cache_ty = self.ty;
        self.cache_constructor = self.constructor;
    }

    /// Loads state from cache.
    pub fn load_state(&mut self) {
        self.ty = self.cache_ty.or(self.ty);
        self.constructor = self.cache_constructor.or(self.constructor);

        self.cache_ty = None;
        self.cache_constructor = None;
    }
}
