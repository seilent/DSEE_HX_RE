import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import { createPortal } from "react-dom";

export type RoundSelectOption = { value: string; label: string };

type Props = {
  value: string;
  options: RoundSelectOption[];
  disabled?: boolean;
  className?: string;
  title?: string;
  "aria-label"?: string;
  onChange?: (value: string) => void;
};

export default function RoundSelect({
  value,
  options,
  disabled,
  className,
  title,
  "aria-label": ariaLabel,
  onChange,
}: Props) {
  const [open, setOpen] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const menuRef = useRef<HTMLUListElement>(null);
  const [pos, setPos] = useState({ top: 0, left: 0, width: 0 });

  const selected =
    options.find((o) => o.value === value) ?? options[0] ?? null;
  const blank =
    Boolean(className?.includes("is-blank")) ||
    Boolean(className?.includes("row-blank"));

  function measureMenu(): { top: number; left: number; width: number } | null {
    const btn = rootRef.current?.querySelector<HTMLElement>(
      ".round-select-trigger",
    );
    if (!btn) return null;
    const r = btn.getBoundingClientRect();
    return { top: r.bottom + 4, left: r.left, width: Math.max(r.width, 72) };
  }

  function placeMenu() {
    const next = measureMenu();
    if (next) setPos(next);
  }

  useEffect(() => {
    if (!open) return;
    placeMenu();
    const onDown = (e: MouseEvent) => {
      const t = e.target as Node;
      if (rootRef.current?.contains(t) || menuRef.current?.contains(t)) return;
      setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    const onReposition = () => placeMenu();
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    window.addEventListener("resize", onReposition);
    document.addEventListener("scroll", onReposition, true);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
      window.removeEventListener("resize", onReposition);
      document.removeEventListener("scroll", onReposition, true);
    };
  }, [open]);

  // Hide until measured — avoids one-frame flash at (0,0) on first open.
  const menuStyle: CSSProperties = {
    top: pos.top,
    left: pos.left,
    minWidth: pos.width,
    visibility: pos.width > 0 ? "visible" : "hidden",
  };

  return (
    <div
      ref={rootRef}
      className={["round-select", className].filter(Boolean).join(" ")}
    >
      <button
        type="button"
        className="round-select-trigger"
        disabled={disabled}
        title={title}
        aria-label={ariaLabel}
        aria-expanded={open}
        aria-haspopup="listbox"
        onClick={() => {
          if (disabled) return;
          if (open) {
            setOpen(false);
            return;
          }
          // Measure in the same click handler so React batches pos+open
          // and the portal never paints at the default (0,0).
          const next = measureMenu();
          if (next) setPos(next);
          setOpen(true);
        }}
      >
        <span className={blank ? "round-select-label is-blank" : "round-select-label"}>
          {selected?.label ?? ""}
        </span>
        <span className="round-select-chevron" aria-hidden>
          ▾
        </span>
      </button>
      {open &&
        createPortal(
          <ul
            ref={menuRef}
            className="round-select-menu"
            role="listbox"
            style={menuStyle}
          >
            {options.map((o) => {
              const active = o.value === value;
              return (
                <li key={o.value} role="presentation">
                  <button
                    type="button"
                    role="option"
                    aria-selected={active}
                    className={`round-select-option${active ? " is-active" : ""}`}
                    onClick={() => {
                      onChange?.(o.value);
                      setOpen(false);
                    }}
                  >
                    {o.label}
                  </button>
                </li>
              );
            })}
          </ul>,
          document.body,
        )}
    </div>
  );
}
