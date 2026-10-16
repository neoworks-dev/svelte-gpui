//! `fetch`: HTTP requests, each run on its own thread.
//!
//! The request is `{ url, method, headers: [[name, value], ...], redirect }` with the
//! body as the request's bytes. The answer is `{ url, redirected, status, statusText,
//! headers: [[name, value], ...] }` with the response body as bytes. The body is read to
//! the end before anything is sent back: there are no streaming bodies. An error status
//! is an answer like any other; only a request that got no response at all fails.

use std::io::Read;
use std::time::Duration;

use serde_json::{Value, json};

use super::Sink;

/// How many redirects `redirect: "follow"` goes through.
const MAX_REDIRECTS: u32 = 20;

/// How long connecting may take. Reading a response has no limit: bodies can be large.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// Starts a request; `fetch` settles `call` with the response or a network error.
pub fn fetch(sink: &Sink, call: u64, args: Value, body: Option<Vec<u8>>) {
    let reply = sink.clone();
    let spawned = std::thread::Builder::new().name(format!("fetch-{call}")).spawn(move || match perform(&args, body) {
        Ok((head, bytes)) => reply.resolve_bytes(call, head, bytes),
        Err(message) => reply.reject(call, message),
    });
    if spawned.is_err() {
        sink.reject(call, "could not start a thread for the request");
    }
}

/// Makes the request and reads the whole response.
fn perform(args: &Value, body: Option<Vec<u8>>) -> Result<(Value, Vec<u8>), String> {
    let url = args["url"].as_str().ok_or("fetch needs a url")?;
    let method = args["method"].as_str().unwrap_or("GET");
    let redirects = if args["redirect"].as_str().unwrap_or("follow") == "follow" { MAX_REDIRECTS } else { 0 };
    let agent = ureq::AgentBuilder::new().redirects(redirects).timeout_connect(CONNECT_TIMEOUT).build();
    let mut request = agent.request(method, url);
    for pair in args["headers"].as_array().into_iter().flatten() {
        if let (Some(name), Some(value)) = (pair[0].as_str(), pair[1].as_str()) {
            request = request.set(name, value);
        }
    }
    let result = match body {
        Some(bytes) if !bytes.is_empty() => request.send_bytes(&bytes),
        _ => request.call(),
    };
    let response = match result {
        Ok(response) => response,
        Err(ureq::Error::Status(_, response)) => response,
        Err(ureq::Error::Transport(transport)) => return Err(transport.to_string()),
    };
    describe(url, response)
}

/// Reads a response's body and describes the rest of it as JSON.
fn describe(requested: &str, response: ureq::Response) -> Result<(Value, Vec<u8>), String> {
    let url = response.get_url().to_string();
    let status = response.status();
    let status_text = response.status_text().to_string();
    let headers = header_pairs(&response);
    let mut bytes = Vec::new();
    response.into_reader().read_to_end(&mut bytes).map_err(|error| error.to_string())?;
    let head = json!({
        "url": url,
        "redirected": url != requested,
        "status": status,
        "statusText": status_text,
        "headers": headers,
    });
    Ok((head, bytes))
}

/// Every header of the response, a repeated one once per value.
fn header_pairs(response: &ureq::Response) -> Vec<[String; 2]> {
    let mut pairs = Vec::new();
    for name in response.headers_names() {
        for value in response.all(&name) {
            pairs.push([name.clone(), value.to_string()]);
        }
    }
    pairs
}
