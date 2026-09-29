# Video Downloader

**High-performance desktop media downloader & transcoder**

Built with **[GPUI-kit](https://gpui-kit.com)** (Rust native UI framework), **[yt-dlp](https://github.com/yt-dlp/yt-dlp)** (media extraction engine), and **[FFmpeg](https://ffmpeg.org)** (merging, remuxing, conversion).

A full Rust rewrite of the original [Video-Downloader](https://github.com/fazi-gondal/Video-Downloader) (Python + Flet) — preserving all features and architecture while delivering native performance and a polished desktop experience.

---

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Quick Start](#quick-start)
- [How It Works](#how-it-works)
- [Download Modes](#download-modes)
- [Format Selection](#format-selection)
- [Audio Tracks](#audio-tracks)
- [Subtitles](#subtitles)
- [Playlists](#playlists)
- [Built-in Converter](#built-in-converter)
- [Download Queue](#download-queue)
- [History](#history)
- [Settings](#settings)
- [Configuration File](#configuration-file)
- [Architecture](#architecture)
- [Development](#development)
- [CI & Packaging](#ci--packaging)
- [License](#license)

---

## Features

### Download Modes
| Mode | Description |
|------|-------------|
| **Video + Audio** | Best video stream merged with audio via FFmpeg |
| **Audio Only** | Extract to MP3, M4A, AAC, FLAC, Opus, or WAV |
| **Video Only** | Silent video track, no audio stream |

### Resolution Presets
| Preset | Max Height |
|--------|-----------|
| Best | Unlimited (highest available) |
| 4K | 2160p |
| 1080p | 1080p |
| 720p | 720p |
| 480p | 480p |
| 360p | 360p |

### Output Containers
| Container | Notes |
|-----------|-------|
| **MKV** | Best multi-audio and subtitle support — auto-selected when needed |
| **MP4** | Universal compatibility |
| **WebM** | Open web format |

> **Auto-switch**: MKV is automatically selected whenever multiple audio tracks or subtitles are enabled.

### Audio Tracks
- **Default Audio** — best audio stream (auto-selected)
- **All Audio Tracks** — embed every available language track (e.g. 22 dubbed tracks on MrBeast videos)
- **Select Tracks** — pick specific tracks from the detected list; checkboxes with scrollable picker
- Format IDs (not language guesses) are used for exact, reliable yt-dlp selection
- `--audio-multistreams` is automatically enabled for multi-track downloads

### Subtitles
- **None** — skip subtitles entirely
- **All Available** — embed every subtitle language + auto-generated captions
- **Select Languages** — tick specific languages from the detected list; scrollable picker
- Subtitles are written, embedded into the container via FFmpeg, then intermediate files are deleted
- Auto-generated captions (`--write-auto-subs`) are included when available

### Metadata Embedding
- **Thumbnail** — cover art embedded into the container
- **Metadata** — title, uploader, description, tags

### Playlists & Channels
- Full playlist/channel analysis in one click
- Checkbox selection: select all, clear all, or cherry-pick entries
- Batch download of selected entries with shared settings

### Download Queue
- Configurable concurrency (1-16 parallel downloads, default 8)
- Per-download fragment connections for faster segmented downloads
- Live progress: percentage, download speed, ETA
- Safe cancel — terminates the yt-dlp process cleanly
- Status: Pending → Downloading → Completed / Failed / Cancelled

### History
- Persistent download history (JSON file)
- Shows title, URL, status, file size, timestamp
- Re-download any entry directly from history

### Built-in Converter
- Convert local media files without re-downloading
- Profiles: Lossless Remux, H.264+AAC MP4, H.265+AAC MKV, MP3 Extract, FLAC Extract
- Conversion queue with live progress

### Network & Privacy
- HTTP and SOCKS5 proxy support
- Cookies exported from browser (chrome, firefox, edge, brave, opera, safari, vivaldi, chromium)
- Optional download rate limit (KB/s)
- `--extractor-args youtube:player_client=all` to expose all dubbed language tracks

### App Shell
- Sidebar navigation: Dashboard, Configure, Downloads, Converter, History, Settings, About
- Real-time theme switching: System / Dark / Light
- Toast notifications when downloads are queued
- About page with automatic runtime check for yt-dlp and ffmpeg

---

## Requirements

### Runtime Dependencies
| Tool | Version | Purpose |
|------|---------|---------|
| **yt-dlp** | Latest recommended | URL analysis, format extraction, download |
| **FFmpeg** | 6.0+ recommended | Stream merge, remux, subtitle embed, conversion |

Both must be on your PATH.

```bash
# Windows (winget)
winget install yt-dlp.yt-dlp
winget install Gyan.FFmpeg

# macOS (Homebrew)
brew install yt-dlp ffmpeg

# Debian / Ubuntu
sudo apt install yt-dlp ffmpeg

# Arch Linux
sudo pacman -S yt-dlp ffmpeg
```

### Build Dependencies
- **Rust** 1.80+ (edition 2024)
- Windows: no extra system dependencies (GPUI uses DirectX 11)
- Linux: X11/Wayland dev headers (see `.github/workflows/` for exact packages)
- macOS: Xcode command-line tools

---

## Quick Start

```bash
git clone https://github.com/fazi-gondal/downloader.git
cd downloader

# Verify runtime dependencies
yt-dlp --version
ffmpeg -version

# Run in development mode
cargo run
```

Release build:

```bash
cargo build --release
# Output: target/release/downloader      (Linux / macOS)
#         target/release/downloader.exe  (Windows)
```

---

## How It Works

```
User pastes URL  →  Dashboard analyzes via yt-dlp --dump-json
                          ↓
       MediaInfo parsed (formats, audio tracks, subtitles, thumbnail)
                          ↓
   Configure view: mode / resolution / container / audio tracks / subs
                          ↓
       FormatBuilder compiles DownloadOptions → yt-dlp CLI args
                          ↓
  DownloadManager spawns yt-dlp in a worker thread, streams progress
                          ↓
        FFmpeg post-processes (merge / embed subs / embed thumbnail)
                          ↓
              Final file saved to the download directory
```

---

## Download Modes

### Video + Audio (default)
Merges the best separate video and audio streams using FFmpeg. Container is determined by the selected output format (MP4, MKV, WebM).

### Audio Only
Extracts and re-encodes (or losslessly copies) audio to the selected format.

### Video Only
Downloads video stream only — no audio. Useful for silent footage or when audio is handled separately.

---

## Format Selection

`FormatBuilder` (`src/services/format_builder.rs`) translates UI options into a yt-dlp format selector string:

| Scenario | Format String |
|----------|--------------|
| Default best | `bv*+ba/b` |
| 1080p best | `bv*[height<=?1080]+ba/b[height<=?1080]/b` |
| 1080p 30fps cap | `bv*[height<=?1080][fps<=?30]+ba/b` |
| Specific format IDs | `137+251` |
| Single audio track (by ID) | `bv*[height<=?1080]+251/bv*[height<=?1080]+ba/b` |
| Multiple audio tracks (by ID) | `bv*+251+140/bv*+mergeall[vcodec=none]/bv*+ba/b` |
| All audio tracks mode | `bv*+mergeall[vcodec=none]/b` |

The `mergeall[vcodec=none]` fallback ensures multi-audio downloads never silently fail if a format ID becomes stale between analysis and download (common with YouTube).

---

## Audio Tracks

### How tracks are detected
yt-dlp returns all audio-only formats in the format list. The app groups them by language (or track name/format ID as fallback), keeping the best bitrate per unique track.

### Selection keys
Audio tracks are keyed by their **format ID** (e.g. `251`, `140`, `233-dub-en`). Format IDs are used directly in the yt-dlp format string, which is more reliable than language attribute filters that often return nothing.

### Multi-audio requirements
- Requires `--audio-multistreams` (added automatically)
- Requires **MKV** container (auto-switched)
- YouTube dubbed tracks are only exposed when `player_client=all` is used (always enabled in this app)

---

## Subtitles

### Embedding process
The app uses: `--write-subs --write-auto-subs --embed-subs --sub-langs LANGS`

yt-dlp's `FFmpegEmbedSubtitlePP` post-processor:
1. Downloads subtitle tracks as `.vtt`/`.srt` files
2. Merges them into the video container via FFmpeg
3. **Deletes the intermediate subtitle files** — only the final video remains

### Subtitle modes
| Mode | yt-dlp flags |
|------|-------------|
| None | *(omitted)* |
| All Available | `--write-subs --write-auto-subs --embed-subs --sub-langs all` |
| Custom (e.g. en + fr) | `--write-subs --write-auto-subs --embed-subs --sub-langs en,fr` |

> **Note**: MKV is recommended for subtitle embedding (auto-switched when subtitles are enabled).

---

## Playlists

1. Paste a playlist or channel URL and click **Analyze**
2. All entries are listed with checkboxes
3. Use **Select All** / **Clear** or tick individual entries
4. Configure download settings (mode, resolution, audio, subtitles) — applies to all selected entries
5. Click **Download Selected** → entries are queued

---

## Built-in Converter

Runs FFmpeg locally on existing media files — no internet required.

| Profile | Equivalent FFmpeg command |
|---------|--------------------------|
| Lossless Remux | `ffmpeg -i input -c copy output.mkv` |
| H.264 + AAC → MP4 | `ffmpeg -i input -c:v libx264 -c:a aac output.mp4` |
| H.265 + AAC → MKV | `ffmpeg -i input -c:v libx265 -c:a aac output.mkv` |
| Extract MP3 | `ffmpeg -i input -c:a libmp3lame -q:a 2 output.mp3` |
| Extract FLAC | `ffmpeg -i input -c:a flac output.flac` |

---

## Download Queue

- Queue managed by `DownloadManager` (`src/services/download_manager.rs`)
- Worker threads spawn `yt-dlp` as a child process
- stdout is streamed line-by-line; `[download]` lines are parsed for %, speed, ETA
- Completed tasks are moved to history automatically
- Failed tasks show the error message from yt-dlp stderr

---

## History

- Stored in `history.json` alongside `settings.json`
- Each entry: title, URL, file path, size (bytes), timestamp, success/failure
- Re-download button re-queues the URL with current settings
- History is loaded at startup and saved after each completed download

---

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| Download Directory | `~/Downloads/VideoDownloader` | Where files are saved |
| Max Concurrent Downloads | `8` | Parallel downloads (1-16) |
| Concurrent Fragments | `1` | Fragment threads per download |
| Proxy | *(empty)* | `http://host:port` or `socks5://host:port` |
| Cookies from Browser | *(empty)* | `chrome`, `firefox`, `edge`, `brave`, etc. |
| Rate Limit (KB/s) | `0` | `0` = unlimited |
| Write Subtitles | `false` | Default subtitle download on/off |
| Multi Audio | `false` | Default multi-audio track mode |
| Embed Thumbnail | `true` | Embed cover art into output file |
| Embed Metadata | `true` | Embed title, tags, description |
| Theme | `System` | `System` / `Dark` / `Light` |

---

## Configuration File

Settings are persisted as JSON in the OS application config directory:

| Platform | Path |
|----------|------|
| **Windows** | `%LOCALAPPDATA%\VideoDownloader\settings.json` |
| **macOS** | `~/Library/Application Support/VideoDownloader/settings.json` |
| **Linux** | `~/.config/VideoDownloader/settings.json` |

History is stored alongside as `history.json`.

Example `settings.json`:
```json
{
  "download_dir": "C:\\Users\\User\\Downloads\\VideoDownloader",
  "max_concurrent": 8,
  "concurrent_fragments": 4,
  "proxy": "",
  "cookies_browser": "",
  "rate_limit_kbps": 0,
  "write_subtitles": false,
  "subtitle_langs": [],
  "multi_audio": false,
  "embed_thumbnail": true,
  "embed_metadata": true,
  "prefer_vp9_video": true,
  "theme_mode": "system"
}
```

---

## Architecture

```
src/
├── main.rs                        # App bootstrap, window options
├── config/
│   └── mod.rs                     # AppSettings - load/save JSON
├── core/
│   ├── app_state.rs               # Shared AppState entity (media, options, services)
│   ├── errors.rs                  # AppError types
│   ├── events.rs                  # Cross-component events
│   └── navigation.rs              # AppRoute enum (Dashboard, Configure, ...)
├── models/
│   ├── media.rs                   # MediaInfo, FormatInfo, AudioTrack, SubtitleTrack,
│   │                              #   DownloadOptions, DownloadMode, SubtitleMode,
│   │                              #   AudioTracksMode, ResolutionPreset, Container
│   ├── download.rs                # DownloadTask, DownloadStatus
│   ├── conversion.rs              # ConversionTask, ConversionProfile
│   └── history.rs                 # HistoryEntry
├── services/
│   ├── ytdlp.rs                   # CLI spawn -> JSON parse -> MediaInfo / PlaylistInfo
│   ├── format_builder.rs          # DownloadOptions -> yt-dlp format string + extra args
│   ├── download_manager.rs        # Thread pool, process spawn, progress parse, cancel
│   ├── conversion.rs              # FFmpeg conversion queue
│   ├── ffmpeg.rs                  # FFmpeg helpers (remux, multi-audio merge)
│   ├── history.rs                 # Persistent history read/write
│   └── network_options.rs         # Proxy, cookies, rate limit, fragments -> CLI args
└── ui/
    ├── app_shell.rs               # Root layout: sidebar + routed view
    ├── theme.rs                   # Theme tokens (dark/light/system)
    ├── prelude.rs                 # Common re-exports for views
    ├── components/
    │   ├── media_card.rs          # Single / playlist media info card
    │   ├── empty_state.rs         # Empty placeholder component
    │   └── status_pill.rs         # Download status badge
    └── views/
        ├── dashboard.rs           # URL input, analyze, media result cards
        ├── config_view.rs         # Mode / format / audio / subtitle config + download button
        ├── downloads.rs           # Live download queue list
        ├── converter.rs           # Local file converter UI
        ├── history.rs             # Download history list
        ├── settings.rs            # App settings form
        └── about.rs               # Dependency info + auto runtime check
```

### Design Principles

1. **UI thread safety** — yt-dlp and FFmpeg always run in worker threads. The UI never blocks.
2. **Single shared state** — one `Entity<AppState>` owned by `AppShell`, passed by reference to every view.
3. **Pure FormatBuilder** — `FormatBuilder::compile` is a pure function (no I/O, no side effects). Fully unit-testable.
4. **GPUI-kit components only** — Button, Input, Checkbox, Scrollbar, ScrollHandle from the documented component set.
5. **Deterministic format fallbacks** — every format selector has a `/fallback` chain so downloads never silently fail.

---

## Development

```bash
cargo run                          # debug build with live logging
cargo test                         # run all unit tests
cargo clippy --all-targets         # lint
cargo fmt --all                    # format
```

### Key crates
| Crate | Purpose |
|-------|---------|
| `gpui-kit 0.7` | Native UI framework (GPUI + component library) |
| `serde` + `serde_json` | Settings and history serialization |
| `dirs 6` | OS-specific config/data directories |
| `thiserror 2` | Ergonomic error types |
| `uuid 1` | Unique download task IDs |
| `chrono 0.4` | Timestamps for history entries |
| `log` + `env_logger` | Structured logging |

---

## CI & Packaging

| Workflow | Trigger | Steps |
|----------|---------|-------|
| `ci.yml` | Push / PR | fmt check -> clippy -> test on Linux, Windows, macOS |
| `build.yml` | Published release | Release binaries for all three platforms as artifacts |

```bash
git tag v0.2.0
git push origin v0.2.0
# Publish a GitHub Release from the tag
# CI builds and attaches platform binaries automatically
```

Release profile (`Cargo.toml`):
```toml
[profile.release]
lto = true          # Link-time optimisation
codegen-units = 1   # Best optimisation
strip = true        # Strip debug symbols
```

---

## License

MIT — see [LICENSE](LICENSE).

Crafted as a Rust + GPUI-kit port of the original Video Downloader by **Fazi Gondal**.
