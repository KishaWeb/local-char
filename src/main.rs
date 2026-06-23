mod tui;
mod web;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let lan = args.iter().any(|a| a == "--lan");

    match args.get(1).map(String::as_str) {
        Some("web") => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(web::run(lan));
        }

        Some("tui") => {
            tui::run();
        }

        _ => {
            println!("Usage:");
            println!("  local-char web [--lan]");
            println!("  local-char tui");
            println!("if you're using cargo:");
            println!("  cargo run -- web [--lan]");
            println!("  cargo run -- tui");
        }
    }
}