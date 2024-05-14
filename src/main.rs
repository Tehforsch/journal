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
                    manager.dashboard()
                },
                _ => {
                    let response = rouille::match_assets(&request, config::JOURNAL_PATH);
                    if response.is_success() {
                        response
                    }
                    else {
                    Response::html(
                        "404 error.",
                    )
                    .with_status_code(404)
                    }
                }
            )
        }
    });
}

impl Manager {
    fn dashboard(&self) -> Response {
        Response::html(self.dashboard_html())
    }

    fn dashboard_html(&self) -> String {
        let mut context = tera::Context::new();
        let num_entries = 3;
        let result = (0..num_entries)
            .map(|_| {
                let random_entry = self.entries.random();
                random_entry
                    .map(|entry| self.entry_html(entry))
                    .unwrap_or("".to_owned())
            })
            .collect::<Vec<_>>()
            .join("\n");
        context.insert("entries", &result);
        self.tera.render("dashboard.html", &context).unwrap()
    }

    fn entry_html(&self, entry: &Entry) -> String {
        let mut context = tera::Context::new();
        context.insert(
            "content",
            &entry.content().unwrap().replace("\n", "\n<br/>"),
        );
        context.insert("date", &entry.date_str());
        context.insert("pics", &self.pics_html(entry));
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
}
