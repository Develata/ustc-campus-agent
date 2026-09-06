use super::{EndpointPolicy, Limits, McpError};
use reqwest::{Client, Url};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub(super) fn validate_url(endpoint: &str, policy: EndpointPolicy) -> Result<Url, McpError> {
    let has_userinfo = endpoint.split_once("://").is_some_and(|(_, rest)| {
        rest.split(['/', '?', '#'])
            .next()
            .is_some_and(|authority| authority.contains('@'))
    });
    if endpoint.trim() != endpoint
        || endpoint.chars().any(char::is_control)
        || endpoint.contains('\\')
        || has_userinfo
    {
        return Err(McpError::EndpointDenied);
    }
    let url = Url::parse(endpoint).map_err(|_| McpError::EndpointDenied)?;
    if endpoint.len() > 2048
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.host_str().is_none()
    {
        return Err(McpError::EndpointDenied);
    }
    let ip = numeric_host(&url);
    match policy {
        EndpointPolicy::PublicHttps => {
            if url.scheme() != "https" || ip.is_some_and(|ip| !public_ip(ip)) {
                return Err(McpError::EndpointDenied);
            }
        }
        EndpointPolicy::LoopbackDevelopment => {
            if url.scheme() != "http" || !ip.is_some_and(|ip| ip.is_loopback()) {
                return Err(McpError::EndpointDenied);
            }
        }
    }
    Ok(url)
}

fn numeric_host(url: &Url) -> Option<IpAddr> {
    url.host_str()?
        .trim_start_matches('[')
        .trim_end_matches(']')
        .parse()
        .ok()
}

/// A new client per request repeats DNS admission and prevents a pooled connection
/// bypassing the current endpoint policy. Only validated addresses reach the connector.
pub(super) async fn pinned_client(
    url: &Url,
    policy: EndpointPolicy,
    limits: &Limits,
) -> Result<Client, McpError> {
    let host = url.host_str().ok_or(McpError::EndpointDenied)?;
    let port = url
        .port_or_known_default()
        .ok_or(McpError::EndpointDenied)?;
    let addresses = if let Some(ip) = numeric_host(url) {
        vec![SocketAddr::new(ip, port)]
    } else {
        tokio::net::lookup_host((host, port))
            .await
            .map_err(|_| McpError::EndpointDenied)?
            .take(33)
            .collect::<Vec<_>>()
    };
    if addresses.is_empty()
        || addresses.len() > 32
        || addresses.iter().any(|address| match policy {
            EndpointPolicy::PublicHttps => !public_ip(address.ip()),
            EndpointPolicy::LoopbackDevelopment => !address.ip().is_loopback(),
        })
    {
        return Err(McpError::EndpointDenied);
    }
    Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .timeout(limits.request_timeout)
        .connect_timeout(limits.request_timeout)
        .resolve_to_addrs(host, &addresses)
        .build()
        .map_err(|_| McpError::Transport)
}

fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => public_v4(ip),
        IpAddr::V6(ip) => public_v6(ip),
    }
}

fn public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    !(a == 0
        || a == 10
        || a == 127
        || a >= 224
        || (a == 100 && (64..=127).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && (b == 168 || (b == 0 && (c == 0 || c == 2)) || (b == 88 && c == 99)))
        || (a == 198 && (b == 18 || b == 19 || (b == 51 && c == 100)))
        || (a == 203 && b == 0 && c == 113))
}

fn public_v6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    // Admit global unicast only; reject translation/tunnelling and documentation
    // blocks conservatively, including every IPv4-mapped representation.
    s[0] & 0xe000 == 0x2000
        && !(s[0] == 0x2001 && (s[1] < 0x200 || s[1] == 0xdb8))
        && s[0] != 0x2002
        && s[0] != 0x3fff
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoint_policy_rejects_private_reserved_and_obfuscated_hosts() {
        for endpoint in [
            "http://example.com/mcp",
            "https://user:secret@example.com",
            "https://@example.com",
            "https://example.com/#x",
            "https://127.0.0.1",
            "https://2130706433",
            "https://0x7f000001",
            "https://[::ffff:127.0.0.1]",
            "https://10.1.1.1",
            "https://100.64.0.1",
            "https://192.0.2.1",
            "https://198.18.0.1",
            "https://[2001:db8::1]",
            "https://[2002:7f00:1::]",
            "https://[fc00::1]",
        ] {
            assert_eq!(
                validate_url(endpoint, EndpointPolicy::PublicHttps).err(),
                Some(McpError::EndpointDenied)
            );
        }
        assert!(validate_url("https://example.com/mcp", EndpointPolicy::PublicHttps).is_ok());
        assert!(
            validate_url(
                "http://127.0.0.1:8788/mcp",
                EndpointPolicy::LoopbackDevelopment
            )
            .is_ok()
        );
        assert!(validate_url("http://localhost/mcp", EndpointPolicy::LoopbackDevelopment).is_err());
        assert!(
            validate_url(
                "http://192.168.1.1/mcp",
                EndpointPolicy::LoopbackDevelopment
            )
            .is_err()
        );
    }
}
