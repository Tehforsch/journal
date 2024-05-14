use std::path::Path;

use crate::entries::Entries;

mod config;
mod entries;

// use rouille::Response;

fn main() {
    println!("Now listening on localhost:8000");

    let entries = Entries::read(&Path::new(config::JOURNAL_PATH));
    dbg!(&entries);

    // rouille::start_server("localhost:8000", move |request| {
    //     {
    //         let response = rouille::match_assets(&request, ".");
    //         if response.is_success() {
    //             return response;
    //         }
    //     }

    //     Response::html(
    //         "404 error. Try <a href=\"/README.md\"`>README.md</a> or \
    //                     <a href=\"/src/lib.rs\">src/lib.rs</a> for example.",
    //     )
    //     .with_status_code(404)
    // });
}
