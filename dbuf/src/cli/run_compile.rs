use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path;
use std::process::exit;
use std::sync::LazyLock;

use dbuf_core::ast::elaborated;
use dbuf_core::cst::{ParsedModule, convert_to_ast, parse_to_cst};
type ElaboratedModule = elaborated::Module<String>;

use dbuf_gen::codegen;
use dbuf_gen::kotlin_gen;
use dbuf_gen::swift_gen;

use super::CompileParams;
use super::file::File;
use super::reporter::Reporter;

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

pub fn run(params: &CompileParams) -> ! {
    let Ok(file) = get_file(params) else {
        eprintln!("Bad input file");
        exit(1);
    };

    let mut reporter = Reporter::new(&file);
    let r = process(params, &file, &mut reporter);

    reporter.print();

    if r.is_ok() {
        exit(0);
    } else {
        exit(1);
    }
}

/// Reads input file based on compile params and return its name and content
fn get_file(params: &CompileParams) -> Result<File, ()> {
    let file = path::Path::new(&params.file);
    let file_name = file
        .file_stem()
        .ok_or_else(|| eprintln!("'{0}' is not a file name", params.file))?
        .to_str()
        .expect("file name is valid unicode");

    let content = fs::read_to_string(file)
        .map_err(|e| eprintln!("Can't read file '{}': {e}", params.file))?;

    Ok(File {
        name: file_name.to_string(),
        content,
    })
}

fn process(params: &CompileParams, file: &File, reporter: &mut Reporter) -> Result<(), ()> {
    let parsed = get_parsed(&file.content, reporter)?;
    let _elaborated = get_elaborated(&parsed, reporter)?;

    let out_dir = path::Path::new(&params.path);
    for out in &params.output {
        let Some(config) = LANGUAGES.get(out.as_str()) else {
            eprintln!("Unsupported language: {out}");
            continue;
        };

        let file_name = file.name.clone() + config.extension;
        let to = out_dir.join(file_name);

        // FIXME: codegens shouldn't consume elaborated tree
        let elaborated = get_elaborated(&parsed, reporter)?;
        let output = (config.codegen)(elaborated);
        write_generated(output, &to)?;
    }

    Ok(())
}

fn get_parsed(input: &str, reporter: &mut Reporter) -> Result<ParsedModule, ()> {
    let (tree, errors) = parse_to_cst(input);

    for e in errors {
        reporter.report(e);
    }

    if let Some(tree) = tree {
        Ok(convert_to_ast(&tree))
    } else {
        Err(())
    }
}

#[allow(clippy::unnecessary_wraps, reason = "elaboration could return errors")]
fn get_elaborated(
    _module: &ParsedModule,
    _reporter: &mut Reporter,
) -> Result<ElaboratedModule, ()> {
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

fn rust_codegen(module: ElaboratedModule) -> String {
    let mut writer = Vec::new();
    assert!(codegen::generate_module(module, &mut writer).is_ok());
    String::from_utf8(writer).expect("generated code must be correct utf8")
}
