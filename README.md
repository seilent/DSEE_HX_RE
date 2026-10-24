# DSEE HX RE

Sony Music Center **DSEE HX** 的离线桌面实现（bit-exact 对齐路径）。

| 模块 | 技术 |
|---|---|
| DSP 内核 | C（`native/`） |
| 壳 | Rust + Tauri 2（`src-tauri/` → `crates/dsee-kern`） |
| UI | React + TypeScript（`src/`） |
| 解码 | ffmpeg / ffprobe |

## 开发

依赖：Node 22+、Rust `stable-x86_64-pc-windows-gnu`、MinGW-w64 `gcc`、ffmpeg/ffprobe（PATH，或 `DSEE_FFMPEG` / `DSEE_FFPROBE`）。

```bat
copy .cargo\config.toml.example .cargo\config.toml
npm install
npm run tauri:dev
```

路径含空格时，在 `.cargo/config.toml` 里把 `target-dir` 指到无空格目录。

## 打包

```bat
npm run tauri:build
```

产物在 `src-tauri/target/...` 或你配置的 `target-dir` 下。

## 许可

研究 / 互操作学习用途。DSEE HX、Music Center 等商标归权利人所有。
