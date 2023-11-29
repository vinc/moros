use crate::sys;
use crate::api::clock;
use crate::api::clock::DATE_TIME_ZONE;
use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::api::time;

use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use smoltcp::iface::SocketSet;
use smoltcp::socket::tcp;
use smoltcp::time::Instant;
use smoltcp::phy::Device;
use smoltcp::wire::IpAddress;

const MAX_CONNECTIONS: usize = 32;
const POLL_DELAY_DIV: usize = 128;

#[derive(Clone)]
struct Request {
    addr: IpAddress,
    verb: String,
    path: String,
    body: Vec<u8>,
    headers: BTreeMap<String, String>,
}

impl Request {
    pub fn new(addr: IpAddress) -> Self {
        Self {
            addr,
            verb: String::new(),
            path: String::new(),
            body: Vec::new(),
            headers: BTreeMap::new(),
        }
    }

    pub fn from(addr: IpAddress, buf: &[u8]) -> Option<Self> {
        let msg = String::from_utf8_lossy(buf);
        if !msg.is_empty() {
            let mut req = Request::new(addr);
            let mut is_header = true;
            for (i, line) in msg.lines().enumerate() {
                if i == 0 { // Request line
                    let fields: Vec<_> = line.split(' ').collect();
                    if fields.len() >= 2 {
                        req.verb = fields[0].to_string();
                        req.path = fields[1].to_string();
                    }
                } else if is_header { // Message header
                    if let Some((key, val)) = line.split_once(':') {
                        req.headers.insert(key.trim().to_string(), val.trim().to_string());
                    } else if line.is_empty() {
                        is_header = false;
                    }
                } else if !is_header { // Message body
                    req.body.extend_from_slice(format!("{}\n", line).as_bytes());
                }
            }
            Some(req)
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct Response {
    req: Request,
    buf: Vec<u8>,
    mime: String,
    time: String,
    code: usize,
    size: usize,
    body: Vec<u8>,
    headers: BTreeMap<String, String>,
}

impl Response {
    pub fn new(req: Request) -> Self {
        let mut headers = BTreeMap::new();
        headers.insert("Date".to_string(), time::now_utc().format("%a, %d %b %Y %H:%M:%S GMT"));
        headers.insert("Server".to_string(), format!("MOROS/{}", env!("CARGO_PKG_VERSION")));
        Self {
            req,
            buf: Vec::new(),
            mime: String::new(),
            time: time::now().format(DATE_TIME_ZONE),
            code: 0,
            size: 0,
            body: Vec::new(),
            headers,
        }
    }

    pub fn end(&mut self) {
        self.size = self.body.len();
        self.headers.insert("Content-Length".to_string(), self.size.to_string());
        self.headers.insert("Connection".to_string(), if self.is_persistent() {
            "keep-alive".to_string()
        } else {
            "close".to_string()
        });
        self.headers.insert("Content-Type".to_string(), if self.mime.starts_with("text/") {
            format!("{}; charset=utf-8", self.mime)
        } else {
            format!("{}", self.mime)
        });
        self.write();
    }

    fn write(&mut self) {
        self.buf.clear();
        self.buf.extend_from_slice(format!("{}\r\n", self.status()).as_bytes());
        for (key, val) in &self.headers {
            self.buf.extend_from_slice(format!("{}: {}\r\n", key, val).as_bytes());
        }
        self.buf.extend_from_slice(b"\r\n");
        self.buf.extend_from_slice(&self.body);
    }

    fn status(&self) -> String {
        let msg = match self.code {
            200 => "OK",
            301 => "Moved Permanently",
            400 => "Bad Request",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _   => "Unknown Error",
        };
        format!("HTTP/1.1 {} {}", self.code, msg)
    }

    fn is_persistent(&self) -> bool {
        if let Some(value) = self.req.headers.get("Connection") {
            if value == "close" {
                return false;
            }
        }
        true
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let csi_blue = Style::color("LightBlue");
        let csi_cyan = Style::color("LightCyan");
        let csi_pink = Style::color("Pink");
        let csi_reset = Style::reset();
        write!(
            f, "{}{} - -{} [{}] {}\"{} {}\"{} {} {}",
            csi_cyan, self.req.addr,
            csi_pink, self.time,
            csi_blue, self.req.verb, self.req.path,
            csi_reset, self.code, self.size
        )
    }
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    let mut read_only = false;
    let mut port = 80;
    let mut dir = sys::process::dir();
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                usage();
                return Ok(());
            }
            "-r" | "--read-only" => {
                read_only = true;
            }
            "-p" | "--port" => {
                if i + 1 < n {
                    port = args[i + 1].parse().unwrap_or(port);
                    i += 1;
                } else {
                    error!("Missing port number");
                    return Err(ExitCode::UsageError);
                }
            }
            "-d" | "--dir" => {
                if i + 1 < n {
                    dir = args[i + 1].to_string();
                    i += 1;
                } else {
                    error!("Missing directory");
                    return Err(ExitCode::UsageError);
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Some((ref mut iface, ref mut device)) = *sys::net::NET.lock() {
        let mut sockets = SocketSet::new(vec![]);

        let mtu = device.capabilities().max_transmission_unit;
        let buf_len = mtu - 14 - 20 - 20; // ETH+TCP+IP headers
        let mut connections = Vec::new();
        for _ in 0..MAX_CONNECTIONS {
            let tcp_rx_buffer = tcp::SocketBuffer::new(vec![0; buf_len]);
            let tcp_tx_buffer = tcp::SocketBuffer::new(vec![0; buf_len]);
            let tcp_socket = tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer);
            let tcp_handle = sockets.add(tcp_socket);

            let send_queue: VecDeque<Vec<u8>> = VecDeque::new();
            let keep_alive = true;
            connections.push((tcp_handle, send_queue, keep_alive));
        }

        println!("{}HTTP Server listening on 0.0.0.0:{}{}", csi_color, port, csi_reset);

        loop {
            if sys::console::end_of_text() || sys::console::end_of_transmission() {
                println!();
                return Ok(());
            }

            let time = Instant::from_micros((clock::realtime() * 1000000.0) as i64);
            iface.poll(time, device, &mut sockets);

            for (tcp_handle, send_queue, keep_alive) in &mut connections {
                let socket = sockets.get_mut::<tcp::Socket>(*tcp_handle);

                if !socket.is_open() {
                    socket.listen(port).unwrap();
                }
                let endpoint = match socket.remote_endpoint() {
                    Some(endpoint) => endpoint,
                    None => continue,
                };
                if socket.may_recv() {
                    let res = socket.recv(|buffer| {
                        if let Some(req) = Request::from(endpoint.addr, buffer) {
                            let mut res = Response::new(req.clone());
                            let sep = if req.path == "/" { "" } else { "/" };
                            let real_path = format!("{}{}{}", dir, sep, req.path.strip_suffix('/').unwrap_or(&req.path)).replace("//", "/");

                            match req.verb.as_str() {
                                "GET" => {
                                    if fs::is_dir(&real_path) && !req.path.ends_with('/') {
                                        res.code = 301;
                                        res.mime = "text/html".to_string();
                                        res.headers.insert("Location".to_string(), format!("{}/", req.path));
                                        res.body.extend_from_slice(b"<h1>Moved Permanently</h1>\r\n");
                                    } else {
                                        let mut not_found = true;
                                        for autocomplete in &["", "/index.html", "/index.htm", "/index.txt"] {
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
                                                res.body.extend_from_slice(format!("<h1>Index of {}</h1>\r\n", req.path).as_bytes());
                                                files.sort_by_key(|f| f.name());
                                                for file in files {
                                                    let path = format!("{}{}", req.path, file.name());
                                                    let link = format!("<li><a href=\"{}\">{}</a></li>\n", path, file.name());
                                                    res.body.extend_from_slice(link.as_bytes());
                                                }
                                            } else {
                                                res.code = 404;
                                                res.mime = "text/html".to_string();
                                                res.body.extend_from_slice(b"<h1>Not Found</h1>\r\n");
                                            }
                                        }
                                    }
                                },
                                "PUT" if !read_only => {
                                    if real_path.ends_with('/') { // Write directory
                                        let real_path = real_path.trim_end_matches('/');
                                        if fs::exists(real_path) {
                                            res.code = 403;
                                        } else if let Some(handle) = fs::create_dir(real_path) {
                                            syscall::close(handle);
                                            res.code = 200;
                                        } else {
                                            res.code = 500;
                                        }
                                    } else { // Write file
                                        if fs::write(&real_path, &req.body).is_ok() {
                                            res.code = 200;
                                        } else {
                                            res.code = 500;
                                        }
                                    }
                                    res.mime = "text/plain".to_string();
                                },
                                "DELETE" if !read_only => {
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
                            res.end();
                            println!("{}", res);
                            (buffer.len(), Some(res))
                        } else {
                            (0, None)
                        }
                    });
                    if let Ok(Some(res)) = res {
                        *keep_alive = res.is_persistent();
                        for chunk in res.buf.chunks(buf_len) {
                            send_queue.push_back(chunk.to_vec());
                        }
                    }
                    if socket.can_send() {
                        if let Some(chunk) = send_queue.pop_front() {
                            let sent = socket.send_slice(&chunk).expect("Could not send chunk");
                            debug_assert!(sent == chunk.len());
                        }
                    }
                    if send_queue.is_empty() && !*keep_alive {
                        socket.close();
                    }
                } else if socket.may_send() {
                    socket.close();
                    send_queue.clear();
                }
            }
            if let Some(wait_duration) = iface.poll_delay(time, &sockets) {
                let t = wait_duration.total_micros() / POLL_DELAY_DIV as u64;
                if t > 0 {
                    syscall::sleep((t as f64) / 1000000.0);
                }
            }
        }
    } else {
        error!("Could not find network interface");
        Err(ExitCode::Failure)
    }
}

fn content_type(path: &str) -> String {
    let ext = path.rsplit_once('.').unwrap_or(("", "")).1;
    match ext {
        "css"          => "text/css",
        "csv"          => "text/csv",
        "gif"          => "text/gif",
        "htm" | "html" => "text/html",
        "jpg" | "jpeg" => "image/jpeg",
        "js"           => "text/javascript",
        "json"         => "application/json",
        "lsp" | "lisp" => "text/plain",
        "png"          => "image/png",
        "sh"           => "application/x-sh",
        "txt"          => "text/plain",
        _              => "application/octet-stream",
    }.to_string()
}

fn usage() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} httpd {}<options>{1}", csi_title, csi_reset, csi_option);
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!("  {0}-d{1}, {0}--dir <path>{1}       Set directory to {0}<path>{1}", csi_option, csi_reset);
    println!("  {0}-p{1}, {0}--port <number>{1}    Listen to port {0}<number>{1}", csi_option, csi_reset);
    println!("  {0}-r{1}, {0}--read-only{1}        Set read-only mode", csi_option, csi_reset);
}
