//! Provide API for building parsed AST with locations.
//!
//!

use dbuf_core::ast::parsed::definition::*;
use dbuf_core::ast::parsed::*;

use crate::core::ast_access::Location;

use crate::core::ast_access::{ConvertibleToString, Loc, Str};

use super::ast_fix_locations::fix_locations;

/// Builder for dependencies
pub struct DependencyBuilder {
    dependencies: Definitions<Loc, Str, TypeExpression<Loc, Str>>,
}

impl DependencyBuilder {
    fn new() -> DependencyBuilder {
        DependencyBuilder {
            dependencies: vec![],
        }
    }
    pub fn with_huge_dependency(
        &mut self,
        name: &str,
        dep_type: &str,
        args: Rec<[Expression<Loc, Str>]>,
    ) -> &mut Self {
        self.dependencies.push(Definition {
            loc: Location::default(),
            name: name.to_loc_string(),
            data: Expression {
                loc: Location::default(),
                node: ExpressionNode::FunCall {
                    fun: dep_type.to_loc_string(),
                    args,
                },
            },
        });
        self
    }
    pub fn with_dependency(&mut self, name: &str, dep_type: &str) -> &mut Self {
        self.with_huge_dependency(name, dep_type, Rec::new([]))
    }
}

/// Builder for constructors
pub struct ConstructorBuilder {
    fields: Definitions<Loc, Str, TypeExpression<Loc, Str>>,
}

impl ConstructorBuilder {
    fn new() -> ConstructorBuilder {
        ConstructorBuilder { fields: vec![] }
    }
    pub fn with_huge_field(
        &mut self,
        name: &str,
        field_type: &str,
        args: Rec<[Expression<Loc, Str>]>,
    ) -> &mut Self {
        self.fields.push(Definition {
            loc: Location::default(),
            name: name.to_loc_string(),
            data: Expression {
                loc: Location::default(),
                node: ExpressionNode::FunCall {
                    fun: field_type.to_loc_string(),
                    args,
                },
            },
        });
        self
    }
    pub fn with_field(&mut self, name: &str, field_type: &str) -> &mut Self {
        self.with_huge_field(name, field_type, Rec::new([]))
    }
}

/// Builder for Message
pub struct MessageBuilder {
    name: Str,
    dependencies: DependencyBuilder,
    fields: ConstructorBuilder,
}

impl MessageBuilder {
    pub fn new(name: &str) -> MessageBuilder {
        MessageBuilder {
            name: name.to_loc_string(),
            dependencies: DependencyBuilder::new(),
            fields: ConstructorBuilder::new(),
        }
    }

    pub fn with_huge_dependency(
        &mut self,
        name: &str,
        dep_type: &str,
        args: Rec<[Expression<Loc, Str>]>,
    ) -> &mut Self {
        self.dependencies.with_huge_dependency(name, dep_type, args);
        self
    }
    pub fn with_dependency(&mut self, name: &str, dep_type: &str) -> &mut Self {
        self.dependencies.with_dependency(name, dep_type);
        self
    }

    pub fn with_huge_field(
        &mut self,
        name: &str,
        field_type: &str,
        args: Rec<[Expression<Loc, Str>]>,
    ) -> &mut Self {
        self.fields.with_huge_field(name, field_type, args);
        self
    }
    pub fn with_field(&mut self, name: &str, field_type: &str) -> &mut Self {
        self.fields.with_field(name, field_type);
        self
    }

    pub fn construct(self) -> Definition<Loc, Str, TypeDeclaration<Loc, Str>> {
        Definition {
            loc: Location::default(),
            name: self.name,
            data: TypeDeclaration {
                dependencies: self.dependencies.dependencies,
                body: TypeDefinition::Message(self.fields.fields),
            },
        }
    }
}

/// Builder for Enum Branches
pub struct EnumBranchBuilder {
    patterns: Vec<Pattern<Loc, Str>>,
    constructors: Vec<ConstructorBuilder>,
    names: Vec<Str>,
}

impl EnumBranchBuilder {
    fn new() -> EnumBranchBuilder {
        EnumBranchBuilder {
            patterns: vec![],
            constructors: vec![],
            names: vec![],
        }
    }
    pub fn with_pattern(&mut self, pattern: PatternNode<Loc, Str, Pattern<Loc, Str>>) -> &mut Self {
        self.patterns.push(Pattern {
            loc: Location::default(),
            node: pattern,
        });
        self
    }

    pub fn with_constructor(&mut self, name: &str) -> &mut ConstructorBuilder {
        self.constructors.push(ConstructorBuilder::new());
        self.names.push(name.to_loc_string());
        if let Some(last) = self.constructors.last_mut() {
            return last;
        }
        panic!("just pushed element vanished");
    }

    pub fn construct(self) -> EnumBranch<Loc, Str> {
        let mut definitions = vec![];

        let names = self.names.into_iter();
        let constructors = self.constructors.into_iter();
        for (name, c) in names.zip(constructors) {
            definitions.push(Definition {
                loc: Location::default(),
                name,
                data: c.fields,
            });
        }

        EnumBranch {
            patterns: self.patterns,
            constructors: definitions,
        }
    }
}

/// Builder for Enum
pub struct EnumBuilder {
    name: Str,
    dependencies: DependencyBuilder,
    branches: Vec<EnumBranchBuilder>,
}

impl EnumBuilder {
    pub fn new(name: &str) -> EnumBuilder {
        EnumBuilder {
            name: name.to_loc_string(),
            dependencies: DependencyBuilder::new(),
            branches: vec![],
        }
    }

    pub fn with_huge_dependency(
        &mut self,
        name: &str,
        dep_type: &str,
        args: Rec<[Expression<Loc, Str>]>,
    ) -> &mut Self {
        self.dependencies.with_huge_dependency(name, dep_type, args);
        self
    }
    pub fn with_dependency(&mut self, name: &str, dep_type: &str) -> &mut Self {
        self.dependencies.with_dependency(name, dep_type);
        self
    }

    pub fn with_branch(&mut self) -> &mut EnumBranchBuilder {
        self.branches.push(EnumBranchBuilder::new());
        if let Some(last) = self.branches.last_mut() {
            return last;
        }
        panic!("just pushed element vanished!")
    }

    pub fn construct(self) -> Definition<Loc, Str, TypeDeclaration<Loc, Str>> {
        let mut branches = vec![];
        for b in self.branches {
            branches.push(b.construct());
        }

        Definition {
            loc: Location::default(),
            name: self.name,
            data: TypeDeclaration {
                dependencies: self.dependencies.dependencies,
                body: TypeDefinition::Enum(branches),
            },
        }
    }
}

enum Structure {
    Message(MessageBuilder),
    Enum(EnumBuilder),
}

impl Structure {
    fn construct(self) -> Definition<Loc, Str, TypeDeclaration<Loc, Str>> {
        match self {
            Structure::Message(builder) => builder.construct(),
            Structure::Enum(builder) => builder.construct(),
        }
    }
}

/// Builder for ast
pub struct AstBuilder {
    data: Vec<Structure>,
}

impl AstBuilder {
    pub fn new() -> AstBuilder {
        AstBuilder { data: vec![] }
    }

    pub fn with_message(&mut self, name: &str) -> &mut MessageBuilder {
        self.data
            .push(Structure::Message(MessageBuilder::new(name)));
        if let Some(Structure::Message(last)) = self.data.last_mut() {
            return last;
        }
        panic!("just pushed element vanished");
    }

    pub fn with_enum(&mut self, name: &str) -> &mut EnumBuilder {
        self.data.push(Structure::Enum(EnumBuilder::new(name)));
        if let Some(Structure::Enum(last)) = self.data.last_mut() {
            return last;
        }
        panic!("just pushed element vanished");
    }

    pub fn construct(self) -> Module<Loc, Str> {
        let mut ast = vec![];

        for e in self.data {
            ast.push(e.construct());
        }

        fix_locations(&mut ast);

        ast
    }
}
