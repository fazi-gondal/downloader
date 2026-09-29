Here is an updated plan after comparing the **reference** (`Video-Downloader` Python/Flet)[https://github.com/fazi-gondal/Video-Downloader] code with the **current Rust** project.

---

# Video Downloader (Rust + GPUI-kit) — Feature Plan

## Goal

Match the reference app’s **architecture and user-facing capabilities**, implemented in Rust + GPUI-kit, with **default OS window controls** (no custom traffic lights).

---

## A. Already included (in Rust source)

| Area | What we have |
|------|----------------|
| **Shell** | Sidebar routes: Dashboard, Configure, Downloads, Converter, History, Settings, About |
| **Window** | Default system minimize / maximize / close |
| **Analysis** | yt-dlp JSON extract for single video + playlist |
| **Modes** | Video+Audio, Audio only, Video only |
| **Resolution** | Best, 4K, 1080p, 720p, 480p (partial preset list) |
| **Config toggles** | Multi-audio, write/embed subtitles, embed thumbnail |
| **Playlist** | Select all / none / per-entry checkboxes |
| **Queue** | Concurrent downloads, progress fields, cancel (cooperative) |
| **Network** | Proxy, cookies-from-browser, rate limit, concurrent fragments |
| **FormatBuilder** | Basic format selectors + size estimate + multi-audio flag |
| **Converter** | Remux, H.264 MP4, H.265 MKV, MP3, FLAC profiles + queue |
| **History** | JSON persistence, list, remove, clear, auto-sync from downloads |
| **Settings** | Download dir, concurrency, fragments, rate limit, proxy, cookies, theme preference, default toggles |
| **About** | Version + yt-dlp / FFmpeg check |
| **CI / docs** | GitHub Actions, PACKAGING.md, README |

These cover the **core pipeline**: analyze → configure → download → history / convert / settings.

---

## B. Missing vs reference (confirmed in reference code)

### B1 — Configure / download options (high impact)

| Missing feature | Reference location | Notes |
|-----------------|--------------------|--------|
| **Video container chips** (mp4 / mkv / webm) | `constants.VIDEO_CONTAINERS`, ConfigView | Model has `Container` but UI does not fully drive it |
| **Full resolution ladder** (1440p, 360p, 240p, 144p) | `RESOLUTION_PRESETS` | Only subset in UI |
| **FPS presets** (Any / 60 / 30) | `FPS_PRESETS`, ConfigView | Not in Rust Config UI |
| **Audio format chips** (mp3, m4a, aac, flac, opus, wav) | `AUDIO_FORMATS` | Model has `AudioFormat`; UI incomplete |
| **Audio quality / bitrate** (Best, 320…128, custom kbps) | `AUDIO_QUALITY_PRESETS`, custom field | Missing |
| **Per-track multi-audio picker** (language / format_id, not only “all”) | `selected_audio_track_ids`, ConfigView checkboxes | Only global multi-audio boolean |
| **Subtitle modes** (none / all / custom langs + list) | ConfigView subtitle mode chips | Only single checkbox |
| **Manual format explorer** (`FormatTable`: pick video + audio format ids) | `format_table.py`, ConfigView expand | Missing entirely |
| **Media card** (thumbnail + title + uploader) | Config / Dashboard | Thumbnail not shown |
| **Destination folder picker** on Config | `FolderPicker` | Text path only in Settings |
| **Live size estimate** with duration × bitrate logic | ConfigView `_calculate_estimated_size` | Basic estimate only |
| **Playlist folder naming** (`Playlist Title/` template) | `playlist_title` / output template | Not wired |
| **Prefer VP9 / format_sort** in selectors | FormatBuilder + settings | Setting exists; selector logic partial |
| **Explicit `video_format_id` / `audio_format_id`** override | `DownloadRequest` | Not in options flow |

### B2 — Download engine / post-process

| Missing feature | Reference | Notes |
|-----------------|-----------|--------|
| **States: preparing / processing** (merge phase) | `DownloadState` | Rust has fewer states |
| **Hard process kill + temp cleanup** | Download manager | Cooperative cancel only |
| **Thumbnail in download tiles** | DownloadRequest.thumbnail_url | Missing |
| **Output path open / reveal in file manager** | History / Downloads | Missing |
| **Retry failed task** | History redownload | Missing |
| **JS runtimes for yt-dlp** (optional deno/node/bun discovery) | `format_builder._get_js_runtimes` | Not required by app, but reference detects them |
| **YouTube player_client / rich vs fast analysis** | `build_analysis_opts` | Missing |
| **Custom HTTP headers** | Settings + format_builder | Missing |
| **Innertube helper** (extra YouTube audio language discovery) | `innertube.py` | Missing (optional enrichment) |
| **Bundled binary resolution** (packaged ffmpeg/deno) | `bundled_tools.py` | Missing |

### B3 — Converter

| Missing feature | Reference | Notes |
|-----------------|-----------|--------|
| **Remux vs reencode as modes** + free container/codec choice | `ConversionMode`, containers list | Fixed profiles only |
| **Audio bitrate on convert** | `ConversionRequest.audio_bitrate_kbps` | Missing |
| **Batch multi-file convert** | Converter view | Single path input |
| **Keep original toggle per job** | ConversionRequest | Global setting only |
| **Native file picker** for source | Converter | Path text field only |

### B4 — History / Downloads UX

| Missing feature | Reference | Notes |
|-----------------|-----------|--------|
| **Filter pills** (all / completed / failed) | HistoryView | Missing |
| **Re-download from history** | `_redownload` | Missing |
| **Open containing folder** | `_open_folder` | Missing |
| **Richer progress UI** (bytes, speed, ETA formatting) | Download tiles | Partial |
| **Empty states / skeletons** | UI components | Minimal placeholders |

### B5 — Settings / About / app chrome

| Missing feature | Reference | Notes |
|-----------------|-----------|--------|
| **Custom HTTP headers editor** | Settings `_parse_headers` | Missing |
| **Settings import / export JSON** | SettingsView | Missing |
| **Dependency cards with install links** (FFmpeg, Deno guide) | Settings + constants URLs | Partial check only |
| **Live theme apply** | Theme selector | Preference saved; runtime apply incomplete |
| **i18n / text catalog** | `ui/texts` | Hardcoded English strings |
| **Event bus** (typed progress events) | `event_bus` / events | Polling / notify instead |
| **Toast system** as first-class component | `toast.py` | One `push_notification` use only |
| **Custom window controls** | Reference has them | **Intentionally not copied** |

### B6 — Packaging / distribution

| Missing feature | Reference | Notes |
|-----------------|-----------|--------|
| **Windows installer / bundled tools layout** | packaging + `assets/bin` | Docs only |
| **Full multi-platform release artifacts** | Reference CI notes | Workflow scaffold exists |

---

## C. Recreated implementation plan (phased)

Use this to close the gaps without redoing what already works.

### Phase 6 — Configure parity (priority)

1. Container chips: mp4 / mkv / webm  
2. Full resolution + FPS presets  
3. Audio format + quality (incl. custom bitrate)  
4. Subtitle mode: none / all / custom language list  
5. Per-track audio selection (language / format_id)  
6. Wire `DownloadOptions` → FormatBuilder for all of the above  
7. Config destination folder field (or reuse settings path)  
8. Stronger size estimate (duration × tbr / filesize)  
9. Playlist output template (`playlist_title` folder)

### Phase 7 — Format explorer

1. `FormatTable` view (id, ext, res, fps, codecs, bitrate, size, type)  
2. Select video row + audio row → set `video_format_id` / `audio_format_id`  
3. Override presets when manual IDs set  

### Phase 8 — Download engine hardening

1. States: preparing / downloading / processing  
2. Kill child process on cancel + cleanup  
3. Thumbnail URL on tasks / tiles  
4. Custom headers in NetworkOptions  
5. Optional analysis opts (timeout, player_client)  
6. Open folder / retry from Downloads + History  

### Phase 9 — Converter parity

1. Kind: video/audio; mode: remux/reencode  
2. Container/format lists from constants  
3. Bitrate for lossy audio  
4. Multi-file queue (list of paths)  
5. Keep-original per job  

### Phase 10 — History / Settings / polish

1. History filters + redownload + open folder  
2. Settings: custom headers, import/export  
3. About/Settings dependency cards + install links  
4. Empty states, better progress formatting  
5. Live theme apply (gpui-kit Appearance API)  
6. Optional: innertube enrichment, JS runtime discovery for yt-dlp, bundled tool paths  

### Phase 11 — Packaging (optional)

1. Bundle ffmpeg (and optionally deno) for Windows  
2. Installer / release assets aligned with reference  

---

## D. What we will **not** copy

- MacBook-style traffic light window controls  
- Flet-specific widgets (reimplemented with GPUI-kit components only)  
- Requiring Deno/Bun/Node as a hard dependency of the app binary  

---

## E. Suggested order of work

| Order | Phase | Outcome |
|-------|--------|---------|
| 1 | **6** Configure parity | Users can choose container, fps, audio format/bitrate, subtitle modes, per-track audio |
| 2 | **7** Format explorer | Power users pick exact streams |
| 3 | **8** Download hardening | Reliable cancel, processing state, headers, open/retry |
| 4 | **9** Converter parity | Flexible local convert |
| 5 | **10** History/Settings polish | Filters, redownload, import/export, theme |
| 6 | **11** Packaging | Optional shipping |

---

## F. Summary

- **Included:** end-to-end shell, analyze, basic configure, queue, network flags, converter profiles, history, settings, about, CI.  
- **Missing (main gaps):** full Config option matrix (container, fps, audio format/bitrate, subtitle modes, per-track audio), **format explorer**, richer download lifecycle, custom headers, history filters/redownload/open folder, converter flexibility, settings import/export, live theme, packaging/bundled tools.