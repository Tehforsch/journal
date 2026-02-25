use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use std::fs;
use std::io::stdout;
use std::path::{Path, PathBuf};

const JOURNAL_PATH: &str = "/home/toni/resource/journal";
const DUMP_FOLDER_PATH: &str = "/home/toni/resource/journal/dump";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dump_path = Path::new(DUMP_FOLDER_PATH);

    if !dump_path.exists() {
        fs::create_dir_all(dump_path)?;
        println!("Created dump folder: {}", DUMP_FOLDER_PATH);
        return Ok(());
    }

    let image_files: Vec<PathBuf> = fs::read_dir(dump_path)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|s| {
                        matches!(
                            s.to_lowercase().as_str(),
                            "jpg" | "jpeg" | "png" | "tiff" | "tif"
                        )
                    })
                    .unwrap_or(false)
        })
        .collect();

    if image_files.is_empty() {
        println!("No images in dump folder.");
        return Ok(());
    }

    let available_dates = get_available_dates()?;

    terminal::enable_raw_mode()?;
    let result = run_interactive(&image_files, &available_dates);
    terminal::disable_raw_mode()?;

    execute!(stdout(), cursor::Show)?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn get_available_dates() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let journal_path = Path::new(JOURNAL_PATH);
    let mut dates: Vec<String> = fs::read_dir(journal_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            e.file_name()
                .to_str()
                .filter(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok())
                .map(|s| s.to_string())
        })
        .collect();
    dates.sort();
    Ok(dates)
}

fn run_interactive(
    image_files: &[PathBuf],
    available_dates: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut last_selected_date: Option<String> = None;

    for image_path in image_files {
        let suggested_date = last_selected_date
            .clone()
            .or_else(|| extract_photo_date(image_path).ok())
            .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());

        let selected_date =
            interactive_date_selection(image_path, &suggested_date, available_dates)?;

        if let Some(date) = selected_date {
            move_file_to_date_folder(image_path, &date)?;
            println!("\r\nMoved {} to {}\r", image_path.display(), date);
            last_selected_date = Some(date);
        } else {
            println!("\r\nSkipped {}\r", image_path.display());
        }
    }

    Ok(())
}

fn interactive_date_selection(
    image_path: &Path,
    suggested_date: &str,
    available_dates: &[String],
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut current_date = NaiveDate::parse_from_str(suggested_date, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Utc::now().date_naive());

    loop {
        clear_screen()?;
        display_image(image_path)?;

        let date_str = current_date.format("%Y-%m-%d").to_string();
        let entry_exists = available_dates.contains(&date_str);
        let entry_content = if entry_exists {
            read_entry_content(&date_str)
        } else {
            None
        };

        println!("\r");
        println!(
            "Image: {}\r",
            image_path.file_name().unwrap().to_str().unwrap()
        );
        println!("\r");
        println!(
            "Date: {} {}\r",
            date_str,
            if entry_exists {
                "(entry exists)"
            } else {
                "(no entry)"
            }
        );
        println!("\r");

        if let Some(content) = entry_content {
            let preview: String = content.lines().take(10).collect::<Vec<_>>().join("\n");
            println!("Entry preview:\r");
            println!("─────────────────────────────────────\r");
            for line in preview.lines() {
                println!("{}\r", line);
            }
            println!("─────────────────────────────────────\r");
        }

        println!("\r");
        println!("Controls: ←/h (prev day) | →/l (next day) | j/↓ (prev week) | k/↑ (next week) | Enter (confirm) | s (skip) | q (quit)\r");

        if let Event::Key(key_event) = event::read()? {
            match (key_event.code, key_event.modifiers) {
                (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Char('q'), _) => {
                    return Ok(None);
                }
                (KeyCode::Enter, _) => {
                    return Ok(Some(date_str));
                }
                (KeyCode::Char('s'), _) => {
                    return Ok(None);
                }
                (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                    current_date = current_date
                        .checked_sub_signed(chrono::Duration::days(1))
                        .unwrap_or(current_date);
                }
                (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                    current_date = current_date
                        .checked_add_signed(chrono::Duration::days(1))
                        .unwrap_or(current_date);
                }
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    current_date = current_date
                        .checked_add_signed(chrono::Duration::days(7))
                        .unwrap_or(current_date);
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    current_date = current_date
                        .checked_sub_signed(chrono::Duration::days(7))
                        .unwrap_or(current_date);
                }
                _ => {}
            }
        }
    }
}

fn clear_screen() -> Result<(), Box<dyn std::error::Error>> {
    execute!(
        stdout(),
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;
    Ok(())
}

fn display_image(image_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let img = match image::open(image_path) {
        Ok(img) => img,
        Err(e) => {
            println!("Could not load image: {}\r", e);
            return Ok(());
        }
    };

    let resized = img.thumbnail(400, 400);

    let conf = viuer::Config {
        width: Some(80),
        height: Some(20),
        absolute_offset: false,
        ..Default::default()
    };

    if let Err(e) = viuer::print(&resized, &conf) {
        println!("Could not display image: {}\r", e);
    }

    Ok(())
}

fn read_entry_content(date_str: &str) -> Option<String> {
    let entry_path = Path::new(JOURNAL_PATH).join(date_str).join("entry.md");
    fs::read_to_string(&entry_path)
        .ok()
        .filter(|s| !s.trim().is_empty())
}

fn extract_photo_date(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(file) = std::fs::File::open(file_path) {
        let mut bufreader = std::io::BufReader::new(&file);
        if let Ok(exifreader) = exif::Reader::new().read_from_container(&mut bufreader) {
            let date_fields = [
                exif::Tag::DateTimeOriginal,
                exif::Tag::DateTime,
                exif::Tag::DateTimeDigitized,
            ];

            for &tag in &date_fields {
                if let Some(field) = exifreader.get_field(tag, exif::In::PRIMARY) {
                    if let exif::Value::Ascii(ref vec) = field.value {
                        if let Some(ascii_val) = vec.first() {
                            let date_str = std::str::from_utf8(ascii_val)?;
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
    let journal_path = Path::new(JOURNAL_PATH);
    let date_folder = journal_path.join(date_str);

    if !date_folder.exists() {
        fs::create_dir_all(&date_folder)?;
        let entry_md_path = date_folder.join("entry.md");
        if !entry_md_path.exists() {
            fs::write(&entry_md_path, "")?;
        }
    }

    let file_name = file_path.file_name().unwrap();
    let target_folder = date_folder.join("pics");

    if !target_folder.exists() {
        fs::create_dir_all(&target_folder)?;
    }

    let target_path = target_folder.join(file_name);
    fs::rename(file_path, target_path)?;

    Ok(())
}
