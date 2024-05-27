use crate::api::clock;
use crate::api::clock::DATE_TIME_ZONE;
use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::api::time;
use crate::sys;
use crate::sys::console;

use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use smoltcp::iface::SocketSet;
use smoltcp::phy::Device;
use smoltcp::socket::tcp;
use smoltcp::time::Instant;
use smoltcp::wire::IpAddress;

const MAX_CONNECTIONS: usize = 32;
const POLL_DELAY_DIV: usize = 128;
const INDEX: [&str; 4] = ["", "/index.html", "/index.htm", "/index.txt"];

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
                if i == 0 {
                    // Request line
                    let fields: Vec<_> = line.split(' ').collect();
                    if fields.len() >= 2 {
                        req.verb = fields[0].to_string();
                        req.path = fields[1].to_string();
                    }
                } else if is_header {
                    // Message header
                    if let Some((key, val)) = line.split_once(':') {
                        let k = key.trim().to_string();
                        let v = val.trim().to_string();
                        req.headers.insert(k, v);
                    } else if line.is_empty() {
                        is_header = false;
                    }
                } else if !is_header {
                    // Message body
                    let s = format!("{}\n", line);
                    req.body.extend_from_slice(s.as_bytes());
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
    real_path: String,
}

impl Response {
    pub fn new(req: Request) -> Self {
        let mut headers = BTreeMap::new();
        headers.insert(
            "Date".to_string(),
            time::now_utc().format("%a, %d %b %Y %H:%M:%S GMT"),
        );
        headers.insert(
            "Server".to_string(),
            format!("MOROS/{}", env!("CARGO_PKG_VERSION")),
        );
        Self {
            req,
            buf: Vec::new(),
            mime: String::new(),
            time: time::now().format(DATE_TIME_ZONE),
            code: 0,
            size: 0,
            body: Vec::new(),
            headers,
            real_path: String::new(),
        }
    }

    pub fn end(&mut self) {
        self.size = self.body.len();
        self.headers.insert(
            "Content-Length".to_string(),
            self.size.to_string()
        );
        self.headers.insert(
            "Connection".to_string(),
            if self.is_persistent() {
                "keep-alive".to_string()
            } else {
                "close".to_string()
            }
        );
        self.headers.insert(
            "Content-Type".to_string(),
            if self.mime.starts_with("text/") {
                format!("{}; charset=utf-8", self.mime)
            } else {
                format!("{}", self.mime)
            }
        );
        self.write();
    }

    fn write(&mut self) {
        self.buf.clear();
        self.buf.extend_from_slice(
            format!("{}\r\n", self.status()).as_bytes()
        );
        for (key, val) in &self.headers {
            self.buf.extend_from_slice(
                format!("{}: {}\r\n", key, val).as_bytes()
            );
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
            _ => "Unknown Error",
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
        let csi_blue = Style::color("blue");
        let csi_cyan = Style::color("LightCyan");
        let csi_pink = Style::color("Pink");
        let csi_reset = Style::reset();
        write!(
            f,
            "{}{} - -{} [{}] {}\"{} {}\"{} {} {}",
            csi_cyan,
            self.req.addr,
            csi_pink,
            self.time,
            csi_blue,
            self.req.verb,
            self.req.path,
            csi_reset,
            self.code,
            self.size
        )
    }
}

fn get(req: &Request, res: &mut Response) {
    if fs::is_dir(&res.real_path) && !req.path.ends_with('/') {
        res.code = 301;
        res.mime = "text/html".to_string();
        res.headers.insert(
            "Location".to_string(),
            format!("{}/", req.path),
        );
        res.body.extend_from_slice(b"<h1>Moved Permanently</h1>\r\n");
    } else {
        let mut not_found = true;
        for index in INDEX {
            let real_path = format!("{}{}", res.real_path, index);
            if fs::is_dir(&real_path) {
                continue;
            }
            if let Ok(buf) = fs::read_to_bytes(&real_path) {
                res.code = 200;
                res.mime = content_type(&real_path);
                let tmp;
                res.body.extend_from_slice(
                    if res.mime.starts_with("text/") {
                        tmp = String::from_utf8_lossy(&buf).to_string().
                            replace("\n", "\r\n");
                        tmp.as_bytes()
                    } else {
                        &buf
                    },
                );
                not_found = false;
                break;
            }
        }
        if not_found {
            if let Ok(mut files) = fs::read_dir(&res.real_path) {
                res.code = 200;
                res.mime = "text/html".to_string();
                res.body.extend_from_slice(
                    format!("<h1>Index of {}</h1>\r\n", req.path).as_bytes()
                );
                files.sort_by_key(|f| f.name());
                for file in files {
                    let path = format!("{}{}", req.path, file.name());
                    let link = format!(
                        "<li><a href=\"{}\">{}</a></li>\n",
                        path,
                        file.name()
                    );
                    res.body.extend_from_slice(link.as_bytes());
                }
            } else {
                res.code = 404;
                res.mime = "text/html".to_string();
                res.body.extend_from_slice(b"<h1>Not Found</h1>\r\n");
            }
        }
    }
}

fn put(req: &Request, res: &mut Response) {
    if res.real_path.ends_with('/') {
        // Write directory
        let real_path = res.real_path.trim_end_matches('/');
        if fs::exists(real_path) {
            res.code = 403;
        } else if let Some(handle) = fs::create_dir(real_path) {
            syscall::close(handle);
            res.code = 200;
        } else {
            res.code = 500;
        }
    } else {
        // Write file
        if fs::write(&res.real_path, &req.body).is_ok() {
            res.code = 200;
        } else {
            res.code = 500;
        }
    }
    res.mime = "text/plain".to_string();
}

fn delete(_req: &Request, res: &mut Response) {
    if fs::exists(&res.real_path) {
        if fs::delete(&res.real_path).is_ok() {
            res.code = 200;
        } else {
            res.code = 500;
        }
    } else {
        res.code = 404;
    }
    res.mime = "text/plain".to_string();
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let csi_color = Style::color("yellow");
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
                    i += 1;
                    port = args[i].parse().unwrap_or(port);
                } else {
                    error!("Missing port number");
                    return Err(ExitCode::UsageError);
                }
            }
            "-d" | "--dir" => {
                if i + 1 < n {
                    i += 1;
                    dir = args[i].to_string();
                } else {
                    error!("Missing directory");
                    return Err(ExitCode::UsageError);
                }
            }
            _ => {}
        }
        i += 1;
    }

    // NOTE: This specific format is needed by `join_path`
    let dir = format!("/{}", fs::realpath(&dir).trim_matches('/'));

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

        println!(
            "{}HTTP Server listening on 0.0.0.0:{}{}",
            csi_color, port, csi_reset
        );

        loop {
            if console::end_of_text() || console::end_of_transmission() {
                println!();
                return Ok(());
            }

            let ms = (clock::realtime() * 1000000.0) as i64;
            let time = Instant::from_micros(ms);
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
                    // The amount of octets queued in the receive buffer may be
                    // larger than the contiguous slice returned by `recv` so
                    // we need to loop over chunks of it until it is empty.
                    let recv_queue = socket.recv_queue();
                    let mut receiving = true;
                    let mut buf = vec![];
                    while receiving {
                        let res = socket.recv(|chunk| {
                            buf.extend_from_slice(chunk);
                            if buf.len() < recv_queue {
                                return (chunk.len(), None);
                            }
                            receiving = false;

                            let addr = endpoint.addr;
                            if let Some(req) = Request::from(addr, &buf) {
                                let mut res = Response::new(req.clone());
                                res.real_path = join_path(&dir, &req.path);

                                match req.verb.as_str() {
                                    "GET" => {
                                        get(&req, &mut res)
                                    }
                                    "PUT" if !read_only => {
                                        put(&req, &mut res)
                                    }
                                    "DELETE" if !read_only => {
                                        delete(&req, &mut res)
                                    }
                                    _ => {
                                        let s = b"<h1>Bad Request</h1>\r\n";
                                        res.body.extend_from_slice(s);
                                        res.code = 400;
                                        res.mime = "text/html".to_string();
                                    }
                                }
                                res.end();
                                println!("{}", res);
                                (chunk.len(), Some(res))
                            } else {
                                (0, None)
                            }
                        });
                        if receiving {
                            continue;
                        }
                        if let Ok(Some(res)) = res {
                            *keep_alive = res.is_persistent();
                            for chunk in res.buf.chunks(buf_len) {
                                send_queue.push_back(chunk.to_vec());
                            }
                        }
                    }
                    if socket.can_send() {
                        if let Some(chunk) = send_queue.pop_front() {
                            let sent = socket.send_slice(&chunk).
                                expect("Could not send chunk");
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
            if let Some(delay) = iface.poll_delay(time, &sockets) {
                let d = delay.total_micros() / POLL_DELAY_DIV as u64;
                if d > 0 {
                    syscall::sleep((d as f64) / 1000000.0);
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
        "txt" | "md"   => "text/plain",
        _              => "application/octet-stream",
    }.to_string()
}

// Join the requested file path to the root dir of the server
fn join_path(dir: &str, path: &str) -> String {
    debug_assert!(dir.starts_with('/'));
    debug_assert!(path.starts_with('/'));
    let path = path.trim_matches('/');
    let sep = if dir == "/" || path == "" { "" } else { "/" };
    format!("{}{}{}", dir, sep, path)
}

fn usage() {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} httpd {}<options>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-d{1}, {0}--dir <path>{1}       Set directory to {0}<path>{1}",
        csi_option, csi_reset
    );
    println!(
        "  {0}-p{1}, {0}--port <number>{1}    Listen to port {0}<number>{1}",
        csi_option, csi_reset
    );
    println!(
        "  {0}-r{1}, {0}--read-only{1}        Set read-only mode",
        csi_option, csi_reset
    );
}

#[test_case]
fn test_join_path() {
    assert_eq!(join_path("/foo", "/bar/"), "/foo/bar");
    assert_eq!(join_path("/foo", "/bar"), "/foo/bar");
    assert_eq!(join_path("/foo", "/"), "/foo");
    assert_eq!(join_path("/", "/bar/"), "/bar");
    assert_eq!(join_path("/", "/bar"), "/bar");
    assert_eq!(join_path("/", "/"), "/");
}
