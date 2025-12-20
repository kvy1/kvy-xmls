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

fn expand_includes(file_path: &Path) -> Result<String> {
    let content = fs::read_to_string(file_path)?;
    let pattern = Regex::new(r#"<!--\s*#include file="(.*?)"\s*-->"#)?;
    let dir = file_path.parent().unwrap_or_else(|| Path::new("."));

    let replaced = pattern.replace_all(&content, |caps: &regex::Captures| {
        let include = caps[1].trim();
        let include_path = normalize_include_path(dir, include);
        if include_path.exists() {
            match expand_includes(&include_path) {
                Ok(included) => {
                    log_message(&format!("Included: {}", include_path.display()));
                    included
                }
                Err(err) => {
                    log_message(&format!(
                        "Error reading include {}: {}",
                        include_path.display(),
                        err
                    ));
                    format!("<!-- Include not found: {} -->", include_path.display())
                }
            }
        } else {
            log_message(&format!("Missing include: {}", include_path.display()));
            format!("<!-- Include not found: {} -->", include_path.display())
        }
    });

    let no_comments = remove_commented_lines(&replaced);
    let wrapped = wrap_placeholder_content(&no_comments);
    Ok(wrapped)
}

fn remove_commented_lines(input: &str) -> String {
    let pattern = Regex::new(r"<!--.*?-->").unwrap();
    input
        .lines()
        .filter(|line| !pattern.is_match(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn wrap_placeholder_content(input: &str) -> String {
    let pattern = Regex::new(r"(?i)<placeholder>(?s)(.*?)</placeholder>").unwrap();
    let cdata_regex = Regex::new(r"<!\[CDATA\[(.*?)\]\]>").unwrap();

    pattern
        .replace_all(input, |caps: &regex::Captures| {
            let inner = caps[1].trim();
            let inner = cdata_regex.replace_all(inner, "$1");
            format!("\n<![CDATA[\n{}\n]]>", inner)
        })
        .to_string()
}

fn process_xml_files(base_dir: &Path, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    let file_re = Regex::new(r"^\d_.*\.xml$")?;

    // Process only one folder deep (e.g., ./KFM/*.xml)
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
        match expand_includes(file) {
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