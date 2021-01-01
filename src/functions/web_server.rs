use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    path::Path,
    sync::mpsc::Sender,
    thread,
};

// Import the functions from the same level file
use super::server_interactions;
use super::shared_data;

pub fn handle_connections(
    data: shared_data::ServerSharedData,
    web_sender: Sender<String>,
) -> std::io::Result<()> {
    loop {
        let listener = TcpListener::bind("0.0.0.0:8000")?;
        let data2 = data.clone();
        let sender = web_sender.clone();
        // For each request create a thread to parse request and send contents
        for stream in listener.incoming() {
            let data3 = data2.clone();
            let sender = sender.clone();
            thread::spawn(move || {
                let data4 = data3.clone();
                let sender = sender.clone();
                let mut stream = stream.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                match line.find("/") {
                    // Every proper request line should contain a forward slash
                    Some(start) => {
                        let request = &line[start..line.find("HTTP").unwrap() - 1];
                        stream
                            .write_all(generate_response(request, data4, sender).as_bytes())
                            .unwrap();
                    }
                    None => {
                        println!("\x1b[0;33m[Request]:\x1b[0m Empty Request Recieved");
                    }
                }
            });
        }
    }
}

fn generate_response(
    request: &str,
    data: shared_data::ServerSharedData,
    web_sender: Sender<String>,
) -> String {
    let default_http_header = "HTTP/1.1 200 OK\r\nConnection: Close\r\nContent-Type:";
    let headers404 = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nConnection: Close";
    println!("\x1b[0;33m[Request]:\x1b[0m {}", request);
    match request {
        "/" => format!(
            "{} text/html\r\n\r\n{}",
            default_http_header,
            get_file_contents("/home.html")
        ),
        "/data/players" => server_interactions::get_players(
            data.current_player_count,
            data.max_player_count,
            data.current_players,
        ),
        "/data/console" => server_interactions::get_console(data.server_output),
        _ => {
            if request.len() > 11 as usize && &request[0..11] == "/data/send?" {
                server_interactions::send_command(&request[10..], web_sender)
            } else {
                if Path::new(
                    format!("../public/{}/{}", get_file_folder(request), &request[1..]).as_str(),
                )
                .exists()
                {
                    format!(
                        "{} {}\r\n\r\n{}",
                        default_http_header,
                        get_file_type(request),
                        get_file_contents(request)
                    )
                } else {
                    headers404.to_string()
                }
            }
        }
    }
}

fn get_file_type(path: &str) -> &str {
    let ext = &path[path.find(".").unwrap()..];
    match ext {
        ".html" => "text/html",
        ".png" => "image/png",
        ".jpg" | ".jpeg" => "image/jpeg",
        ".gif" => "image/gif",
        ".ico" => "image/x-icon",
        _ => "text/plain",
    }
}

fn get_file_folder(path: &str) -> &str {
    let ext = &path[path.find(".").unwrap()..];
    // println!("{} - {}", path, ext);
    match ext {
        ".html" => "html",
        ".css" => "css",
        ".js" => "javascript",
        _ => "images",
    }
}

fn get_file_contents(path: &str) -> String {
    fs::read_to_string(format!(
        "../public/{}/{}",
        get_file_folder(path),
        &path[1..]
    ))
    .expect(format!("Failed to read file: {}", path).as_str())
}

#[cfg(test)]
mod tests {
    #[test]
    fn sample() {
        assert_eq!(2 + 2, 4, "sample message");
    }
}
