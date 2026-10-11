//! Shared network / auth options passed to yt-dlp (proxy, cookies, rate limit,
//! and custom HTTP headers).

use serde::{Deserialize, Serialize};

/// Options applied to both extract_info and download commands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkOptions {
    pub proxy: String,
    /// One custom HTTP header per line in `Header-Name: value` format.
    #[serde(default)]
    pub custom_headers: String,
    pub cookies_browser: String,
    pub rate_limit_kbps: u32,
    pub concurrent_fragments: u32,
}

impl NetworkOptions {
    /// Parse the user-editable custom header format into safe yt-dlp values.
    ///
    /// Blank lines and lines beginning with `#` are ignored. Invalid header
    /// names, missing values, and control characters are ignored rather than
    /// being passed to the subprocess.
    pub fn parse_custom_headers(input: &str) -> Vec<String> {
        input
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }

                let (name, value) = line.split_once(':')?;
                let name = name.trim();
                let value = value.trim();
                if name.is_empty()
                    || value.is_empty()
                    || !name.bytes().all(is_header_name_byte)
                    || value.chars().any(char::is_control)
                {
                    return None;
                }

                Some(format!("{name}: {value}"))
            })
            .collect()
    }

    /// Append matching yt-dlp CLI flags to `args`.
    pub fn append_cli_args(&self, args: &mut Vec<String>) {
        if !self.proxy.trim().is_empty() {
            args.push("--proxy".into());
            args.push(self.proxy.trim().to_string());
        }
        if !self.cookies_browser.trim().is_empty() {
            args.push("--cookies-from-browser".into());
            args.push(self.cookies_browser.trim().to_string());
        }
        for header in Self::parse_custom_headers(&self.custom_headers) {
            args.push("--add-header".into());
            args.push(header);
        }
        if self.rate_limit_kbps > 0 {
            args.push("--limit-rate".into());
            args.push(format!("{}K", self.rate_limit_kbps));
        }
        if self.concurrent_fragments > 1 {
            args.push("--concurrent-fragments".into());
            args.push(self.concurrent_fragments.to_string());
        }
    }
}

fn is_header_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_proxy_and_rate() {
        let opts = NetworkOptions {
            proxy: "socks5://127.0.0.1:9050".into(),
            custom_headers: "X-Test: enabled\nReferer: https://example.test/watch".into(),
            cookies_browser: "firefox".into(),
            rate_limit_kbps: 500,
            concurrent_fragments: 4,
        };
        let mut args = Vec::new();
        opts.append_cli_args(&mut args);
        assert!(args.contains(&"--proxy".into()));
        assert!(args.contains(&"socks5://127.0.0.1:9050".into()));
        assert!(args.contains(&"--cookies-from-browser".into()));
        assert!(args.contains(&"--limit-rate".into()));
        assert!(args.contains(&"500K".into()));
        assert!(args.contains(&"--concurrent-fragments".into()));
        let expected: Vec<String> = [
            "--proxy",
            "socks5://127.0.0.1:9050",
            "--cookies-from-browser",
            "firefox",
            "--add-header",
            "X-Test: enabled",
            "--add-header",
            "Referer: https://example.test/watch",
            "--limit-rate",
            "500K",
            "--concurrent-fragments",
            "4",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        assert_eq!(args, expected);
    }

    #[test]
    fn empty_skips_flags() {
        let opts = NetworkOptions::default();
        let mut args = Vec::new();
        opts.append_cli_args(&mut args);
        assert!(args.is_empty());
    }

    #[test]
    fn parses_valid_headers_and_skips_blank_or_invalid_lines() {
        let headers = NetworkOptions::parse_custom_headers(
            "\n  # comments are ignored\nUser-Agent: downloader\nInvalid\n: missing name\nX-Empty:\nBad Header: value\nX-Trace: value:with:colons\n",
        );

        assert_eq!(
            headers,
            vec!["User-Agent: downloader", "X-Trace: value:with:colons",]
        );
    }

    #[test]
    fn rejects_control_characters_in_header_values() {
        let headers = NetworkOptions::parse_custom_headers("X-Good: yes\nX-Bad: no\u{7f}please");

        assert_eq!(headers, vec!["X-Good: yes"]);
    }
}
