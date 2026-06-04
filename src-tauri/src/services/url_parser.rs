/// Extracts the hostname from a URL string.
///
/// Handles:
/// - Full URLs: `https://www.example.com/path` → `www.example.com`
/// - Schemeless: `www.example.com/path` → `www.example.com`
/// - Bare domains: `example.com` → `example.com`
/// - With port: `example.com:8080` → `example.com`
///
/// Returns `None` if the input cannot be parsed into a valid hostname.
#[must_use]
pub fn extract_domain(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Strip scheme if present
    let after_scheme = trimmed.find("://").map_or(trimmed, |pos| &trimmed[pos + 3..]);

    // Take everything up to '/' or '?' or '#'
    let host_port = after_scheme
        .split(&['/', '?', '#'])
        .next()
        .unwrap_or(after_scheme);

    // Strip port if present
    let host = host_port.split(':').next().unwrap_or(host_port);

    let host = host.trim();
    if host.is_empty() {
        return None;
    }

    Some(host.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_from_https_url() {
        assert_eq!(
            extract_domain("https://www.example.com/path"),
            Some("www.example.com".to_string())
        );
    }

    #[test]
    fn extracts_from_http_url() {
        assert_eq!(
            extract_domain("http://github.com/user/repo"),
            Some("github.com".to_string())
        );
    }

    #[test]
    fn extracts_bare_domain() {
        assert_eq!(
            extract_domain("example.com"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn extracts_domain_with_port() {
        assert_eq!(
            extract_domain("https://example.com:8080/path"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn extracts_from_schemeless_url() {
        assert_eq!(
            extract_domain("www.google.com/search?q=test"),
            Some("www.google.com".to_string())
        );
    }

    #[test]
    fn returns_none_for_empty() {
        assert_eq!(extract_domain(""), None);
    }

    #[test]
    fn returns_none_for_whitespace_only() {
        assert_eq!(extract_domain("   "), None);
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(
            extract_domain("  https://example.com  "),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn extracts_subdomain() {
        assert_eq!(
            extract_domain("https://cdn.anthropic.com/assets"),
            Some("cdn.anthropic.com".to_string())
        );
    }
}
