use std::fs;
use std::io::{Read, Seek};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;
use epub::doc::EpubDoc;
use pdf::{file::FileOptions, object::InfoDict};

struct FileMetadata {
    author: Option<String>,
    title: Option<String>,
}

impl From<InfoDict> for FileMetadata {
    fn from(value: InfoDict) -> Self {
        FileMetadata {
            author: value.author.and_then(|v| v.to_string().ok()),
            title: value.title.and_then(|v| v.to_string().ok()),
        }
    }
}

impl<R: Read + Seek> From<EpubDoc<R>> for FileMetadata {
    fn from(value: EpubDoc<R>) -> Self {
        FileMetadata {
            author: value.mdata("creator"),
            title: value.mdata("title"),
        }
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Slugify output file names
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    slugify: bool,

    /// Dry run mode
    #[arg(short = 'n', long, action = clap::ArgAction::SetTrue)]
    dry_run: bool,

    /// Files to rename
    files: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let args = Cli::parse();

    match args
        .files
        .iter()
        .try_for_each(|f| process_file(f, args.slugify, args.dry_run))
    {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn process_file(old_file: &PathBuf, slugify: bool, dry_run: bool) -> Result<(), ()> {
    let metadata = match get_file_metadata(old_file) {
        Ok(m) => m,
        Err(e) => {
            println!("Could not process file: {}", e);
            return Ok(());
        }
    };

    let extension = old_file.as_path().extension().unwrap().to_str().unwrap();
    let author = match metadata.author {
        Some(a) => format!("{a} - "),
        None => String::new(),
    };
    let title = metadata.title.unwrap_or(String::new());

    let new_filename = match (format!("{author}{title}"), slugify) {
        (fname, true) => slug::slugify(fname),
        (fname, false) => fname,
    };
    let new_filename = format!("{new_filename}.{extension}");

    let new_filepath = old_file.as_path().with_file_name(new_filename);

    if old_file == &new_filepath {
        println!("Nothing to do, skipping: {}", old_file.display());
        return Ok(());
    }

    println!(
        "Renaming {} -> {}",
        old_file.display(),
        new_filepath.display()
    );

    if dry_run {
        return Ok(());
    }

    fs::rename(old_file, &new_filepath).map_err(|e| {
        println!(
            "Failed to rename {} to {}: {}",
            old_file.display(),
            new_filepath.display(),
            e
        );
    })?;

    Ok(())
}

fn get_file_metadata(file: &PathBuf) -> anyhow::Result<FileMetadata> {
    let extension = file
        .as_path()
        .extension()
        .map(|s| s.to_ascii_lowercase())
        .with_context(|| format!("no extension: {}", file.display()))?;

    match extension.to_str().unwrap_or("") {
        "epub" => {
            let epub = EpubDoc::new(file).with_context(|| format!("file: {}", file.display()))?;
            Ok(FileMetadata::from(epub))
        }
        "pdf" => {
            let pdf = FileOptions::cached()
                .open(file)
                .with_context(|| format!("file: {}", file.display()))?;
            match pdf.trailer.info_dict {
                Some(i) => Ok(FileMetadata::from(i)),
                None => Err(anyhow::anyhow!("missing metadata: {}", file.display())),
            }
        }
        _ => Err(anyhow::anyhow!("unknown extension: {}", file.display())),
    }
}
