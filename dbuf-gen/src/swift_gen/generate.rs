use std::fmt::Write as _;

use crate::ast;
use crate::ast::Module;

/// Accumulate Swift code as an UTF-8 string – simple and fast for the needs
/// of canonicalisation.
pub fn generate_module(module: &Module) -> String {
    let mut code = String::new();
    code.push_str("import Foundation\n\n");

    for ty_rc in &module.types {
        let ty = ty_rc.as_ref();
        code.push_str(&generate_type(ty));
        code.push('\n');
    }

    code
}

fn generate_type(ty: &ast::Type) -> String {
    let mut s = String::new();

    let module_name = ty.name.to_lowercase();
    let body_name = "Body".to_string();

    // namespace enum
    writeln!(s, "public enum {module_name} {{").expect("Writing into String is always ok");

    // Dependencies imports placeholder (empty for now)
    s.push_str("    public enum deps {}\n\n");

    // Body enum
    fill_body_enum(&mut s, ty, &body_name);

    // Dependencies struct
    fill_dependencies_struct(&mut s, ty);

    // Main message/enum struct
    fill_main_struct(&mut s, ty, &body_name);

    // Constructor functions
    fill_constructor_functions(&mut s, ty, &body_name);

    // Serialization helpers
    fill_serialization_helpers(&mut s, ty);

    // Close struct
    s.push_str("    }\n");

    // Close namespace enum
    s.push_str("}\n\n");

    // Typealias
    fill_typealias(&mut s, ty, &module_name);

    s
}

fn fill_body_enum(s: &mut String, ty: &ast::Type, body_name: &str) {
    writeln!(s, "    public indirect enum {body_name}: Codable {{")
        .expect("Writing into String is always ok");

    for constructor_rc in &ty.constructors {
        let constructor = constructor_rc.as_ref();
        let case_name = constructor.name.to_lowercase();
        write!(s, "        case {case_name}").expect("Writing into String is always ok");
        if !constructor.fields.is_empty() {
            s.push('(');
            for (i, field) in constructor.fields.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }

                write!(s, "{}: {}", field.name, type_expr_to_swift(&field.ty))
                    .expect("Writing into String is always ok");
            }
            s.push(')');
        }
        s.push('\n');
    }
    s.push_str("    }\n\n");
}

fn fill_dependencies_struct(s: &mut String, ty: &ast::Type) {
    s.push_str("    public struct Dependencies: Codable {\n");
    for dep_symbol in &ty.dependencies {
        writeln!(
            s,
            "        public var {}: {}",
            dep_symbol.name,
            type_expr_to_swift(&dep_symbol.ty)
        )
        .expect("Writing into String is always ok");
    }
    s.push_str("    }\n\n");
}

fn fill_main_struct(s: &mut String, ty: &ast::Type, body_name: &str) {
    writeln!(s, "    public struct {}: Codable {{", ty.name)
        .expect("Writing into String is always ok");

    writeln!(s, "        public var body: {body_name}").expect("Writing into String is always ok");
    s.push_str("        public var dependencies: Dependencies\n\n");
}

fn fill_constructor_functions(s: &mut String, ty: &ast::Type, body_name: &str) {
    for constructor_rc in &ty.constructors {
        let constructor = constructor_rc.as_ref();
        let func_name = constructor.name.to_lowercase();

        // Parameters list (implicits first, then fields)
        write!(s, "        public static func {func_name}(")
            .expect("Writing into String is always ok");

        let mut params_written = 0;
        for imp in &constructor.implicits {
            if params_written > 0 {
                s.push_str(", ");
            }
            write!(s, "{}: {}", imp.name, type_expr_to_swift(&imp.ty))
                .expect("Writing into String is always ok");
            params_written += 1;
        }

        for field in &constructor.fields {
            if params_written > 0 {
                s.push_str(", ");
            }
            write!(s, "{}: {}", field.name, type_expr_to_swift(&field.ty))
                .expect("Writing into String is always ok");
            params_written += 1;
        }
        writeln!(s, ") -> {} {{", ty.name).expect("Writing into String is always ok");

        // body construction
        write!(s, "            let body = {body_name}.{func_name}")
            .expect("Writing into String is always ok");
        if !constructor.fields.is_empty() {
            s.push('(');
            for (i, field) in constructor.fields.iter().enumerate() {
                if i > 0 {
                    s.push_str(", ");
                }
                write!(s, "{}: {}", field.name, field.name)
                    .expect("Writing into String is always ok");
            }
            s.push(')');
        }
        s.push('\n');

        // Build Dependencies initializer
        if ty.dependencies.is_empty() {
            s.push_str("            let dependencies = Dependencies()\n");
        } else {
            // get dependency expressions from result_type
            let dep_exprs = match &constructor.result_type {
                ast::TypeExpression::Type { dependencies, .. } => dependencies,
            };
            s.push_str("            let dependencies = Dependencies(");
            for (idx, dep_sym) in ty.dependencies.iter().enumerate() {
                if idx > 0 {
                    s.push_str(", ");
                }
                let expr = &dep_exprs[idx];
                write!(s, "{}: {}", dep_sym.name, value_expr_to_swift(expr))
                    .expect("Writing into String is always ok");
            }
            s.push_str(")\n");
        }

        writeln!(
            s,
            "            return {}(body: body, dependencies: dependencies)",
            ty.name
        )
        .expect("Writing into String is always ok");
        s.push_str("        }\n\n");
    }
}

fn fill_serialization_helpers(s: &mut String, ty: &ast::Type) {
    s.push_str("        public func serialize() -> Data {\n");
    s.push_str("            return try! JSONEncoder().encode(self)\n");
    s.push_str("        }\n\n");

    writeln!(
        s,
        "        public static func deserialize(_ data: Data) throws -> {} {{",
        ty.name
    )
    .expect("Writing into String is always ok");
    s.push_str("            return try JSONDecoder().decode(Self.self, from: data)\n");
    s.push_str("        }\n");
}

fn fill_typealias(s: &mut String, ty: &ast::Type, module_name: &str) {
    writeln!(
        s,
        "public typealias {} = {}.{}",
        ty.name, module_name, ty.name
    )
    .expect("Writing into String is always ok");
}

fn type_expr_to_swift(expr: &ast::TypeExpression) -> String {
    match expr {
        ast::TypeExpression::Type { call, .. } => {
            let ty = call.upgrade().expect("dangling reference to type");
            ty.name.clone()
        }
    }
}

fn value_expr_to_swift(expr: &ast::ValueExpression) -> String {
    match expr {
        ast::ValueExpression::Variable(weak) => {
            weak.upgrade().map_or("_".into(), |s| s.name.clone())
        }
        ast::ValueExpression::Constructor {
            call,
            implicits: _,
            arguments,
        } => {
            let ctor = call.upgrade().expect("dangling constructor");
            let ty_name = ctor.result_type.get_type().name.clone();
            let mut res = format!("{}.{name}(", ty_name, name = ctor.name.to_lowercase());
            let mut first = true;
            for (sym_idx, arg) in arguments.iter().enumerate() {
                if first {
                    first = false;
                } else {
                    res.push_str(", ");
                }
                // use positional arguments: fieldN:
                let field_name = &ctor.fields[sym_idx].name;
                write!(
                    res,
                    "{field}: {}",
                    value_expr_to_swift(arg),
                    field = field_name
                )
                .expect("Writing into String is always ok");
            }
            res.push(')');
            res
        }
        ast::ValueExpression::OpCall(op) => match op {
            ast::OpCall::Literal(lit) => match lit {
                ast::Literal::Int(i) => i.to_string(),
                ast::Literal::Str(s) => format!("\"{s}\""),
                ast::Literal::Bool(b) => b.to_string(),
                ast::Literal::Double(d) => d.to_string(),
                ast::Literal::UInt(u) => u.to_string(),
            },
            ast::OpCall::Unary(_, expr) => format!("-{}", value_expr_to_swift(expr)),
            ast::OpCall::Binary(_, lhs, rhs) => format!(
                "({} + {})",
                value_expr_to_swift(lhs),
                value_expr_to_swift(rhs)
            ),
        },
    }
}
