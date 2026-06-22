use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

pub fn run() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    println!("Server running on http://127.0.0.1:7878");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);

    let request_line = match buf_reader.lines().next() {
        Some(Ok(line)) => line,
        _ => return,
    };

    println!("{request_line}");

    // Serve JS
    if request_line.starts_with("GET /script.js") {
        let contents =
            fs::read_to_string("src/web/script.js").unwrap();

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: application/javascript\r\n\
             Content-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );

        stream.write_all(response.as_bytes()).unwrap();
        return;
    }

    // Serve HTML
    if request_line.starts_with("GET /") {
        let contents =
            fs::read_to_string("src/web/index.html").unwrap();

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html\r\n\
             Content-Length: {}\r\n\r\n{}",
            contents.len(),
            contents
        );

        stream.write_all(response.as_bytes()).unwrap();
        return;
    }

    // 404
    let response =
        "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n";

    stream.write_all(response.as_bytes()).unwrap();
}