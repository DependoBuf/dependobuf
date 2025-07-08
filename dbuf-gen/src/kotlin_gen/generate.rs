use pretty::{BoxAllocator, BoxDoc, DocAllocator};

use crate::ast::{Module, Type, TypeKind};
use crate::kotlin_gen::target as kotlin;

fn generate_class<'a>(t: &Type, alloc: &'a BoxAllocator) -> BoxDoc<'a> {
    if t.kind == TypeKind::Message {
        assert!(t.constructors.len() == 1);
    }
    let class = kotlin::SealedClass {
        name: t.name.clone(),
        fields: t
            .dependencies
            .iter()
            .map(|dep| kotlin::Field::new(dep))
            .collect(),
        constructors: t
            .constructors
            .iter()
            .map(|constructor| kotlin::InnerClass {
                name: constructor.name.clone(),
                fields: constructor
                    .fields
                    .iter()
                    .map(|field| kotlin::Field::new(field))
                    .collect(),
                parent_params: constructor
                    .implicits
                    .iter()
                    .map(|field| kotlin::Field::new(field))
                    .collect(),
                result_type: constructor.result_type.clone(),
            })
            .collect(),
    };

    class.generate(alloc)
}

pub fn generate_module(module: &Module) -> String {
    let alloc = &BoxAllocator;
    let mut writer = Vec::new();
    for t in &module.types {
        generate_class(t, alloc)
            .append(alloc.hardline())
            .render(40, &mut writer)
            .expect("To be ok");
    }

    String::from_utf8(writer).expect("generated code must be correct utf8")
}
