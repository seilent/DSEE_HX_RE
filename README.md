# Sony DSEE HX — Reverse Engineering & Reimplementation

**Sony DSEE HX：逆向与实现**

Offline rebuild of the Sony **DSEE HX** bandwidth-extension filter from Music Center: ffmpeg decode + a C kernel (HEQ / DSX), writing 16/24/32-bit WAV.

Sony Music Center 里 **DSEE HX** 带宽扩展滤镜的离线实现：ffmpeg 解码 + C 内核（HEQ / DSX），输出 16/24/32-bit WAV。

---

## Download / 下载

Get the latest Windows build from **[Releases](https://github.com/x1aoqv/DSEE_HX_RE/releases)**.

到 **[Releases](https://github.com/x1aoqv/DSEE_HX_RE/releases)** 下载最新 Windows 安装包 / 可执行文件。

Requires **Windows x64**. ffmpeg + ffprobe must be available (on `PATH`, or set paths in Settings).

需要 **Windows x64**。系统需能找到 ffmpeg / ffprobe（加入 `PATH`，或在设置里指定路径）。

---

## Quick start / 快速开始

<!-- screenshot: docs/screenshots/01-empty.png -->

1. Open **DSEE HX RE**.
2. Click **Add** / 添加, or drag audio files onto the window.
3. Click **Browse** / 浏览 to choose an output folder.
4. Adjust per-track (or toolbar batch) **upsample** and **bit depth** if needed.
5. Click **Start** / 开始 and wait until rows show done.

1. 打开 **DSEE HX RE**。
2. 点 **添加**，或把音频拖进窗口。
3. 点 **浏览** 选择输出文件夹。
4. 按需改每行（或工具栏批量）的 **上采样**、**位深**。
5. 点 **开始**，等到各行状态完成即可。

Supported inputs include common formats ffmpeg can decode (WAV / FLAC / MP3 / M4A / …). Outputs are WAV.

输入为 ffmpeg 能解的常见格式；输出为 WAV。

---

## Main window / 主界面

<!-- screenshot: docs/screenshots/02-queue.png -->

| Area / 区域 | What it does / 作用 |
|--|--|
| **Add** / 添加 | Pick files into the queue / 把文件加入列表 |
| **Browse** / 浏览 | Set output directory / 设定输出目录 |
| Table rows / 表格行 | Per-track sample rate, same-rate / upsample, bits, status / 采样率、同率或上采样、位深、状态 |
| Toolbar dropdowns / 工具栏下拉 | Batch-change Level / upsample / bits for all tracks / 批量改 Level、倍率、位深 |
| **Start** / 开始 | Run the queue / 开始处理 |
| **Settings** / 设置 | Language, theme, ffmpeg paths, advanced / 语言、主题、ffmpeg、高级 |

**Upsample** must stay in the same rate family as the source (e.g. 44.1k → 44.1 / 88.2 / 176.4; 48k → 48 / 96 / 192). The UI only offers valid factors.

**上采样**须与源采样率同族；界面只会列出允许的倍率。

---

## Settings / 设置

<!-- screenshot: docs/screenshots/03-settings.png -->

- **Language / 语言** — Chinese or English UI / 中英界面  
- **Theme / 主题** — base color / 基础配色  
- **FFmpeg / FFprobe** — custom paths or auto-detect / 自定义路径或自动检测  
- **Advanced / 高级** — unlock manual same-rate Level, `1×`, and HEQ bypass (results may differ from stock DSEE HX) / 解锁手动同率 Level、`1×`、关闭 HEQ（结果可能与原版不同）  
- **About / 关于** — version, repository, credits / 版本、仓库、署名  

---

## Tips / 提示

- Prefer a dedicated empty output folder so results are easy to find. / 建议单独空目录作输出，方便找结果。  
- Long albums: leave the window running; status updates per track. / 长专辑可挂机跑，状态按曲更新。  
- If analyze fails, check that ffmpeg/ffprobe work in Settings. / 分析失败时先到设置确认 ffmpeg/ffprobe 可用。  

---

## Build from source / 从源码构建（可选）

Developers only. End users should use Releases.

仅开发者需要；普通用户请用 Releases。

```bat
copy .cargo\config.toml.example .cargo\config.toml
npm install
npm run tauri:dev
```

```bat
npm run tauri:build
```

Needs Node 22+, Rust `stable-x86_64-pc-windows-gnu`, MinGW-w64 `gcc`, and ffmpeg/ffprobe.

---

## License / 许可说明

For research / interoperability study. DSEE HX, Music Center, and related marks belong to their owners. Check local law and licenses before your use case.

研究 / 互操作学习用途。DSEE HX、Music Center 等商标归权利人所有。请自行确认当地法律与许可是否允许你的用途。
