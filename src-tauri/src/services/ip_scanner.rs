use std::collections::HashMap;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use native_tls::TlsConnector;

/// Built-in candidate IPs for key GitHub domains (from github-host `builtin_ip_pool.json`).
/// Each domain has up to 8 candidate IPs for TCP+TLS verification.
const GITHUB_CANDIDATE_IPS: &[(&str, &[&str])] = &[
    ("github.com", &["140.82.112.3", "140.82.112.4", "140.82.113.3", "140.82.113.4", "140.82.114.3", "140.82.114.4", "140.82.115.3", "140.82.115.4"]),
    ("api.github.com", &["140.82.112.5", "140.82.113.5", "140.82.114.5", "140.82.115.5", "140.82.116.5", "20.26.156.215", "20.27.177.113", "20.29.145.100"]),
    ("github.githubassets.com", &["185.199.110.154", "185.199.109.154", "185.199.111.154", "185.199.108.154", "185.199.110.215", "185.199.111.215", "185.199.109.215", "185.199.108.215"]),
    ("assets-cdn.github.com", &["185.199.108.153", "185.199.109.153", "185.199.110.153", "185.199.111.153", "185.199.108.154", "185.199.109.154", "185.199.110.154", "185.199.111.154"]),
    ("codeload.github.com", &["140.82.112.9", "140.82.113.9", "140.82.114.9", "140.82.115.9", "140.82.116.9", "20.26.156.215", "20.27.177.113", "20.29.145.100"]),
    ("raw.githubusercontent.com", &["185.199.108.133", "185.199.109.133", "185.199.110.133", "185.199.111.133", "185.199.108.154", "185.199.109.154", "185.199.110.154", "185.199.111.154"]),
    ("gist.github.com", &["140.82.112.3", "140.82.113.3", "140.82.114.3", "140.82.115.3", "140.82.116.3", "20.26.156.215", "20.27.177.113", "20.29.145.100"]),
    ("alive.github.com", &["140.82.112.26", "140.82.113.26", "140.82.114.26", "140.82.112.25", "140.82.113.25", "140.82.114.25"]),
    ("collector.github.com", &["140.82.114.22", "140.82.114.21", "140.82.112.21", "140.82.113.22", "140.82.112.22", "140.82.113.21"]),
    ("central.github.com", &["140.82.112.21", "140.82.114.22", "140.82.112.22", "140.82.113.21", "140.82.114.21", "140.82.113.22"]),
    ("education.github.com", &["140.82.114.22", "140.82.113.22", "140.82.113.21", "140.82.112.21", "140.82.112.22", "140.82.114.21"]),
    ("github.community", &["140.82.113.17", "140.82.114.18", "140.82.112.17", "140.82.113.18", "140.82.112.18", "140.82.114.17"]),
    ("github.io", &["185.199.109.153", "185.199.110.153", "185.199.108.153", "185.199.111.153"]),
];

const TCP_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_PARALLEL: usize = 8;

/// Trait for IP scanning, allowing test mocks.
pub trait IpScannerTrait: Send + Sync {
    /// Scan the given domains and return a mapping of input domain → verified IP.
    fn scan_domains(&self, domains: &[String]) -> HashMap<String, String>;
}

/// Result of testing a single IP against a domain.
struct TestResult {
    ip: String,
    latency_ms: u64,
}

/// Scanner that verifies candidate IPs for domains via TCP connect test.
///
/// For each domain, it tries the built-in candidate IPs and returns
/// the fastest reachable one. Domains without any reachable candidate
/// are excluded from the result (they will fall back to proxy routing).
#[derive(Debug, Clone)]
pub struct IpScanner;

impl IpScanner {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Returns true if the domain matched via parent matching (not exact or suffix).
    /// Parent-matched domains need wildcard hosts entries (`+.domain`) to cover all subdomains.
    #[must_use]
    pub fn is_parent_match(domain: &str) -> bool {
        // Exact match exists → not parent match
        if GITHUB_CANDIDATE_IPS.iter().any(|(d, _)| *d == domain) {
            return false;
        }
        // Suffix match exists → not parent match
        if GITHUB_CANDIDATE_IPS.iter().any(|(d, _)| domain.ends_with(*d) && domain.len() > d.len()) {
            return false;
        }
        // Check if any candidate domain is a subdomain of this domain
        let suffix = format!(".{domain}");
        GITHUB_CANDIDATE_IPS.iter().any(|(d, _)| d.ends_with(&suffix))
    }
}

impl IpScannerTrait for IpScanner {
    /// Scan the given domains and return a mapping of input domain → verified IP.
    ///
    /// Only domains with at least one reachable candidate IP are included
    /// in the result. Domains without candidates in the built-in pool are
    /// skipped (they will use proxy routing as fallback).
    ///
    /// The returned keys are the **input** domains (as passed by the caller),
    /// not the matched candidate domains. This ensures callers can look up
    /// results using their own domain names.
    ///
    /// # Panics
    ///
    /// Panics if internal mutex is poisoned (should never happen in single-threaded use).
    fn scan_domains(&self, domains: &[String]) -> HashMap<String, String> {
        let results = Arc::new(Mutex::new(HashMap::new()));

        // Build list of (input_domain, candidate_ips) for domains with known candidates
        let work: Vec<(String, &[&str])> = domains
            .iter()
            .filter_map(|d| {
                Self::find_candidates(d).map(|(_, ips)| (d.clone(), ips))
            })
            .collect();

        if work.is_empty() {
            return HashMap::new();
        }

        // Process in batches of MAX_PARALLEL domains
        for chunk in work.chunks(MAX_PARALLEL) {
            let handles: Vec<_> = chunk
                .iter()
                .map(|(input_domain, ips)| {
                    let input_domain = input_domain.clone();
                    let ips: Vec<String> = ips.iter().map(std::string::ToString::to_string).collect();
                    let results = Arc::clone(&results);
                    std::thread::spawn(move || {
                        if let Some(best) = Self::test_domain(&input_domain, &ips) {
                            results.lock().unwrap().insert(input_domain, best);
                        }
                    })
                })
                .collect();

            for h in handles {
                let _ = h.join();
            }
        }

        Arc::try_unwrap(results).unwrap().into_inner().unwrap()
    }
}

impl IpScanner {
    /// Find candidate IPs for a domain from the built-in pool.
    ///
    /// Matching priority:
    /// 1. Exact match (e.g., "github.com" → "github.com")
    /// 2. Suffix match — input is a subdomain (e.g., "www.github.com" → "github.com")
    /// 3. Parent match — input is a parent domain (e.g., "githubassets.com" → "github.githubassets.com")
    fn find_candidates(domain: &str) -> Option<(&'static str, &'static [&'static str])> {
        // Exact match
        for (d, ips) in GITHUB_CANDIDATE_IPS {
            if *d == domain {
                return Some((*d, *ips));
            }
        }
        // Suffix match (e.g., "www.github.com" matches "github.com" entry)
        for (d, ips) in GITHUB_CANDIDATE_IPS {
            if domain.ends_with(d) && domain.len() > d.len() {
                return Some((*d, *ips));
            }
        }
        // Parent match (e.g., "githubassets.com" matches "github.githubassets.com" entry)
        // Input domain is a parent of a known candidate — use that candidate's IPs
        let suffix = format!(".{domain}");
        for (d, ips) in GITHUB_CANDIDATE_IPS {
            if d.ends_with(&suffix) {
                return Some((*d, *ips));
            }
        }
        None
    }

    /// Test all candidate IPs for a domain in parallel, return the fastest reachable one.
    fn test_domain(domain: &str, candidates: &[String]) -> Option<String> {
        let results: Arc<Mutex<Vec<TestResult>>> = Arc::new(Mutex::new(Vec::new()));

        let handles: Vec<_> = candidates
            .iter()
            .map(|ip| {
                let domain = domain.to_string();
                let ip = ip.clone();
                let results = Arc::clone(&results);
                std::thread::spawn(move || {
                    if let Some(r) = Self::test_tcp(&domain, &ip) {
                        results.lock().unwrap().push(r);
                    }
                })
            })
            .collect();

        for h in handles {
            let _ = h.join();
        }

        let best = results.lock().unwrap().iter().min_by_key(|r| r.latency_ms).map(|r| r.ip.clone());
        best
    }

    /// Test TCP+TLS connectivity to ip:443 and measure latency.
    ///
    /// Performs a full TLS handshake with SNI verification to ensure the IP
    /// not only accepts TCP connections but also serves a valid certificate
    /// for the target domain. This filters out IPs that are TCP-reachable
    /// but don't actually serve the expected HTTPS content (e.g., GFW
    /// interference patterns).
    fn test_tcp(domain: &str, ip: &str) -> Option<TestResult> {
        let addr = format!("{ip}:443");
        let start = Instant::now();

        // TCP connect
        let socket_addr = addr.to_socket_addrs().ok()?.next()?;
        let stream = TcpStream::connect_timeout(&socket_addr, TCP_TIMEOUT).ok()?;

        // Set read/write timeout on the socket so TLS handshake doesn't hang indefinitely
        // (without this, native_tls::connect uses the OS default TCP timeout ~30-120s)
        stream.set_read_timeout(Some(TCP_TIMEOUT)).ok()?;
        stream.set_write_timeout(Some(TCP_TIMEOUT)).ok()?;

        // TLS handshake with SNI verification
        let connector = TlsConnector::new().ok()?;
        let tls_stream = connector.connect(domain, stream).ok()?;
        drop(tls_stream);

        let latency_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

        Some(TestResult {
            ip: ip.to_string(),
            latency_ms,
        })
    }
}

impl Default for IpScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_candidates_exact_match() {
        let result = IpScanner::find_candidates("github.com");
        assert!(result.is_some());
        let (domain, ips) = result.unwrap();
        assert_eq!(domain, "github.com");
        assert!(!ips.is_empty());
    }

    #[test]
    fn find_candidates_suffix_match() {
        let result = IpScanner::find_candidates("www.github.com");
        assert!(result.is_some());
    }

    #[test]
    fn find_candidates_no_match() {
        let result = IpScanner::find_candidates("example.com");
        assert!(result.is_none());
    }

    #[test]
    fn find_candidates_raw_githubusercontent() {
        let result = IpScanner::find_candidates("raw.githubusercontent.com");
        assert!(result.is_some());
    }

    #[test]
    fn scan_domains_empty_input() {
        let scanner = IpScanner::new();
        let result = scanner.scan_domains(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn scan_domains_unknown_domain() {
        let scanner = IpScanner::new();
        let result = scanner.scan_domains(&["unknown.example.com".to_string()]);
        assert!(result.is_empty());
    }

    #[test]
    fn candidate_ips_data_integrity() {
        // Verify the constant data is well-formed
        for (domain, ips) in GITHUB_CANDIDATE_IPS {
            assert!(!domain.is_empty());
            assert!(!ips.is_empty());
            for ip in *ips {
                // Basic IP format check (xxx.xxx.xxx.xxx)
                let parts: Vec<&str> = ip.split('.').collect();
                assert_eq!(parts.len(), 4, "Invalid IP format: {ip}");
                for part in parts {
                    assert!(part.parse::<u8>().is_ok(), "Invalid IP octet: {part}");
                }
            }
        }
    }

    #[test]
    fn test_tcp_invalid_ip_returns_none() {
        // 0.0.0.0 is not a valid connect target
        let result = IpScanner::test_tcp("test.com", "0.0.0.0");
        assert!(result.is_none());
    }

    #[test]
    fn find_candidates_parent_match_githubassets() {
        // "githubassets.com" should match "github.githubassets.com" via parent match
        let result = IpScanner::find_candidates("githubassets.com");
        assert!(result.is_some());
        let (matched, ips) = result.unwrap();
        assert_eq!(matched, "github.githubassets.com");
        assert!(!ips.is_empty());
    }

    #[test]
    fn find_candidates_parent_match_githubusercontent() {
        // "githubusercontent.com" should match "raw.githubusercontent.com" via parent match
        let result = IpScanner::find_candidates("githubusercontent.com");
        assert!(result.is_some());
        let (matched, _) = result.unwrap();
        assert_eq!(matched, "raw.githubusercontent.com");
    }

    #[test]
    fn is_parent_match_returns_true_for_parent_domains() {
        assert!(IpScanner::is_parent_match("githubassets.com"));
        assert!(IpScanner::is_parent_match("githubusercontent.com"));
    }

    #[test]
    fn is_parent_match_returns_false_for_exact_domains() {
        assert!(!IpScanner::is_parent_match("github.com"));
        assert!(!IpScanner::is_parent_match("raw.githubusercontent.com"));
    }

    #[test]
    fn is_parent_match_returns_false_for_no_match() {
        assert!(!IpScanner::is_parent_match("example.com"));
    }

    #[test]
    fn scan_domains_returns_input_domain_as_key() {
        // Verify that the result uses INPUT domain as key, not candidate domain
        let scanner = IpScanner::new();
        // "githubassets.com" is a parent domain — verify it appears as key
        let result = scanner.scan_domains(&[
            "githubassets.com".to_string(),
            "github.com".to_string(),
        ]);
        // Both should have entries with input domain as key
        // (skipped if no candidates reachable, which is fine for CI)
        for key in result.keys() {
            assert!(
                key == "githubassets.com" || key == "github.com",
                "Unexpected key: {key}"
            );
        }
    }
}

/// Mock IP scanner for tests. Returns preset results without network access.
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct MockIpScanner {
    /// Preset domain → IP mapping to return.
    pub results: HashMap<String, String>,
}

#[cfg(test)]
impl MockIpScanner {
    #[must_use]
    pub fn new(results: HashMap<String, String>) -> Self {
        Self { results }
    }
}

#[cfg(test)]
impl IpScannerTrait for MockIpScanner {
    fn scan_domains(&self, domains: &[String]) -> HashMap<String, String> {
        domains
            .iter()
            .filter_map(|d| self.results.get(d).map(|ip| (d.clone(), ip.clone())))
            .collect()
    }
}
