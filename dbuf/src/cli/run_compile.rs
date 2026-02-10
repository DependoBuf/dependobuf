use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path;
use std::sync::LazyLock;

use dbuf_core::ast::elaborated;
use dbuf_core::cst::{ParsedModule, convert_to_ast, parse_to_cst};
type ElaboratedModule = elaborated::Module<String>;

use dbuf_gen::codegen;
use dbuf_gen::kotlin_gen;
use dbuf_gen::swift_gen;

use super::CompileParams;

struct LanguageConfig {
    extension: &'static str,
    codegen: fn(ElaboratedModule) -> String,
}

static LANGUAGES: LazyLock<HashMap<&str, LanguageConfig>> = LazyLock::new(|| {
    HashMap::from([
        (
            "rust",
            LanguageConfig {
                extension: ".rs",
                codegen: rust_codegen,
            },
        ),
        (
            "kotlin",
            LanguageConfig {
                extension: ".kt",
                codegen: kotlin_gen::generate_module,
            },
        ),
        (
            "swift",
            LanguageConfig {
                extension: ".swift",
                codegen: swift_gen::generate_module,
            },
        ),
    ])
});

pub fn run(params: CompileParams) {
    let file = path::Path::new(&params.file);
    let Some(file_name) = file.file_stem() else {
        eprintln!("{0} is not a file name", params.file);
        return;
    };

    let out_dir = path::Path::new(&params.path);

    for out in params.output {
        let Some(config) = LANGUAGES.get(out.as_str()) else {
            eprintln!("Unsupported language: {out}");
            return;
        };

        let file_name = file_name.to_str().expect("valid OsStr").to_owned() + config.extension;
        let to = out_dir.join(file_name);
        if produce(file, &to, config.codegen).is_err() {
            eprintln!("Error in codegen to {out}");
            return;
        }
    }
}

fn get_parsed(file: &path::Path) -> Result<ParsedModule, ()> {
    let input = fs::read_to_string(file).map_err(|e| {
        eprintln!("Error while trying to open file: {e}");
    })?;

    let parse_result = parse_to_cst(input.as_ref());

    let tree = parse_result.into_result().map_err(|errs| {
        eprintln!("Parsing errors:");
        for err in errs {
            eprintln!("{err:#?}");
        }
    })?;

    Ok(convert_to_ast(&tree))
}

#[allow(clippy::unnecessary_wraps, reason = "elaboration could return errors")]
fn get_elaborated(_module: ParsedModule) -> Result<ElaboratedModule, ()> {
    eprintln!("UNIMPLEMENTED: convertation from parsed module to elaborated");

    Ok(ElaboratedModule {
        types: vec![],
        constructors: BTreeMap::new(),
    })
}

fn write_generated(generated: String, to: &path::Path) -> Result<(), ()> {
    fs::write(to, generated).map_err(|e| {
        eprintln!("Error while creating file: {e}");
    })
}

fn produce<T: FnOnce(ElaboratedModule) -> String>(
    file: &path::Path,
    to: &path::Path,
    codegen: T,
) -> Result<(), ()> {
    let parsed = get_parsed(file)?;
    let elaborated = get_elaborated(parsed)?;
    let ans = codegen(elaborated);
    write_generated(ans, to)
}

fn rust_codegen(module: ElaboratedModule) -> String {
    let mut writer = Vec::new();
    assert!(codegen::generate_module(module, &mut writer).is_ok());
    String::from_utf8(writer).expect("generated code must be correct utf8")
}
