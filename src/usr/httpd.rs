use crate::sys;
use crate::api::clock;
use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;

use alloc::collections::btree_map::BTreeMap;
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
    buf: Vec<u8>,
    code: usize,
    size: usize,
    mime: String,
    date: String,
    body: Vec<u8>,
    headers: BTreeMap<String, String>,
}

impl Response {
    pub fn new() -> Self {
        let mut headers = BTreeMap::new();
        headers.insert("Server".to_string(), format!("MOROS/{}", env!("CARGO_PKG_VERSION")));
        headers.insert("Date".to_string(), strftime("%a, %d %b %Y %H:%M:%S GMT"));
        Self {
            buf: Vec::new(),
            code: 0,
            size: 0,
            mime: String::new(),
            date: strftime("%d/%b/%Y:%H:%M:%S %z"),
            body: Vec::new(),
            headers,
        }
    }

    pub fn write(&mut self) {
        let status = match self.code {
            200 => "OK",
            301 => "Moved Permanently",
            400 => "Bad Request",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _   => "Unknown Error",
        }.to_string();

        self.headers.insert("Content-Type".to_string(), if self.mime.starts_with("text/") {
            format!("{}; charset=utf-8", self.mime)
        } else {
            format!("{}", self.mime)
        });

        self.size = self.body.len();
        self.headers.insert("Content-Length".to_string(), self.size.to_string());
        self.headers.insert("Connection".to_string(), "close".to_string());

        self.buf.clear();
        self.buf.extend_from_slice(&format!("HTTP/1.0 {} {}\r\n", self.code, status).as_bytes());
        for (key, val) in &self.headers {
            self.buf.extend_from_slice(&format!("{}: {}\r\n", key, val).as_bytes());
        }

        self.buf.extend_from_slice(b"\r\n");
        self.buf.extend_from_slice(&self.body);
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
                                    res.headers.insert("Location".to_string(), path.strip_suffix('/').unwrap().to_string());
                                    res.body.extend_from_slice(b"<h1>Moved Permanently</h1>\r\n");
                                } else {
                                    let mut not_found = true;
                                    for autocomplete in vec!["", "/index.html", "/index.htm", "/index.txt"] {
                                        let real_path = format!("{}{}", real_path, autocomplete);
                                        if fs::is_dir(&real_path) {
                                            continue;
                                        }
                                        if let Ok(buf) = fs::read_to_bytes(&real_path) {
                                            res.code = 200;
                                            res.mime = content_type(&real_path);
                                            let tmp;
                                            res.body.extend_from_slice(if res.mime.starts_with("text/") {
                                                tmp = String::from_utf8_lossy(&buf).to_string().replace("\n", "\r\n");
                                                tmp.as_bytes()
                                            } else {
                                                &buf
                                            });
                                            not_found = false;
                                            break;
                                        }
                                    }
                                    if not_found {
                                        if let Ok(mut files) = fs::read_dir(&real_path) {
                                            res.code = 200;
                                            res.mime = "text/html".to_string();
                                            res.body.extend_from_slice(&format!("<h1>Index of {}</h1>\r\n", path).as_bytes());
                                            files.sort_by_key(|f| f.name());
                                            for file in files {
                                                let sep = if path == "/" { "" } else { "/" };
                                                let path = format!("{}{}{}", path, sep, file.name());
                                                let link = format!("<li><a href=\"{}\">{}</a></li>\n", path, file.name());
                                                res.body.extend_from_slice(&link.as_bytes());
                                            }
                                        } else {
                                            res.code = 404;
                                            res.mime = "text/html".to_string();
                                            res.body.extend_from_slice(b"<h1>Not Found</h1>\r\n");
                                        }
                                    }
                                }
                            },
                            "PUT" => {
                                if real_path.ends_with('/') { // Write directory
                                    let real_path = real_path.trim_end_matches('/');
                                    if fs::exists(&real_path) {
                                        res.code = 403;
                                    } else if let Some(handle) = fs::create_dir(&real_path) {
                                        syscall::close(handle);
                                        res.code = 200;
                                    } else {
                                        res.code = 500;
                                    }
                                } else { // Write file
                                    if fs::write(&real_path, contents.as_bytes()).is_ok() {
                                        res.code = 200;
                                    } else {
                                        res.code = 500;
                                    }
                                }
                                res.mime = "text/plain".to_string();
                            },
                            "DELETE" => {
                                if fs::exists(&real_path) {
                                    if fs::delete(&real_path).is_ok() {
                                        res.code = 200;
                                    } else {
                                        res.code = 500;
                                    }
                                } else {
                                    res.code = 404;
                                }
                                res.mime = "text/plain".to_string();
                            },
                            _ => {
                                res.code = 400;
                                res.mime = "text/html".to_string();
                                res.body.extend_from_slice(b"<h1>Bad Request</h1>\r\n");
                            },
                        }
                        res.write();
                        println!("{} - - [{}] \"{} {}\" {} {}", addr, res.date, verb, path, res.code, res.size);
                    }
                    (buffer.len(), res)
                }).unwrap();
                for chunk in res.buf.chunks(mtu) {
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
