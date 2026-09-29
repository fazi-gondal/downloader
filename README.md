# Video Downloader

**High-performance desktop media downloader & transcoder**

Built with **[GPUI-kit](https://gpui-kit.com)** (Rust native UI), **[yt-dlp](https://github.com/yt-dlp/yt-dlp)** (extraction engine), and **[FFmpeg](https://ffmpeg.org)** (media processing).

This is a full rewrite of the original [Video-Downloader](https://github.com/fazi-gondal/Video-Downloader) (Python + Flet), preserving architecture, features, and design intent while using **default system window controls** (minimize / maximize / close — not custom traffic-light chrome).

---

## Features

### Download modes & formats
- **Video + Audio** — best video and audio streams, merged via FFmpeg
- **Audio only** — extract to MP3, M4A, AAC, FLAC, Opus, or WAV
- **Video only** — silent video tracks
- Resolution presets: Best, 4K, 1080p, 720p, 480p (and more in the model layer)
- Containers driven by format selection and post-processing

### Tracks & metadata
- Multi-language audio (all tracks when enabled)
- Subtitles download and soft embedding
- Embed thumbnail and metadata
- Live estimated file size from stream data

### Playlists
- Playlist / channel analysis
- Select all, select none, or cherry-pick entries before download

### Queue & concurrency
- Configurable concurrent downloads (default 8, max 16)
- Concurrent fragments per download
- Live progress (%), speed, ETA
- Safe cancel

### Network & privacy
- HTTP / SOCKS proxy
- Cookies from browser (`chrome`, `firefox`, `edge`, `brave`, …)
- Optional rate limit (KB/s)

### Built-in converter
- Lossless remux (stream copy)
- H.264 + AAC (MP4), H.265 + AAC (MKV)
- Audio extract to MP3 / FLAC
- Local file path input and conversion queue

### App
- Sidebar navigation: Dashboard, Configure, Downloads, Converter, History, Settings, About
- Persistent settings and download history (JSON in the OS config directory)
- Theme preference: System / Dark / Light
- Dependency inspector for yt-dlp and FFmpeg (About)
- Toast when downloads are queued

---

## Requirements

### Runtime
| Tool | Purpose |
|------|---------|
| **yt-dlp** | URL analysis and download |
| **FFmpeg** | Merge, remux, convert |

Both must be on your `PATH`. No Deno, Bun, or Node.js is required by this app.

```bash
# Debian / Ubuntu
sudo apt install yt-dlp ffmpeg

# macOS
brew install yt-dlp ffmpeg

# Windows (examples)
winget install yt-dlp.yt-dlp
winget install Gyan.FFmpeg
```

### Build
- **Rust** 1.80+ (edition 2024)
- Platform GUI libraries required by GPUI (see CI workflow for Linux packages)

---

## Quick start

```bash
git clone https://github.com/fazi-gondal/downloader.git
cd downloader

# Ensure yt-dlp and ffmpeg are available
yt-dlp --version
ffmpeg -version

cargo run
```

Release build:

```bash
cargo build --release
# Binary: target/release/downloader  (or .exe on Windows)
```

---

## Development

```bash
cargo run              # debug build
cargo test             # unit tests
cargo clippy --all-targets
cargo fmt --all
```

Or via Makefile:

```bash
make test
make lint
make fmt
make check             # fmt + clippy + test
```

---

## Configuration

Settings are stored as JSON under the OS application config directory:

| Platform | Path |
|----------|------|
| Linux | `~/.config/VideoDownloader/settings.json` |
| macOS | `~/Library/Application Support/VideoDownloader/settings.json` |
| Windows | `%LOCALAPPDATA%\VideoDownloader\settings.json` |

History is stored alongside as `history.json`.

| Setting | Default | Description |
|---------|---------|-------------|
| `download_dir` | `~/Downloads/VideoDownloader` | Output directory |
| `max_concurrent` | `8` | Parallel downloads (1–16) |
| `concurrent_fragments` | `1` | Fragment connections per download |
| `proxy` | _(empty)_ | e.g. `http://…` or `socks5://…` |
| `cookies_browser` | _(empty)_ | Browser name for cookie export |
| `rate_limit_kbps` | `0` | Cap in KB/s (`0` = unlimited) |
| `keep_originals` | `true` | Keep sources after merge/convert |
| `write_subtitles` | `false` | Default subtitle download |
| `multi_audio` | `false` | Default multi-audio |
| `embed_thumbnail` | `true` | Embed cover art |
| `embed_metadata` | `true` | Embed title / tags |
| `prefer_vp9_video` | `true` | Prefer VP9 when available |
| `theme_mode` | `system` | `system` / `dark` / `light` |

---

## Architecture

```text
src/
├── main.rs                 # Bootstrap, default WindowOptions, Root
├── core/                   # AppState, errors, events, navigation
├── config/                 # AppSettings + JSON persistence
├── models/                 # MediaInfo, DownloadTask, ConversionTask, HistoryEntry
├── services/
│   ├── ytdlp.rs            # CLI extract + JSON parse
│   ├── format_builder.rs   # Pure UI options → yt-dlp args + size estimate
│   ├── download_manager.rs # Queue, spawn, progress, cancel
│   ├── conversion.rs       # Local FFmpeg conversion queue
│   ├── history.rs          # Persistent history
│   ├── ffmpeg.rs           # Remux / multi-audio merge
│   └── network_options.rs  # Proxy, cookies, rate limit, fragments
└── ui/
    ├── app_shell.rs        # Sidebar + routing, owns Entity<AppState>
    └── views/              # Dashboard, Config, Downloads, Converter, History, Settings, About
```

**Design tenets**
1. **Task isolation** — yt-dlp and FFmpeg run off the UI thread (`background_spawn` / worker threads).
2. **Shared AppState** — one `Entity<AppState>` for media, options, downloads, history, settings.
3. **Pure FormatBuilder** — deterministic format selectors; unit-tested.
4. **GPUI-kit only** — documented components and coding/design guides; default system window chrome.

---

## Packaging & CI

See **[PACKAGING.md](PACKAGING.md)** for release builds and distribution notes.

GitHub Actions (`.github/workflows/rust.yml`):
- On push/PR: `fmt`, `clippy`, `test` on Linux, Windows, and macOS
- On published release: multi-platform release binaries as artifacts

```bash
git tag v0.1.0
git push origin v0.1.0
# Publish a GitHub Release from the tag to trigger binary builds
```

---

## License

MIT — see [`LICENSE`](LICENSE) if present.

Crafted as a Rust + GPUI-kit port of the original Video Downloader by **Fazi Gondal**.
