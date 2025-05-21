use std::borrow::Borrow;
use std::{collections::HashMap, hash::Hash};

use dbuf_core::ast::operators::{OpCall, UnaryOp};
use dbuf_core::ast::parsed::Rec;
use dbuf_core::ast::parsed::{
    definition::Definition, ConstructorBody, EnumBranch, Expression, ExpressionNode, Module,
    Pattern, PatternNode, TypeDeclaration, TypeDefinition,
};

pub type InternedString = u32;
pub type InternedModule<Loc> = Module<Loc, InternedString>;
pub type InternedDefinition<Loc, Data> = Definition<Loc, InternedString, Data>;
pub type InternedTypeDeclaration<Loc> =
    InternedDefinition<Loc, TypeDeclaration<Loc, InternedString>>;
pub type InternedExpression<Loc> = Expression<Loc, InternedString>;
pub type InternedConstructor<Loc> = ConstructorBody<Loc, InternedString>;
pub type InternedPattern<Loc> = Pattern<Loc, InternedString>;

#[derive(Default, Debug)]
pub struct StringInterner<Str: Hash + Eq + Clone> {
    mapping: HashMap<Str, InternedString>,
    reverse_mapping: Vec<Str>,
    unused_index: InternedString,
}

impl<Str: Default + Hash + Eq + Clone> StringInterner<Str> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_string<T>(&mut self, string: &T) -> Result<InternedString, InternedString>
    where
        T: ToOwned<Owned = Str> + ?Sized,
    {
        let new_index = match self.mapping.insert(string.to_owned(), self.unused_index) {
            None => self.unused_index,
            Some(index) => return Err(index),
        };
        self.reverse_mapping.push(string.to_owned());
        self.unused_index += 1;
        Ok(new_index)
    }

    pub fn get_index<T>(&self, string: &T) -> Option<InternedString>
    where
        Str: Borrow<T>,
        T: Hash + Eq + ?Sized,
    {
        self.mapping.get(string).copied()
    }

    pub fn transform<T>(&mut self, string: &T) -> InternedString
    where
        Str: Borrow<T>,
        T: ToOwned<Owned = Str> + Hash + Eq + ?Sized,
    {
        match self.get_index(string) {
            None => self.add_string(string).expect("Item not added yet"),
            Some(index) => index,
        }
    }

    pub fn get_string(&self, index: InternedString) -> Option<&Str> {
        self.reverse_mapping.get(index as usize)
    }
}

#[derive(Default, Debug)]
pub struct ModuleInterner<Str: Hash + Eq + Clone> {
    pub interner: StringInterner<Str>,
}

impl<Str: Default + Hash + Eq + Clone> ModuleInterner<Str> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn transform_module<Loc: Clone>(
        &mut self,
        module: Module<Loc, Str>,
    ) -> Module<Loc, InternedString> {
        module
            .into_iter()
            .map(|definition| Definition {
                loc: definition.loc,
                name: self.interner.transform(&definition.name),
                data: self.transform_type_declaration(definition.data),
            })
            .collect()
    }

    fn transform_type_declaration<Loc: Clone>(
        &mut self,
        declaration: TypeDeclaration<Loc, Str>,
    ) -> TypeDeclaration<Loc, InternedString> {
        let dependencies = declaration
            .dependencies
            .into_iter()
            .map(|definition| Definition {
                loc: definition.loc,
                name: self.interner.transform(&definition.name),
                data: self.transform_expression(definition.data),
            })
            .collect();
        let body = self.transfrom_type_definition(declaration.body);
        TypeDeclaration { dependencies, body }
    }

    fn transfrom_type_definition<Loc: Clone>(
        &mut self,
        definition: TypeDefinition<Loc, Str>,
    ) -> TypeDefinition<Loc, InternedString> {
        use TypeDefinition::*;

        match definition {
            Message(definition) => Message(self.transform_constructor(definition)),
            Enum(branches) => Enum(
                branches
                    .into_iter()
                    .map(|branch| self.transform_enum_branch(branch))
                    .collect(),
            ),
        }
    }

    fn transform_enum_branch<Loc: Clone>(
        &mut self,
        branch: EnumBranch<Loc, Str>,
    ) -> EnumBranch<Loc, InternedString> {
        let patterns = branch
            .patterns
            .into_iter()
            .map(|pattern| self.transform_pattern(pattern))
            .collect();
        let constructors = branch
            .constructors
            .into_iter()
            .map(|definition| Definition {
                loc: definition.loc,
                name: self.interner.transform(&definition.name),
                data: self.transform_constructor(definition.data),
            })
            .collect();

        EnumBranch {
            patterns,
            constructors,
        }
    }

    fn transform_constructor<Loc: Clone>(
        &mut self,
        fields: ConstructorBody<Loc, Str>,
    ) -> ConstructorBody<Loc, InternedString> {
        fields
            .into_iter()
            .map(|definition| Definition {
                loc: definition.loc,
                name: self.interner.transform(&definition.name),
                data: self.transform_expression(definition.data),
            })
            .collect()
    }

    fn transform_pattern<Loc>(
        &mut self,
        pattern: Pattern<Loc, Str>,
    ) -> Pattern<Loc, InternedString> {
        use PatternNode::*;

        let node = match pattern.node {
            ConstructorCall { name, fields } => {
                let fields = fields
                    .into_iter()
                    .map(|definition| Definition {
                        loc: definition.loc,
                        name: self.interner.transform(&definition.name),
                        data: self.transform_pattern(definition.data),
                    })
                    .collect();
                ConstructorCall {
                    name: self.interner.transform(&name),
                    fields,
                }
            }
            Variable { name } => Variable {
                name: self.interner.transform(&name),
            },
            Literal(literal) => Literal(literal),
            Underscore => Underscore,
        };

        Pattern {
            loc: pattern.loc,
            node,
        }
    }

    fn transform_expression<Loc: Clone>(
        &mut self,
        expression: Expression<Loc, Str>,
    ) -> Expression<Loc, InternedString> {
        use ExpressionNode::{ConstructorCall, FunCall, TypedHole, Variable};

        let node = match expression.node {
            ExpressionNode::OpCall(opcall) => {
                use OpCall::*;
                let opcall = match opcall {
                    Literal(literal) => Literal(literal),
                    Unary(op, expr) => {
                        use UnaryOp::*;
                        let op = match op {
                            Access(name) => Access(self.interner.transform(&name)),
                            Minus => Minus,
                            Bang => Bang,
                        };
                        Unary(op, Rec::new(self.transform_expression((*expr).clone())))
                    }
                    Binary(op, lhs, rhs) => {
                        let lhs = Rec::new(self.transform_expression((*lhs).clone()));
                        let rhs = Rec::new(self.transform_expression((*rhs).clone()));
                        Binary(op, lhs, rhs)
                    }
                };
                ExpressionNode::OpCall(opcall)
            }
            FunCall { fun, args } => {
                let args = args
                    .into_iter()
                    .cloned()
                    .map(|expr| self.transform_expression(expr))
                    .collect();
                FunCall {
                    fun: self.interner.transform(&fun),
                    args,
                }
            }
            ConstructorCall { name, fields } => {
                let fields = fields
                    .into_iter()
                    .map(|definition| Definition {
                        loc: definition.loc,
                        name: self.interner.transform(&definition.name),
                        data: self.transform_expression(definition.data),
                    })
                    .collect();
                ConstructorCall {
                    name: self.interner.transform(&name),
                    fields,
                }
            }
            Variable { name } => Variable {
                name: self.interner.transform(&name),
            },
            TypedHole => TypedHole,
        };

        Expression {
            loc: expression.loc,
            node,
        }
    }
}
