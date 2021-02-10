use std::env;
use std::fs;
use std::path::Path;
use std::str;
use std::thread;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use regex::Regex;

fn main() {
    let doc_root: String = env::var("DOCUMENT_ROOT").unwrap_or(String::from(env::current_dir().unwrap().to_str().unwrap()));
    let host: String = env::var("HOST").unwrap_or(String::from("localhost"));
    let port: String = env::var("PORT").unwrap_or(String::from("7878"));

    let listen = format!("{}:{}", host, port);

    // listen は borrow させて ownership は委譲しない
    let ret = TcpListener::bind(&listen);
    if let Err(_) = ret {
        return println!("can not listen {}", listen);
    }

    for stream in ret.unwrap().incoming() {
        // イテレーションごとに clone して ownership を明確にする
        let doc_root_copy = doc_root.clone();
        // spawn に doc_root_copy を move する
        thread::spawn(move || {
            // doc_root_copy は borrow せずに ownership を移す
            handle_connection(stream.unwrap(), doc_root_copy);
        });
    }
}

fn handle_connection(mut stream: TcpStream, doc_root: String) {
    let mut buffer = [0; 1024];
    if let Err(_) = stream.read(&mut buffer) {
        return println!("stream read error");
    };

    let lines = str::from_utf8(&buffer);
    if let Err(_) = lines {
        return println!("from utf8 error");
    }

    let re = Regex::new(r"^GET ([^\s]+)").unwrap();
    let cap = re.captures(lines.unwrap());
    if let None = cap {
        return println!("none captured");
    }

    let uri = cap.unwrap();
    let mut path = format!("{}{}", &doc_root, &uri[1]);
    let path_strc = Path::new(&path);
    if !path_strc.exists() {
        return println!("path {} not exists", &path);
    }

    if path_strc.is_dir() {
        path = format!("{}index.html", &path);
    }

    let contents = fs::read_to_string(&path);
    if let Err(_) = contents {
        return println!("can not read {}", &path);
    }

    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents.unwrap());
    if let Err(_) = stream.write(response.as_bytes()) {
        println!("failed to write stream");
    }

    if let Err(_) = stream.flush() {
        println!("failed to flush stream");
    }
}
