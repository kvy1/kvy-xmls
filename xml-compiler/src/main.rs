use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use walkdir::WalkDir;

static LOG_FILE: Lazy<Mutex<fs::File>> = Lazy::new(|| {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("processing.log")
        .expect("Failed to create or open log file");
    Mutex::new(file)
});

fn timestamp() -> String {
    let now = SystemTime::now();
    let datetime: DateTime<Local> = now.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn log_message(msg: &str) {
    let entry = format!("[{}]  {}", timestamp(), msg);
    if let Ok(mut f) = LOG_FILE.lock() {
        let _ = writeln!(f, "{}", entry);
    }
}

fn log_section(title: &str) {
    if let Ok(mut f) = LOG_FILE.lock() {
        let _ = writeln!(f, "\n────────────────────────────────────────────");
        let _ = writeln!(f, "{}", title);
        let _ = writeln!(f, "────────────────────────────────────────────");
    }
}

fn normalize_include_path(base_dir: &Path, include: &str) -> PathBuf {
    let normalized = if cfg!(windows) {
        include.to_string()
    } else {
        include.replace('\\', "/")
    };
    base_dir.join(normalized)
}

fn expand_includes(file_path: &Path, is_root: bool) -> Result<String> {
    let content = fs::read_to_string(file_path)?;
    let include_re = Regex::new(r#"<!--\s*#include file="(.*?)"\s*-->"#)?;
    let dir = file_path.parent().unwrap_or_else(|| Path::new("."));

    let replaced = include_re.replace_all(&content, |caps: &regex::Captures| {
        let include_path = normalize_include_path(dir, caps[1].trim());
        if !include_path.exists() {
            log_message(&format!("Missing include: {}", include_path.display()));
            return format!("<!-- Include not found: {} -->", include_path.display());
        }

        match expand_includes(&include_path, false) {
            Ok(included_content) => {
                let inner = remove_placeholders(&included_content);
                let inner = strip_comments_and_format_spaces(&inner);

                log_message(&format!("Included: {}", include_path.display()));

                if is_root {
                    format!("<![CDATA[\n{}\n]]>", inner)
                } else {
                    inner
                }
            }
            Err(err) => {
                log_message(&format!(
                    "Error including {}: {}",
                    include_path.display(),
                    err
                ));
                format!("<!-- Error including {}: {} -->", include_path.display(), err)
            }
        }
    });

    if is_root {
        Ok(replaced.to_string())
    } else {
        let cleaned = remove_comments(&remove_placeholders(&replaced));
        Ok(cleaned)
    }
}

fn remove_placeholders(input: &str) -> String {
    let re = Regex::new(r"(?is)<placeholder[^>]*>(.*?)</placeholder>").unwrap();
    re.replace_all(input, "$1").to_string()
}

fn remove_comments(input: &str) -> String {
    let re = Regex::new(r"(?s)<!--.*?-->").unwrap();
    re.replace_all(input, "").to_string()
}

fn strip_comments_and_format_spaces(input: &str) -> String {
    let comment_re = Regex::new(r"(?s)<!--.*?-->").unwrap();
    let space_re = Regex::new(r"\s{2,}").unwrap();
    let temp = comment_re.replace_all(input, "");
    let temp = temp.replace('\n', "").replace('\r', "");
    space_re.replace_all(&temp, " ").into_owned()
}

fn process_xml_files(base_dir: &Path, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;
    let file_re = Regex::new(r"^\d_.*\.xml$")?;

    let files: Vec<PathBuf> = WalkDir::new(base_dir)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| file_re.is_match(&e.file_name().to_string_lossy()))
        .map(|e| e.path().to_path_buf())
        .collect();

    if files.is_empty() {
        log_message("No XML files found to process.");
    }

    files.par_iter().for_each(|file| {
        match expand_includes(file, true) {
            Ok(expanded) => {
                let out_path = output_dir.join(file.file_name().unwrap());
                if let Err(err) = fs::write(&out_path, expanded) {
                    log_message(&format!("Error writing {}: {}", out_path.display(), err));
                } else {
                    log_message(&format!("Processed: {}", file.display()));
                }
            }
            Err(err) => {
                log_message(&format!("Error processing {}: {}", file.display(), err));
            }
        }
    });

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let (base_dir, output_dir) = if args.len() > 1 {
        let dir = PathBuf::from(&args[1]);
        if !dir.exists() {
            return Err(anyhow!("Specified directory does not exist: {}", dir.display()));
        }
        (dir.clone(), dir.join("compiled"))
    } else {
        let dir = env::current_dir()?;
        (dir.clone(), dir.join("compiled"))
    };

    log_section(&format!("Starting processing in {}", base_dir.display()));
    process_xml_files(&base_dir, &output_dir)?;
    log_section(&format!(
        "Processing complete. Compiled XMLs saved in {}",
        output_dir.display()
    ));

    Ok(())
}