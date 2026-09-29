//! Shared network / auth options passed to yt-dlp (proxy, cookies, rate limit).

use serde::{Deserialize, Serialize};

/// Options applied to both extract_info and download commands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkOptions {
    pub proxy: String,
    pub cookies_browser: String,
    pub rate_limit_kbps: u32,
    pub concurrent_fragments: u32,
}

impl NetworkOptions {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_proxy_and_rate() {
        let opts = NetworkOptions {
            proxy: "socks5://127.0.0.1:9050".into(),
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
    }

    #[test]
    fn empty_skips_flags() {
        let opts = NetworkOptions::default();
        let mut args = Vec::new();
        opts.append_cli_args(&mut args);
        assert!(args.is_empty());
    }
}
