use search::search;

mod db;
mod loading;
mod search;

fn main() {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_env();

    match parser.next().unwrap() {
        Some(Value(v)) if v == "build" => {
            db::create();
            return;
        }
        Some(Value(v)) if v == "search" => {
            let mut titles = vec![];
            while let Ok(Some(Value(arg))) = parser.next() {
                titles.push(arg.to_string_lossy().into_owned());
            }

            println!("Starting search with: {}", titles.join(", "));

            search(titles);
        }
        _ => {
            eprintln!("Must supply either build or search command");
        }
    }
}
