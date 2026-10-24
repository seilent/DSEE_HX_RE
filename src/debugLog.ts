/**
 * 调试日志约定（导出后给人/AI 定位用）
 *
 * 行格式：
 *   <ISO8601>|ui|<event>|<k=v> <k=v>...
 *   <unix_ms>|rs|<event>|<k=v> <k=v>...
 *
 * event（尽量短、稳定）：
 *   boot settings media ingest analyze up_policy bits run hop err export
 *
 * 字段约定：
 *   path / out / n / up / bits / rate / in_fs / out_fs / ok / fail / err / wall_s / slots
 *   active / allowed / level / same / lang / theme / ffmpeg / ffprobe
 *
 * 原则：一行一事；成功摘要 + 失败必带 err 全文；路径尽量完整。
 */
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { resolveDialogStart } from "./dialogMemory";

function kv(fields?: Record<string, unknown>): string {
  if (!fields) return "";
  const parts: string[] = [];
  for (const [k, v] of Object.entries(fields)) {
    if (v === undefined || v === null) continue;
    let s: string;
    if (typeof v === "string") s = v.includes(" ") ? JSON.stringify(v) : v;
    else if (typeof v === "boolean" || typeof v === "number") s = String(v);
    else {
      try {
        s = JSON.stringify(v);
      } catch {
        s = String(v);
      }
    }
    parts.push(`${k}=${s}`);
  }
  return parts.length ? `|${parts.join(" ")}` : "";
}

/** UI 侧写一行 */
export function dlog(
  event: string,
  fields?: Record<string, unknown>,
): void {
  const line = `${new Date().toISOString()}|ui|${event}${kv(fields)}`;
  void invoke("debug_log_append", { line }).catch(() => {});
}

export async function exportDebugLog(): Promise<string | null> {
  const suggested = `dsee-hx-debug-${new Date()
    .toISOString()
    .replace(/[:.]/g, "-")
    .slice(0, 19)}.log`;
  const desktop = await resolveDialogStart();
  const defaultPath = desktop
    ? `${desktop.replace(/[\\/]$/, "")}${desktop.includes("\\") ? "\\" : "/"}${suggested}`
    : suggested;
  const path = await save({
    defaultPath,
    filters: [
      { name: "Log", extensions: ["log", "txt"] },
      { name: "All", extensions: ["*"] },
    ],
  });
  if (typeof path !== "string" || !path) return null;
  await invoke("debug_log_export", { path });
  dlog("export", { path });
  return path;
}
