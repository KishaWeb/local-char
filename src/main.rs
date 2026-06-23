mod tui;
mod web;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("web") => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(web::run());
        }

        Some("tui") => {
            tui::run();
        }

        _ => {
            println!("Usage:");
            println!("  local-char web");
            println!("  local-char tui");
            println!("if your using cargo");
            println!("  cargo run -- web");
            println!("  cargo run -- tui");
        }
    }
}