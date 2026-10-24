/** 各对话框路径分开记忆；无记忆时回落到桌面 */

import { desktopDir } from "@tauri-apps/api/path";

const KEY = "dsee-hx-dialog-paths";

export type DialogMemory = {
  /** 添加音频：上次所在文件夹 */
  addDir: string;
  /** 浏览输出：上次选中的输出文件夹 */
  outDir: string;
  /** ffmpeg 浏览：上次选中的文件路径 */
  ffmpegPath: string;
  /** ffprobe 浏览：上次选中的文件路径 */
  ffprobePath: string;
};

const EMPTY: DialogMemory = {
  addDir: "",
  outDir: "",
  ffmpegPath: "",
  ffprobePath: "",
};

export function loadDialogMemory(): DialogMemory {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return { ...EMPTY };
    const p = JSON.parse(raw) as Partial<DialogMemory>;
    return {
      addDir: typeof p.addDir === "string" ? p.addDir : "",
      outDir: typeof p.outDir === "string" ? p.outDir : "",
      ffmpegPath: typeof p.ffmpegPath === "string" ? p.ffmpegPath : "",
      ffprobePath: typeof p.ffprobePath === "string" ? p.ffprobePath : "",
    };
  } catch {
    return { ...EMPTY };
  }
}

export function saveDialogMemory(partial: Partial<DialogMemory>) {
  const next = { ...loadDialogMemory(), ...partial };
  localStorage.setItem(KEY, JSON.stringify(next));
}

/** 取第一个非空候选；都没有则桌面 */
export async function resolveDialogStart(
  ...candidates: Array<string | null | undefined>
): Promise<string | undefined> {
  for (const c of candidates) {
    const s = typeof c === "string" ? c.trim() : "";
    if (s) return s;
  }
  try {
    return await desktopDir();
  } catch {
    return undefined;
  }
}
