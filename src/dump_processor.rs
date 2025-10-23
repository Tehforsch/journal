use crate::config;
use chrono::{DateTime, NaiveDateTime, Utc};
use id3::TagLike;
use std::fs;
use std::path::Path;

pub fn process_dump_folder() -> Result<(), Box<dyn std::error::Error>> {
    let dump_path = Path::new(config::DUMP_FOLDER_PATH);

    if !dump_path.exists() {
        fs::create_dir_all(dump_path)?;
        println!("Created dump folder: {}", config::DUMP_FOLDER_PATH);
        return Ok(());
    }

    let entries = fs::read_dir(dump_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            match process_file(&path) {
                Ok(target_date) => {
                    move_file_to_date_folder(&path, &target_date)?;
                    println!("Moved {} to {}", path.display(), target_date);
                }
                Err(e) => {
                    eprintln!("Failed to process {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(())
}

fn process_file(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());

    match extension.as_deref() {
        Some("jpg") | Some("jpeg") | Some("png") | Some("tiff") | Some("tif") => {
            extract_photo_date(file_path)
        }
        Some("mp3") | Some("m4a") | Some("flac") | Some("wav") => extract_audio_date(file_path),
        _ => get_file_creation_date(file_path),
    }
}

fn extract_photo_date(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // Try to extract EXIF date first
    if let Ok(file) = std::fs::File::open(file_path) {
        let mut bufreader = std::io::BufReader::new(&file);
        if let Ok(exifreader) = exif::Reader::new().read_from_container(&mut bufreader) {
            // Try different date fields in order of preference
            let date_fields = [
                exif::Tag::DateTimeOriginal,
                exif::Tag::DateTime,
                exif::Tag::DateTimeDigitized,
            ];

            for &tag in &date_fields {
                if let Some(field) = exifreader.get_field(tag, exif::In::PRIMARY) {
                    if let exif::Value::Ascii(ref vec) = field.value {
                        if let Some(ascii_val) = vec.first() {
                            let date_str = std::str::from_utf8(ascii_val)
                                .map_err(|_| "Invalid UTF-8 in EXIF date")?;

                            // EXIF dates are in format "YYYY:MM:DD HH:MM:SS"
                            if let Ok(naive_dt) =
                                NaiveDateTime::parse_from_str(date_str, "%Y:%m:%d %H:%M:%S")
                            {
                                let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc);
                                return Ok(dt.format("%Y-%m-%d").to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Fall back to file modification time if EXIF extraction fails
    get_file_creation_date(file_path)
}

fn extract_audio_date(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
        if extension.to_lowercase() == "mp3" {
            match id3::Tag::read_from_path(file_path) {
                Ok(tag) => {
                    if let Some(date_recorded) = tag.date_recorded() {
                        let year = date_recorded.year;
                        let month = date_recorded.month.unwrap_or(1);
                        let day = date_recorded.day.unwrap_or(1);
                        return Ok(format!("{:04}-{:02}-{:02}", year, month, day));
                    }

                    if let Some(year) = tag.year() {
                        return Ok(format!("{:04}-01-01", year));
                    }
                }
                Err(_) => {}
            }
        }
    }

    get_file_creation_date(file_path)
}

fn get_file_creation_date(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(file_path)?;
    let modified_time = metadata.modified()?;
    let datetime: DateTime<Utc> = modified_time.into();
    Ok(datetime.format("%Y-%m-%d").to_string())
}

fn move_file_to_date_folder(
    file_path: &Path,
    date_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let journal_path = Path::new(config::JOURNAL_PATH);
    let date_folder = journal_path.join(date_str);

    if !date_folder.exists() {
        fs::create_dir_all(&date_folder)?;

        let entry_md_path = date_folder.join("entry.md");
        if !entry_md_path.exists() {
            fs::write(&entry_md_path, format!("# {}\n\n", date_str))?;
        }
    }

    let file_name = file_path.file_name().unwrap();
    let extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase());

    let target_subfolder = match extension.as_deref() {
        Some("jpg") | Some("jpeg") | Some("png") | Some("tiff") | Some("tif") => "pics",
        Some("mp3") | Some("m4a") | Some("flac") | Some("wav") => "audio",
        _ => return Err("Unsupported file type".into()),
    };

    let target_folder = date_folder.join(target_subfolder);
    if !target_folder.exists() {
        fs::create_dir_all(&target_folder)?;
    }

    let target_path = target_folder.join(file_name);
    fs::rename(file_path, target_path)?;

    Ok(())
}
