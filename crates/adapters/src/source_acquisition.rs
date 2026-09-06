//! Bounded HTTPS adapter for the explicitly reviewed local M60 observation profile.
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;
use ustc_campus_agent_core::source_workspace::ReviewedSource;

static FETCH_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
struct FetchSlot;
impl Drop for FetchSlot {
    fn drop(&mut self) {
        FETCH_ACTIVE.store(false, std::sync::atomic::Ordering::Release);
    }
}

pub struct AcquiredSource {
    pub raw: Vec<u8>,
    pub text: String,
    pub last_modified: Option<String>,
}
/// Fetches one exact reviewed public USTC URL. No cookies, credentials, proxy,
/// redirects or inherited DNS resolution at connection time are permitted.
pub async fn acquire(source: &ReviewedSource) -> Result<AcquiredSource, String> {
    source.validate()?;
    FETCH_ACTIVE
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire,
        )
        .map_err(|_| "source_fetch_busy")?;
    let _slot = FetchSlot;
    tokio::time::timeout(Duration::from_secs(20), acquire_inner(source))
        .await
        .map_err(|_| "source_fetch_timeout".to_owned())?
}
async fn acquire_inner(source: &ReviewedSource) -> Result<AcquiredSource, String> {
    let url = reqwest::Url::parse(&source.url).map_err(|_| "invalid_source_url")?;
    let host = url.host_str().ok_or("invalid_source_url")?;
    let answers = tokio::net::lookup_host((host, 443))
        .await
        .map_err(|_| "source_dns_unavailable")?;
    let mut addresses: Vec<_> = answers
        .filter_map(|a| match a {
            SocketAddr::V4(v4) => Some(*v4.ip()),
            _ => None,
        })
        .collect();
    addresses.sort();
    addresses.dedup();
    if addresses.is_empty() || addresses.len() > 16 || addresses.iter().any(|a| !public_ipv4(*a)) {
        return Err("source_dns_not_public".into());
    }
    let peer = SocketAddr::from((addresses[0], 443));
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .http1_only()
        .pool_max_idle_per_host(0)
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .resolve(host, peer)
        .build()
        .map_err(|_| "source_transport_unavailable")?;
    let mut response = client
        .get(url)
        .header("Accept", "text/html, text/plain;q=0.9")
        .header("Accept-Encoding", "identity")
        .header("Connection", "close")
        .send()
        .await
        .map_err(|_| "source_fetch_failed")?;
    if response.status().as_u16() != 200 {
        return Err(format!("source_http_status_{}", response.status().as_u16()));
    }
    if response.remote_addr().is_none_or(|a| a.ip() != peer.ip()) {
        return Err("source_peer_mismatch".into());
    }
    let media = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .ok_or("source_media_type_missing")?
        .to_lowercase();
    let html = media
        .split(';')
        .next()
        .is_some_and(|s| s.trim() == "text/html");
    if !html
        && !media
            .split(';')
            .next()
            .is_some_and(|s| s.trim() == "text/plain")
    {
        return Err("source_media_type_unsupported".into());
    }
    if media.contains("charset=")
        && !media.contains("charset=utf-8")
        && !media.contains("charset=\"utf-8\"")
    {
        return Err("source_charset_unsupported".into());
    }
    if response
        .headers()
        .get(reqwest::header::CONTENT_ENCODING)
        .is_some_and(|v| v != "identity")
    {
        return Err("source_encoding_unsupported".into());
    }
    if response.content_length().is_some_and(|n| n > 1048576) {
        return Err("source_body_too_large".into());
    }
    let last_modified = response
        .headers()
        .get(reqwest::header::LAST_MODIFIED)
        .and_then(|v| v.to_str().ok())
        .filter(|s| s.len() <= 256)
        .map(str::to_owned);
    let mut raw = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "source_body_failed")? {
        if raw.len() + chunk.len() > 1048576 {
            return Err("source_body_too_large".into());
        }
        raw.extend_from_slice(&chunk);
    }
    let decoded = std::str::from_utf8(&raw).map_err(|_| "source_charset_unsupported")?;
    let text = if html {
        html_text(decoded)
    } else {
        decoded.to_owned()
    };
    if text.len() > 131072 || text.trim().is_empty() {
        return Err("source_text_out_of_bounds".into());
    }
    Ok(AcquiredSource {
        raw,
        text,
        last_modified,
    })
}
// Same frozen IPv4 deny table as source-retrieval/v0. This adapter check grants
// no authority; it only prevents unsafe connections under the local profile.
fn public_ipv4(ip: Ipv4Addr) -> bool {
    let n = u32::from(ip);
    ![
        (0x00000000, 8),
        (0x0a000000, 8),
        (0x64400000, 10),
        (0x7f000000, 8),
        (0xa9fe0000, 16),
        (0xac100000, 12),
        (0xc0000000, 24),
        (0xc0000200, 24),
        (0xc0586300, 24),
        (0xc0a80000, 16),
        (0xc6120000, 15),
        (0xc6336400, 24),
        (0xcb007100, 24),
        (0xe0000000, 4),
        (0xf0000000, 4),
    ]
    .iter()
    .any(|(base, bits)| n & (u32::MAX << (32 - bits)) == *base)
}
/// Deterministic plain-text projection only, not a semantic campus fact parser.
fn html_text(input: &str) -> String {
    let mut out = String::new();
    let mut tag = String::new();
    let mut in_tag = false;
    let mut suppressed: Option<String> = None;
    for ch in input.chars() {
        if ch == '<' {
            in_tag = true;
            tag.clear();
            continue;
        }
        if in_tag {
            if ch == '>' {
                in_tag = false;
                let lower = tag.trim().to_ascii_lowercase();
                let name = lower.split_whitespace().next().unwrap_or("");
                if let Some(current) = &suppressed {
                    if name == format!("/{current}") {
                        suppressed = None;
                    }
                } else if matches!(name, "script" | "style" | "noscript") {
                    suppressed = Some(name.into());
                } else if matches!(
                    name,
                    "p" | "/p"
                        | "div"
                        | "/div"
                        | "br"
                        | "br/"
                        | "li"
                        | "/li"
                        | "tr"
                        | "/tr"
                        | "h1"
                        | "h2"
                        | "h3"
                ) {
                    out.push('\n');
                }
            } else if tag.len() < 2048 {
                tag.push(ch);
            }
        } else if suppressed.is_none() {
            out.push(ch);
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_documentation_and_rebinding_addresses_are_denied() {
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "169.254.169.254",
            "203.0.113.1",
            "192.168.1.1",
            "100.64.0.1",
            "224.0.0.1",
        ] {
            assert!(!public_ipv4(ip.parse().expect("ip")));
        }
        assert!(public_ipv4("202.38.64.1".parse().expect("ip")));
    }
    #[test]
    fn extracted_text_excludes_script_and_style_and_preserves_chinese() {
        assert_eq!(
            html_text(
                "<style>bad</style><h1>通知</h1><script>alert(1)</script><p>报名 &amp; 截止</p>"
            ),
            "通知\n报名 & 截止"
        );
    }
}
