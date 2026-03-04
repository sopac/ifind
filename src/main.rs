use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use time::{Date, OffsetDateTime};
use walkdir::WalkDir;

type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Serialize, Deserialize)]
struct FileEntry {
    filename: String,
    folder: String,
    extension: String,
    modified: Date,
}

#[derive(Debug, Parser)]
#[command(name = "ifind")]
struct Cli {
    query: Option<String>,
    #[arg(short, long, requires = "query")]
    extension: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Update {
        directory: PathBuf,
    },
    Search {
        query: String,
        #[arg(short, long)]
        extension: Option<String>,
    },
    Clear {},
}

fn main() -> AppResult<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Update { directory }) => update_index(&directory),
        Some(Commands::Search { query, extension }) => search_index(&query, extension.as_deref()),
        Some(Commands::Clear {}) => clear_index(),
        None => search_index(
            cli.query.as_deref().ok_or("query is required")?,
            cli.extension.as_deref(),
        ),
    }
}

fn clear_index() -> AppResult<()> {
    let index_path = cache_file_path()?;
    if index_path.exists() {
        fs::remove_file(index_path)?;
        println!("Index Cleared.");
    } else {
        println!("Index Not Found.");
    }
    Ok(())
}

fn update_index(directory: &Path) -> AppResult<()> {
    let mut entries = Vec::new();

    let index_path = cache_file_path()?;

    // append existing serialization
    if index_path.exists() {
        let bytes = fs::read(&index_path)?;
        entries = serde_cbor::from_slice(&bytes)?;
    }

    for entry in WalkDir::new(directory) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }

        let metadata = entry.metadata()?;
        let modified = system_time_to_date(metadata.modified()?)?;
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().into_owned();
        let folder = path
            .parent()
            .map(|parent| parent.to_string_lossy().into_owned())
            .unwrap_or_default();
        let extension = path
            .extension()
            .map(|ext| ext.to_string_lossy().into_owned())
            .unwrap_or_default();

        let ignore_list: Vec<&str> = vec![".git", ".build", ".venv", "target/"];

        if !ignore_list.iter().any(|ignore| folder.contains(ignore)) {
            entries.push(FileEntry {
                filename,
                folder,
                extension,
                modified,
            });
        }
    }

    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = serde_cbor::to_vec(&entries)?;
    fs::write(&index_path, bytes)?;
    println!(
        "Indexed {} files at {}",
        entries.len(),
        index_path.display()
    );
    Ok(())
}

fn search_index(query: &str, extension: Option<&str>) -> AppResult<()> {
    let index_path = cache_file_path()?;

    if !index_path.exists() {
        println!("Index Not Found. Please Run 'ifind update <directory>' First.");
        return Ok(());
    }

    let bytes = fs::read(&index_path)?;
    let mut entries: Vec<FileEntry> = serde_cbor::from_slice(&bytes)?;
    let query = query.to_lowercase();
    let extension = extension.map(normalize_extension);

    entries.retain(|entry| {
        let matches_query = entry.filename.to_lowercase().contains(&query)
            || entry.folder.to_lowercase().contains(&query);
        let matches_extension = extension.as_ref().map_or(true, |ext| {
            entry
                .extension
                .eq_ignore_ascii_case(ext.trim_start_matches('.'))
        });
        matches_query && matches_extension
    });

    entries.sort_by_key(|entry| Reverse(entry.modified));
    for entry in entries {
        let mut folder = entry.folder;
        folder = folder.replace("/share/", "");
        folder = folder.replace("/Users/sachin/tmp/", "");

        println!(
            "{}\t{}\t{}\t{}",
            entry.modified, entry.extension, folder, entry.filename
        );
    }

    Ok(())
}

fn system_time_to_date(system_time: SystemTime) -> AppResult<Date> {
    let dt = OffsetDateTime::from(system_time);
    Ok(dt.date())
}

fn cache_file_path() -> AppResult<PathBuf> {
    let home = std::env::var_os("HOME").ok_or("HOME environment variable is not set")?;
    Ok(PathBuf::from(home).join(".cache").join("ifind.cbor"))
}

fn normalize_extension(extension: &str) -> String {
    extension.trim_start_matches('.').to_ascii_lowercase()
}
