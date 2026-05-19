//! Module contains entry point to run formatter.
use std::fs;
use std::path::PathBuf;
use std::process::exit;

use crate::cli::FormatParams;
use crate::file::File;
use crate::file_content::Error;
use crate::file_content::FileContent;
use crate::reporter::Reporter;

use dbuf_format::pretty_print;

/// Main for formatter.
pub fn run(params: &FormatParams) -> ! {
    let mut ok = true;

    for file in &params.target {
        let res = match format_file(file) {
            Ok(res) => res,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        };

        if params.check {
            if let Some(new) = res.new
                && new == res.old
            {
                continue;
            }
            ok = false;
            eprintln!("File '{}' is not formatted", file.to_string_lossy());
        } else {
            let Some(new) = res.new else {
                ok = false;
                eprintln!("Cannot format '{}'", file.to_string_lossy());
                continue;
            };
            if let Err(e) = fs::write(file, new) {
                eprintln!(
                    "Error while modifying file '{}': {e}",
                    file.to_string_lossy()
                );
                ok = false;
            }
        }
    }

    if ok {
        exit(0);
    } else {
        exit(1);
    }
}

/// Result of formatting a string.
struct FormatResult {
    /// String that was formatted.
    old: String,
    /// Pretty string.
    new: Option<String>,
}

/// Reads file and returns formatted content.
fn format_file(file: &PathBuf) -> Result<FormatResult, Error> {
    let content = FileContent::new(file)?;
    let mut file = File::new(&content);

    let mut reporter = Reporter::new(&content);
    file.process_cst(&mut reporter);

    let new = if let Some(tree) = file.get_cst() {
        pretty_print(tree).into()
    } else {
        None
    };

    reporter.print();
    Ok(FormatResult {
        old: content.get_content().to_string(),
        new,
    })
}
