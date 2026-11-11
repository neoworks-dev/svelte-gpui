//! The HTTP half of the server: the built app's files, and the checks that let a connection
//! become the app's WebSocket.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use super::{Shared, same_token, session};

/// The most a request's head may be, which is far more than a browser sends.
const MAX_HEAD: usize = 16 * 1024;

/// How long a request's head may take to arrive.
const HEAD_TIMEOUT: Duration = Duration::from_secs(5);

/// A request's first line and its headers, with the header names in lower case.
#[derive(Debug, PartialEq)]
pub(super) struct Head {
    pub method: String,
    pub target: String,
    pub headers: Vec<(String, String)>,
}

impl Head {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
    }

    pub fn path(&self) -> &str {
        self.target.split(['?', '#']).next().unwrap_or("")
    }

    /// The value of a query parameter, for the short hex values this server uses.
    pub fn query(&self, name: &str) -> Option<&str> {
        let query = self.target.split_once('?')?.1.split('#').next()?;
        query.split('&').find_map(|pair| pair.strip_prefix(name)?.strip_prefix('='))
    }

    pub fn is_upgrade(&self) -> bool {
        self.header("upgrade").is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
    }
}

pub(super) fn parse_head(text: &str) -> Option<Head> {
    let mut lines = text.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let (method, target) = (first.next()?.to_string(), first.next()?.to_string());
    if !first.next()?.starts_with("HTTP/1.") || first.next().is_some() {
        return None;
    }
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':')?;
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
    }
    Some(Head { method, target, headers })
}

/// Reads the head without taking it from the stream, so a WebSocket handshake can still read it
/// itself. Returns it with its length.
fn peek_head(stream: &TcpStream) -> std::io::Result<(Head, usize)> {
    use std::io::{Error, ErrorKind};
    stream.set_read_timeout(Some(HEAD_TIMEOUT))?;
    let deadline = Instant::now() + HEAD_TIMEOUT;
    let mut buffer = vec![0u8; MAX_HEAD];
    loop {
        let length = stream.peek(&mut buffer)?;
        if length == 0 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "the connection closed before a request came"));
        }
        if let Some(end) = buffer[..length].windows(4).position(|window| window == b"\r\n\r\n") {
            let text = std::str::from_utf8(&buffer[..end + 2])
                .map_err(|_| Error::new(ErrorKind::InvalidData, "the request is not text"))?;
            let head = parse_head(text).ok_or_else(|| Error::new(ErrorKind::InvalidData, "not an HTTP request"))?;
            return Ok((head, end + 4));
        }
        if length == MAX_HEAD || Instant::now() >= deadline {
            return Err(Error::new(ErrorKind::InvalidData, "the request head is too large or too slow"));
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Whether `host` is this server's own address: the name a page was opened by, with its port.
pub(super) fn own_host(host: &str, port: u16) -> bool {
    host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
}

/// Whether `origin` is this server's own origin.
pub(super) fn own_origin(origin: &str, port: u16) -> bool {
    origin == format!("http://127.0.0.1:{port}") || origin == format!("http://localhost:{port}")
}

/// Answers one connection: a file, a refusal, or the start of the app's session.
pub(super) fn handle(mut stream: TcpStream, shared: &Shared) {
    let _ = stream.set_nodelay(true);
    let Ok((head, length)) = peek_head(&stream) else {
        return;
    };
    if !head.header("host").is_some_and(|host| own_host(host, shared.port)) {
        respond(&mut stream, 403, "Forbidden", "text/plain", b"wrong host", false);
        return;
    }
    if head.is_upgrade() {
        // The handshake reads the head again.
        if head.path() != "/io" {
            consume(&mut stream, length);
            respond(&mut stream, 404, "Not Found", "text/plain", b"not found", false);
            return;
        }
        let origin_ok = head.header("origin").is_some_and(|origin| own_origin(origin, shared.port));
        let token_ok = head.query("token").is_some_and(|given| same_token(given, &shared.token));
        if !origin_ok || !token_ok {
            consume(&mut stream, length);
            let why: &[u8] = if origin_ok { b"wrong token" } else { b"wrong origin" };
            respond(&mut stream, 403, "Forbidden", "text/plain", why, false);
            return;
        }
        session::run(stream, shared);
        return;
    }
    consume(&mut stream, length);
    if head.method != "GET" && head.method != "HEAD" {
        respond(&mut stream, 405, "Method Not Allowed", "text/plain", b"only GET", false);
        return;
    }
    let only_head = head.method == "HEAD";
    match find(&shared.dir, head.path()) {
        Some(file) => match fs::read(&file) {
            Ok(body) => respond(&mut stream, 200, "OK", content_type(&file), &body, only_head),
            Err(_) => respond(&mut stream, 404, "Not Found", "text/plain", b"not found", only_head),
        },
        None => respond(&mut stream, 404, "Not Found", "text/plain", b"not found", only_head),
    }
}

fn consume(stream: &mut TcpStream, length: usize) {
    let mut sink = vec![0u8; length];
    let _ = stream.read_exact(&mut sink);
}

fn respond(stream: &mut TcpStream, status: u16, reason: &str, content_type: &str, body: &[u8], only_head: bool) {
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nX-Frame-Options: DENY\r\n\
         Referrer-Policy: no-referrer\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    if !only_head {
        let _ = stream.write_all(body);
    }
    let _ = stream.flush();
}

/// The file a request path names under `dir`: `/` is `index.html`, and nothing outside `dir` is
/// reachable, whether by `..` or by a link.
pub(super) fn find(dir: &Path, path: &str) -> Option<PathBuf> {
    let decoded = percent_decode(path)?;
    let mut file = dir.to_path_buf();
    for component in Path::new(decoded.trim_start_matches('/')).components() {
        match component {
            Component::Normal(name) => file.push(name),
            Component::CurDir => {}
            _ => return None,
        }
    }
    let root = fs::canonicalize(dir).ok()?;
    let mut file = fs::canonicalize(&file).ok()?;
    if file.is_dir() {
        file = file.join("index.html");
        file = fs::canonicalize(&file).ok()?;
    }
    (file.starts_with(&root) && file.is_file()).then_some(file)
}

fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let digits = std::str::from_utf8(bytes.get(index + 1..index + 3)?).ok()?;
            out.push(u8::from_str_radix(digits, 16).ok()?);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    let text = String::from_utf8(out).ok()?;
    (!text.contains('\0') && !text.contains('\\')).then_some(text)
}

fn content_type(file: &Path) -> &'static str {
    match file.extension().and_then(|extension| extension.to_str()).unwrap_or("") {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_head_is_read_with_lower_case_header_names() {
        let head = parse_head("GET /io?token=abc HTTP/1.1\r\nHost: 127.0.0.1:80\r\nUpgrade: WebSocket\r\n\r\n").unwrap();
        assert_eq!(head.path(), "/io");
        assert_eq!(head.query("token"), Some("abc"));
        assert_eq!(head.query("other"), None);
        assert_eq!(head.header("host"), Some("127.0.0.1:80"));
        assert!(head.is_upgrade());
    }

    #[test]
    fn something_that_is_not_http_is_refused() {
        assert_eq!(parse_head("hello"), None);
        assert_eq!(parse_head("GET / SPDY/3\r\n\r\n"), None);
    }

    #[test]
    fn only_the_servers_own_address_is_its_host_and_origin() {
        assert!(own_host("127.0.0.1:4000", 4000));
        assert!(own_host("localhost:4000", 4000));
        assert!(!own_host("evil.example:4000", 4000));
        assert!(!own_host("127.0.0.1:4001", 4000));
        assert!(own_origin("http://localhost:4000", 4000));
        assert!(!own_origin("http://evil.example", 4000));
        assert!(!own_origin("https://127.0.0.1:4000", 4000));
    }

    #[cfg(unix)]
    #[test]
    fn files_are_found_under_the_directory_and_nowhere_else() {
        let root = std::env::temp_dir().join(format!("svelte-gpui-serve-{}", std::process::id()));
        let outside = root.join("outside.txt");
        let dir = root.join("app");
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(&outside, "secret").unwrap();
        fs::write(dir.join("index.html"), "<p>").unwrap();
        fs::write(dir.join("sub").join("a.js"), "1").unwrap();
        std::os::unix::fs::symlink(&outside, dir.join("link.txt")).unwrap();

        assert!(find(&dir, "/").unwrap().ends_with("index.html"));
        assert!(find(&dir, "/sub/a.js").unwrap().ends_with("a.js"));
        assert!(find(&dir, "/sub/").is_none());
        assert!(find(&dir, "/../outside.txt").is_none());
        assert!(find(&dir, "/%2e%2e/outside.txt").is_none());
        assert!(find(&dir, "/sub/%2e%2e/%2e%2e/outside.txt").is_none());
        assert!(find(&dir, "/link.txt").is_none());
        assert!(find(&dir, "/missing.js").is_none());
        let _ = fs::remove_dir_all(&root);
    }
}
