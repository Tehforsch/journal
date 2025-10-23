use std::path::Path;

use entries::Entry;
use rouille::{router, Response};
use tera::Tera;

use crate::entries::Entries;

mod config;
mod entries;

struct Manager {
    entries: Entries,
    tera: Tera,
}

fn main() {
    println!("Now listening on localhost:8000");

    let entries = Entries::read(&Path::new(config::JOURNAL_PATH)).unwrap();
    let mut tera = Tera::default();
    // Embedding these here for simplicity, so I can just run the binary from anywhere
    tera.add_raw_template("pic.html", include_str!("../templates/pic.html"))
        .unwrap();
    tera.add_raw_template("audio.html", include_str!("../templates/audio.html"))
        .unwrap();
    tera.add_raw_template("entry.html", include_str!("../templates/entry.html"))
        .unwrap();
    tera.add_raw_template(
        "dashboard.html",
        include_str!("../templates/dashboard.html"),
    )
    .unwrap();
    tera.autoescape_on(vec![]);
    let manager = Manager { entries, tera };

    rouille::start_server("localhost:8000", move |request| {
        {
            router!(request,
                (GET) (/dashboard) => {
                    Response::html(manager.dashboard_html())
                },
                (GET) (/{date: String}) => {
                    manager.entry_for_date(date)
                },
                _ => {
                    let response = rouille::match_assets(&request, config::JOURNAL_PATH);
                    if response.is_success() {
                        response
                    }
                    else {
                        manager.response_404()
                    }
                }
            )
        }
    });
}

impl Manager {
    fn response_404(&self) -> Response {
        Response::html("404 error.").with_status_code(404)
    }

    fn entry_for_date(&self, date: String) -> Response {
        let entry = self.entries.get_by_date(date);
        if let Some(entry) = entry {
            Response::html(self.entry_html(entry))
        } else {
            self.response_404()
        }
    }

    fn dashboard_html(&self) -> String {
        let mut context = tera::Context::new();
        let num_entries = 3;
        let result = (0..num_entries)
            .map(|_| {
                let random_entry = self.entries.random();
                random_entry
                    .map(|entry| self.dashboard_entry_preview(entry))
                    .unwrap_or("".to_owned())
            })
            .collect::<Vec<_>>()
            .join("\n");
        context.insert("entries", &result);
        self.tera.render("dashboard.html", &context).unwrap()
    }

    fn dashboard_entry_preview(&self, entry: &Entry) -> String {
        let content = entry.content().unwrap_or_default();
        let preview = content;

        format!(
            r#"<div class="entry-preview">
                <div style="font-weight: 600; color: #4facfe; margin-bottom: 0.5rem;">
                    <a href="/{}" style="text-decoration: none; color: inherit;">{}</a>
                </div>
                <div style="color: #6c757d; line-height: 1.6;">
                    {}
                </div>
            </div>"#,
            entry.date_str(),
            entry.date_str(),
            preview.replace("\n", "<br/>")
        )
    }

    fn entry_html(&self, entry: &Entry) -> String {
        let mut context = tera::Context::new();
        context.insert(
            "content",
            &entry.content().unwrap().replace("\n", "\n<br/>"),
        );
        context.insert("date", &entry.date_str());
        context.insert("pics", &self.pics_html(entry));
        context.insert("audio", &self.audio_html(entry));
        let prev = self.entries.prev(entry);
        let next = self.entries.next(entry);
        context.insert("link_entry", &self.entry_link(entry));
        context.insert("link_prev", &self.entry_link(prev.unwrap_or(entry)));
        context.insert("link_next", &self.entry_link(next.unwrap_or(entry)));
        self.tera.render("entry.html", &context).unwrap()
    }

    fn pics_html(&self, entry: &Entry) -> String {
        if entry.pics().is_empty() {
            "".to_string()
        } else {
            entry
                .pics()
                .iter()
                .map(|pic| {
                    let mut context = tera::Context::new();
                    context.insert("pic", pic);
                    self.tera.render("pic.html", &context).unwrap()
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    fn audio_html(&self, entry: &Entry) -> String {
        if entry.audio().is_empty() {
            "".to_string()
        } else {
            entry
                .audio()
                .iter()
                .map(|audio| {
                    let mut context = tera::Context::new();
                    context.insert("audio", audio);
                    self.tera.render("audio.html", &context).unwrap()
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    fn entry_link(&self, prev: &Entry) -> String {
        prev.date_str()
    }
}
