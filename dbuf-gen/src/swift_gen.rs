use std::io::{self, Write};

use crate::ast;

/// Generate Swift source code for the provided elaborated module.
///
/// The implementation is intentionally *very* small – it is **only** capable
/// of generating code for the subset of DependoBuf that is currently covered
/// by the canonicalisation tests (`basic` and `nat_vec`).
///
/// It is good enough for snapshot-testing purposes and can be gradually
/// replaced by a full-featured backend later.
pub fn generate_module<Writer: Write>(
    module: ast::elaborated::Module<String>,
    w: &mut Writer,
) -> io::Result<()> {
    // Convert the elaborated AST used by the type-checker into the internal
    // representation expected by the generators.
    let module = ast::Module::from_elaborated(module);

    // Accumulate Swift code as an UTF-8 string – simple and fast for the needs
    // of canonicalisation.
    let mut code = String::new();
    code.push_str("import Foundation\n\n");

    for ty_rc in &module.types {
        let ty = ty_rc.as_ref();
        code.push_str(&generate_type(ty));
        code.push('\n');
    }

    w.write_all(code.as_bytes())
}

fn generate_type(ty: &ast::Type) -> String {
    let mut s = String::new();

    let module_name = ty.name.to_lowercase();
    let body_name = format!("Body");

    // namespace enum
    s.push_str(&format!("public enum {} {{\n", module_name));

    // Dependencies imports placeholder (empty for now)
    s.push_str("    public enum deps {}\n\n");

    // Body enum
    s.push_str("    public indirect enum ");
    s.push_str(&body_name);
    s.push_str(": Codable {\n");
    for constructor_rc in &ty.constructors {
        let constructor = constructor_rc.as_ref();
        let case_name = constructor.name.to_lowercase();
        s.push_str("        case ");
        s.push_str(&case_name);
        if !constructor.fields.is_empty() {
            s.push('(');
            for (i, field) in constructor.fields.iter().enumerate() {
                if i > 0 { s.push_str(", "); }
                s.push_str(&field.name);
                s.push_str(": ");
                s.push_str(&type_expr_to_swift(&field.ty));
            }
            s.push(')');
        }
        s.push('\n');
    }
    s.push_str("    }\n\n");

    // Dependencies struct
    s.push_str("    public struct Dependencies: Codable {\n");
    for dep_symbol in &ty.dependencies {
        s.push_str("        public var ");
        s.push_str(&dep_symbol.name);
        s.push_str(": ");
        s.push_str(&type_expr_to_swift(&dep_symbol.ty));
        s.push('\n');
    }
    s.push_str("    }\n\n");

    // Main message/enum struct
    s.push_str(&format!("    public struct {}: Codable {{\n", ty.name));
    s.push_str("        public var body: ");
    s.push_str(&body_name);
    s.push('\n');
    s.push_str("        public var dependencies: Dependencies\n\n");

    // Constructor functions
    for constructor_rc in &ty.constructors {
        let constructor = constructor_rc.as_ref();
        let func_name = constructor.name.to_lowercase();

        // Parameters list (implicits first, then fields)
        s.push_str("        public static func ");
        s.push_str(&func_name);
        s.push('(');

        let mut params_written = 0;
        for imp in &constructor.implicits {
            if params_written > 0 { s.push_str(", "); }
            s.push_str(imp.name.as_str());
            s.push_str(": ");
            s.push_str(&type_expr_to_swift(&imp.ty));
            params_written += 1;
        }

        for field in &constructor.fields {
            if params_written > 0 { s.push_str(", "); }
            s.push_str(field.name.as_str());
            s.push_str(": ");
            s.push_str(&type_expr_to_swift(&field.ty));
            params_written += 1;
        }
        s.push_str(") -> ");
        s.push_str(ty.name.as_str());
        s.push_str(" {\n");

        // body construction
        s.push_str("            let body = ");
        s.push_str(&body_name);
        s.push('.');
        s.push_str(&func_name);
        if !constructor.fields.is_empty() {
            s.push('(');
            for (i, field) in constructor.fields.iter().enumerate() {
                if i > 0 { s.push_str(", "); }
                s.push_str(field.name.as_str());
                s.push_str(": ");
                s.push_str(field.name.as_str());
            }
            s.push(')');
        }
        s.push_str("\n");

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
                if idx > 0 { s.push_str(", "); }
                s.push_str(&dep_sym.name);
                s.push_str(": ");
                let expr = &dep_exprs[idx];
                s.push_str(&value_expr_to_swift(expr));
            }
            s.push_str(")\n");
        }

        s.push_str("            return ");
        s.push_str(ty.name.as_str());
        s.push_str("(body: body, dependencies: dependencies)\n");
        s.push_str("        }\n\n");
    }

    // Serialization helpers
    s.push_str("        public func serialize() -> Data {\n");
    s.push_str("            return try! JSONEncoder().encode(self)\n");
    s.push_str("        }\n\n");

    s.push_str("        public static func deserialize(_ data: Data) throws -> ");
    s.push_str(&ty.name);
    s.push_str(" {\n");
    s.push_str("            return try JSONDecoder().decode(Self.self, from: data)\n");
    s.push_str("        }\n");

    // Close struct
    s.push_str("    }\n");

    // Close namespace enum
    s.push_str("}\n\n");

    // Typealias
    s.push_str("public typealias ");
    s.push_str(&ty.name);
    s.push_str(" = ");
    s.push_str(&module_name);
    s.push('.');
    s.push_str(&ty.name);
    s.push('\n');

    s
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
            weak.upgrade().map(|s| s.name.clone()).unwrap_or("_".into())
        }
        ast::ValueExpression::Constructor { call, implicits: _, arguments } => {
            let ctor = call.upgrade().expect("dangling constructor");
            let ty_name = ctor.result_type.get_type().name.clone();
            let mut res = format!("{}.{name}(", ty_name, name=ctor.name.to_lowercase());
            let mut first = true;
            for (sym_idx, arg) in arguments.iter().enumerate() {
                if !first { res.push_str(", "); } else { first = false; }
                // use positional arguments: fieldN:
                let field_name = &ctor.fields[sym_idx].name;
                res.push_str(&format!("{field}: {}", value_expr_to_swift(arg), field=field_name));
            }
            res.push(')');
            res
        }
        ast::ValueExpression::OpCall(op) => match op {
            ast::OpCall::Literal(lit) => match lit {
                ast::Literal::Int(i) => i.to_string(),
                ast::Literal::Str(s) => format!("\"{}\"", s),
                ast::Literal::Bool(b) => b.to_string(),
                ast::Literal::Double(d) => d.to_string(),
                ast::Literal::UInt(u) => u.to_string(),
            },
            ast::OpCall::Unary(_, expr) => format!("-{}", value_expr_to_swift(expr)),
            ast::OpCall::Binary(_, lhs, rhs) => format!("({} + {})", value_expr_to_swift(lhs), value_expr_to_swift(rhs)),
        },
    }
} 