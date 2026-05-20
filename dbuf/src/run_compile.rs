//! Module contains entry point to run compiler.
use std::collections::HashMap;
use std::fs;
use std::path;
use std::process::exit;
use std::sync::LazyLock;

use dbuf_core::ast::elaborated as e;
type ElaboratedModule = e::Module<String>;

use super::file::File;
use super::reporter::Reporter;
use crate::cli::CompileParams;
use crate::file_content::FileContent;

/// Configuration for supported language generation
struct LanguageConfig {
    /// Extensions of files for that language.
    extension: &'static str,
    /// Code generation function for language.
    codegen: fn(&ElaboratedModule) -> String,
}

/// Supported languages.
static LANGUAGES: LazyLock<HashMap<&str, Option<LanguageConfig>>> = LazyLock::new(|| {
    #[cfg(feature = "rust")]
    let rust_set = Some(LanguageConfig {
        extension: ".rs",
        codegen: rust_gen_impl::run,
    });
    #[cfg(not(feature = "rust"))]
    let rust_set = None;

    #[cfg(feature = "kotlin")]
    let kotlin_set = Some(LanguageConfig {
        extension: ".kt",
        codegen: kotlin_gen_impl::run,
    });
    #[cfg(not(feature = "kotlin"))]
    let kotlin_set = None;

    #[cfg(feature = "swift")]
    let swift_set = Some(LanguageConfig {
        extension: ".swift",
        codegen: swift_gen_impl::run,
    });
    #[cfg(not(feature = "swift"))]
    let swift_set = None;

    HashMap::from([
        ("rust", rust_set),
        ("kotlin", kotlin_set),
        ("swift", swift_set),
    ])
});

/// Main for compiler.
pub fn run(params: &CompileParams) -> ! {
    let file_content = match FileContent::new(&params.file) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };
    let mut file = File::new(&file_content);

    let mut reporter = Reporter::new(&file_content);
    let r = process(params, &mut file, &mut reporter);

    reporter.print();

    if r.is_ok() {
        exit(0);
    } else {
        exit(1);
    }
}

/// Process a file.
fn process(params: &CompileParams, file: &mut File, reporter: &mut Reporter) -> Result<(), ()> {
    file.process_cst(reporter);
    file.process_ast(reporter);
    file.process_east(reporter);

    let out_dir = path::Path::new(&params.path);
    for out in &params.output {
        let Some(opt_config) = LANGUAGES.get(out.as_str()) else {
            eprintln!("Unsupported language: {out}");
            return Err(());
        };

        let Some(config) = opt_config else {
            eprintln!("Feature for language {out} is not enabled");
            return Err(());
        };

        let file_name = file.get_name().to_string() + config.extension;
        let to = out_dir.join(file_name);

        if let Some(elaborated) = file.get_east() {
            let output = (config.codegen)(elaborated);
            write_generated(output, &to)?;
        } else {
            eprintln!("No elaborated ast to generate code");
            return Err(());
        }
    }

    Ok(())
}

/// Write generated text to path
fn write_generated(generated: String, to: &path::Path) -> Result<(), ()> {
    fs::write(to, generated).map_err(|e| {
        eprintln!("Error while creating file: {e}");
    })
}

#[cfg(feature = "kotlin")]
mod kotlin_gen_impl {
    use super::ElaboratedModule;
    use dbuf_gen::kotlin_gen;

    /// impl of kotlin code generation.
    pub fn run(module: &ElaboratedModule) -> String {
        // FIXME: stop clonning module.
        kotlin_gen::generate_module(module.clone())
    }
}

#[cfg(feature = "rust")]
mod rust_gen_impl {
    use super::ElaboratedModule;
    use dbuf_gen::codegen;

    /// impl of rust code generation.
    pub fn run(module: &ElaboratedModule) -> String {
        let mut writer = Vec::new();
        // FIXME: stop clonning module.
        assert!(codegen::generate_module(module.clone(), &mut writer).is_ok());
        String::from_utf8(writer).expect("generated code must be correct utf8")
    }
}

#[cfg(feature = "swift")]
mod swift_gen_impl {
    use super::ElaboratedModule;
    use dbuf_gen::swift_gen;

    /// impl of swift code generation.
    pub fn run(module: &ElaboratedModule) -> String {
        // FIXME: stop clonning module.
        swift_gen::generate_module(module.clone())
    }
}
