use testing::{
    ThreadPool,
    parse_request_content,
    create_file_if_needed
    };
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs::OpenOptions;
use std::io::Write;


fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    let pool = ThreadPool::new(100);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request_content = parse_request_content(&buffer);

    let get = b"GET / HTTP/1.1\r\n";

    let (status_line, contents) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "<!DOCTYPE html><html><body><h1>Stats</h1><p>When this works it should show stats.</p></body></html>")
    } else if buffer.starts_with(b"POST /stats HTTP/1.1") {
        // Build file name from request content
        let mut file_name = String::new();
        file_name.push_str("stats/");
        file_name.push_str(request_content.split(" ").collect::<Vec<_>>()[0]);
        file_name.push_str(".txt");

        create_file_if_needed(&file_name);

        let mut data_file = OpenOptions::new()
            .append(true)
            .open(file_name)
            .expect("cannot open file");

        let mut file_content = String::new();
        file_content.push_str(request_content);
        file_content.push_str("\n");
        data_file
            .write(file_content.as_bytes())
            .expect("write failed");
        ("HTTP/1.1 200 OK", "ok")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404")
    };

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
