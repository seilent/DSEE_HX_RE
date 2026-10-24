import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

function IconMinimize() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
      <path d="M1 5h8" stroke="currentColor" strokeWidth="1" fill="none" />
    </svg>
  );
}

function IconMaximize() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
      <rect
        x="1.5"
        y="1.5"
        width="7"
        height="7"
        stroke="currentColor"
        strokeWidth="1"
        fill="none"
      />
    </svg>
  );
}

function IconRestore() {
  // 后框右上（只露顶边/右边），前框左下完整方框
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
      <path
        d="M3 1.5h5.5v5.5"
        stroke="currentColor"
        strokeWidth="1"
        fill="none"
      />
      <rect
        x="1"
        y="3.5"
        width="5.5"
        height="5.5"
        stroke="currentColor"
        strokeWidth="1"
        fill="none"
      />
    </svg>
  );
}

function IconClose() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden>
      <path
        d="M1.5 1.5l7 7M8.5 1.5l-7 7"
        stroke="currentColor"
        strokeWidth="1.1"
        fill="none"
      />
    </svg>
  );
}

type Props = {
  /** 小窗可不显示最大化 */
  showMaximize?: boolean;
};

export default function WindowControls({ showMaximize = true }: Props) {
  const win = getCurrentWindow();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    if (!showMaximize) return;
    let unlisten: (() => void) | undefined;
    const sync = (m: boolean) => {
      setMaximized(m);
      document.documentElement.classList.toggle("is-maximized", m);
    };
    void win.isMaximized().then(sync);
    void win
      .onResized(() => {
        void win.isMaximized().then(sync);
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => {
      unlisten?.();
      document.documentElement.classList.remove("is-maximized");
    };
  }, [win, showMaximize]);

  return (
    <div className="window-controls">
      <button
        type="button"
        className="win-btn"
        aria-label="Minimize"
        onClick={() => void win.minimize()}
      >
        <IconMinimize />
      </button>
      {showMaximize ? (
        <button
          type="button"
          className="win-btn"
          aria-label={maximized ? "Restore" : "Maximize"}
          onClick={() => void win.toggleMaximize()}
        >
          {maximized ? <IconRestore /> : <IconMaximize />}
        </button>
      ) : null}
      <button
        type="button"
        className="win-btn win-btn-close"
        aria-label="Close"
        onClick={() => void win.close()}
      >
        <IconClose />
      </button>
    </div>
  );
}
