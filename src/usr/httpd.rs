use crate::sys;
use crate::api::clock;
use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;
use alloc::collections::vec_deque::VecDeque;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use smoltcp::socket::{TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::phy::Device;
use time::OffsetDateTime;

struct Response {
    code: u16,
    mime: String,
    date: String,
    body: Vec<u8>,
    raw: Vec<u8>,
}

impl Response {
    pub fn new() -> Self {
        Self {
            code: 0,
            mime: String::new(),
            date: strftime("%d/%b/%Y:%H:%M:%S %z"),
            body: Vec::new(),
            raw: Vec::new(),
        }
    }
}

pub fn main(_args: &[&str]) -> Result<(), ExitCode> {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    let port = 80;
    let root = sys::process::dir();

    if let Some(ref mut iface) = *sys::net::IFACE.lock() {
        println!("{}HTTP Server listening on 0.0.0.0:{}{}", csi_color, port, csi_reset);

        let mtu = iface.device().capabilities().max_transmission_unit;
        let tcp_rx_buffer = TcpSocketBuffer::new(vec![0; mtu]);
        let tcp_tx_buffer = TcpSocketBuffer::new(vec![0; mtu]);
        let tcp_socket = TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer);
        let tcp_handle = iface.add_socket(tcp_socket);

        let mut send_queue: VecDeque<Vec<u8>> = VecDeque::new();
        loop {
            if sys::console::end_of_text() || sys::console::end_of_transmission() {
                iface.remove_socket(tcp_handle);
                println!();
                return Ok(());
            }

            let timestamp = Instant::from_micros((clock::realtime() * 1000000.0) as i64);
            if let Err(e) = iface.poll(timestamp) {
                error!("Network Error: {}", e);
            }

            let socket = iface.get_socket::<TcpSocket>(tcp_handle);

            if !socket.is_open() {
                socket.listen(port).unwrap();
            }
            let addr = socket.remote_endpoint().addr;
            if socket.may_recv() {
                let res = socket.recv(|buffer| {
                    let mut res = Response::new();
                    let req = String::from_utf8_lossy(buffer);
                    if !req.is_empty() {
                        let mut verb = "";
                        let mut path = "";
                        let mut header = true;
                        let mut contents = String::new();
                        for (i, line) in req.lines().enumerate() {
                            if i == 0 {
                                let fields: Vec<_> = line.split(' ').collect();
                                if fields.len() >= 2 {
                                    verb = fields[0];
                                    path = fields[1];
                                }
                            } else if header && line.is_empty() {
                                header = false;
                            } else if !header {
                                contents.push_str(&format!("{}\n", line));
                            }
                        }

                        let real_path = if path == "/" {
                            root.clone()
                        } else {
                            format!("{}/{}", root, path)
                        }.replace("//", "/");

                        match verb {
                            "GET" => {
                                if path.len() > 1 && path.ends_with('/') {
                                    res.code = 301;
                                    res.mime = "text/html".to_string();
                                    res.raw.extend_from_slice(b"HTTP/1.0 301 Moved Permanently\r\n");
                                    res.raw.extend_from_slice(&format!("Location: {}\r\n", path.strip_suffix('/').unwrap()).as_bytes());
                                    res.body.extend_from_slice(b"<h1>Moved Permanently</h1>\r\n");
                                } else if let Ok(mut files) = fs::read_dir(&real_path) {
                                    res.code = 200;
                                    res.mime = "text/html".to_string();
                                    res.raw.extend_from_slice(b"HTTP/1.0 200 OK\r\n");
                                    res.body.extend_from_slice(&format!("<h1>Index of {}</h1>\r\n", path).as_bytes());
                                    files.sort_by_key(|f| f.name());
                                    for file in files {
                                        let sep = if path == "/" { "" } else { "/" };
                                        let path = format!("{}{}{}", path, sep, file.name());
                                        res.body.extend_from_slice(&format!("<li><a href=\"{}\">{}</a></li>\n", path, file.name()).as_bytes());
                                    }
                                } else if let Ok(buf) = fs::read_to_bytes(&real_path) {
                                    res.code = 200;
                                    res.mime = content_type(&real_path);
                                    res.raw.extend_from_slice(b"HTTP/1.0 200 OK\r\n");
                                    let tmp;
                                    res.body.extend_from_slice(if res.mime.starts_with("text/") {
                                        tmp = String::from_utf8_lossy(&buf).to_string().replace("\n", "\r\n");
                                        tmp.as_bytes()
                                    } else {
                                        &buf
                                    });
                                } else {
                                    res.code = 404;
                                    res.mime = "text/html".to_string();
                                    res.raw.extend_from_slice(b"HTTP/1.0 404 Not Found\r\n");
                                    res.body.extend_from_slice(b"<h1>Not Found</h1>\r\n");
                                }
                            },
                            "PUT" => {
                                if real_path.ends_with('/') { // Write directory
                                    let real_path = real_path.trim_end_matches('/');
                                    if fs::exists(&real_path) {
                                        res.code = 403;
                                        res.raw.extend_from_slice(b"HTTP/1.0 403 Forbidden\r\n");
                                    } else if let Some(handle) = fs::create_dir(&real_path) {
                                        syscall::close(handle);
                                        res.code = 200;
                                        res.raw.extend_from_slice(b"HTTP/1.0 200 OK\r\n");
                                    } else {
                                        res.code = 500;
                                        res.raw.extend_from_slice(b"HTTP/1.0 500 Internal Server Error\r\n");
                                    }
                                } else { // Write file
                                    if fs::write(&real_path, contents.as_bytes()).is_ok() {
                                        res.code = 200;
                                        res.raw.extend_from_slice(b"HTTP/1.0 200 OK\r\n");
                                    } else {
                                        res.code = 500;
                                        res.raw.extend_from_slice(b"HTTP/1.0 500 Internal Server Error\r\n");
                                    }
                                }
                                res.mime = "text/plain".to_string();
                            },
                            "DELETE" => {
                                if fs::exists(&real_path) {
                                    if fs::delete(&real_path).is_ok() {
                                        res.code = 200;
                                        res.raw.extend_from_slice(b"HTTP/1.0 200 OK\r\n");
                                    } else {
                                        res.code = 500;
                                        res.raw.extend_from_slice(b"HTTP/1.0 500 Internal Server Error\r\n");
                                    }
                                } else {
                                    res.code = 404;
                                    res.raw.extend_from_slice(b"HTTP/1.0 404 Not Found\r\n");
                                }
                                res.mime = "text/plain".to_string();
                            },
                            _ => {
                                res.code = 400;
                                res.mime = "text/html".to_string();
                                res.raw.extend_from_slice(b"HTTP/1.0 400 Bad Request\r\n");
                                res.body.extend_from_slice(b"<h1>Bad Request</h1>\r\n");
                            },
                        }
                        let size = res.body.len();
                        res.raw.extend_from_slice(&format!("Server: MOROS/{}\r\n", env!("CARGO_PKG_VERSION")).as_bytes());
                        res.raw.extend_from_slice(&format!("Date: {}\r\n", strftime("%a, %d %b %Y %H:%M:%S GMT")).as_bytes());
                        if res.mime.starts_with("text/") {
                            res.raw.extend_from_slice(&format!("Content-Type: {}; charset=utf-8\r\n", res.mime).as_bytes());
                        } else {
                            res.raw.extend_from_slice(&format!("Content-Type: {}\r\n", res.mime).as_bytes());
                        }
                        res.raw.extend_from_slice(&format!("Content-Length: {}\r\n", size).as_bytes());
                        res.raw.extend_from_slice(b"Connection: close\r\n");
                        res.raw.extend_from_slice(b"\r\n");
                        res.raw.extend_from_slice(&res.body);
                        println!("{} - - [{}] \"{} {}\" {} {}", addr, res.date, verb, path, res.code, size);
                    }
                    (buffer.len(), res)
                }).unwrap();
                for chunk in res.raw.chunks(mtu) {
                    send_queue.push_back(chunk.to_vec());
                }
                if socket.can_send() {
                    if let Some(chunk) = send_queue.pop_front() {
                        socket.send_slice(&chunk).unwrap();
                    }
                }
            } else if socket.may_send() {
                socket.close();
                send_queue.clear();
            }
            if let Some(wait_duration) = iface.poll_delay(timestamp) {
                syscall::sleep((wait_duration.total_micros() as f64) / 1000000.0);
            }
        }
    } else {
        error!("Could not find network interface");
        Err(ExitCode::Failure)
    }
}

fn strftime(format: &str) -> String {
    let timestamp = clock::realtime();
    OffsetDateTime::from_unix_timestamp(timestamp as i64).format(format)
}

fn content_type(path: &str) -> String {
    let ext = path.rsplit_once('.').unwrap_or(("", "")).1;
    match ext {
        "html" | "htm" => "text/html",
        "txt"          => "text/plain",
        "png"          => "image/png",
        "jpeg" | "jpg" => "image/jpeg",
        _              => "application/octet-stream",
    }.to_string()
}
