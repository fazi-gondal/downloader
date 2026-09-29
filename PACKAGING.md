# Packaging & distribution

## Local release build

```bash
cargo build --release
```

Binary location:

- Linux/macOS: `target/release/downloader`
- Windows: `target/release/downloader.exe`

## Runtime dependencies

Users need:

1. **yt-dlp** on `PATH` (or set a custom path later via settings)
2. **FFmpeg** on `PATH` for merge / convert

Optional: install via package manager

```bash
# Debian/Ubuntu
sudo apt install yt-dlp ffmpeg

# macOS
brew install yt-dlp ffmpeg

# Windows (winget / chocolatey)
winget install yt-dlp.yt-dlp
winget install Gyan.FFmpeg
```

## CI

GitHub Actions workflow (`.github/workflows/rust.yml`):

- On every push/PR: `fmt`, `clippy`, `test` on Linux, Windows, macOS
- On published release tags: release binaries uploaded as artifacts

## Creating a release

```bash
git tag v0.1.0
git push origin v0.1.0
# Create a GitHub Release from the tag — CI builds and attaches binaries
```

## Notes

- GPUI apps need platform GUI libraries at build and runtime (see CI Linux apt packages).
- For a fully offline installer (bundled yt-dlp/FFmpeg), extend `DownloadManager` / `FfmpegService` to prefer vendored binaries under an app data directory — out of scope for the core Phase 4 surface.
