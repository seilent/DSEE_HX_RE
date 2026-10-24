# DSEE HX（桌面软件）

**唯一硬指标：与 Sony Music Center 的 DSEE HX 输出严格一致（bit-exact / hop 对照）。**  
语言不统一强制——哪个模块合适用哪个。

## 模块与语言

| 模块 | 语言 | 原因 |
|---|---|---|
| DSP 内核（HEQ / F76 / F80 / ext80 / 码本） | **C**（`native/`） | 已从 AX 对齐；`-ffp-contract=off` + 软 80 位；改语言无收益 |
| 桌面壳 / 选文件 / 调内核 | **Rust + Tauri 2** | 窗口与 FFI 方便；只做薄封装 |
| UI | **React (TS)** | 界面 |
| 解码 | **ffmpeg**（进程） | 与 FINAL 相同策略 |
| 对照 / Frida / 回归 | **Python** | 已有 hook 与 hop 对比脚本 |

可选以后再加：WebGPU/WGSL 仅作**近似预览或频谱显示**，不进正式导出路径。

## 结构

```
src/                 React UI
src-tauri/           Tauri 壳（调 dsee-kern）
crates/dsee-kern/    Rust API → 链接 native C
native/kern|config/  与 DSEE HX FINAL 同源的 C（一致性来源）
```

## 跑起来

依赖：Node 22+、Rust（本仓库默认 `windows-gnu` + MinGW `gcc`）、ffmpeg/ffprobe（PATH，或仓库旁 `issue1/ffmpeg/.../bin`，或 `DSEE_FFMPEG`）。

```bat
npm install
npm run tauri:dev
```

打包：

```bat
npm run tauri:build
```

## 一致性

1. 默认后端始终是 vendored C（与 FINAL / MC 对齐路径相同）。  
2. 与 Music Center 的 Frida/hop 对照继续用 FINAL 的 hooks。  
3. 不一致只改 `native/` 的 C，不改壳。
