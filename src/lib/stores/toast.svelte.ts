// Transient notification surface (errors, confirmations).

export type ToastKind = "error" | "info" | "ok";

export interface Toast {
  id: number;
  kind: ToastKind;
  text: string;
  detail?: string;
}

let seq = 0;

export const toasts = $state<{ items: Toast[] }>({ items: [] });

export function pushToast(kind: ToastKind, text: string, detail?: string) {
  const id = ++seq;
  toasts.items.push({ id, kind, text, detail });
  if (kind !== "error") {
    setTimeout(() => dismissToast(id), 4000);
  }
}

export function dismissToast(id: number) {
  const i = toasts.items.findIndex((t) => t.id === id);
  if (i >= 0) toasts.items.splice(i, 1);
}

/** Normalize a thrown backend AppError (or anything) into a toast. */
export function reportError(e: unknown, fallback = "Operation failed") {
  if (e && typeof e === "object" && "message" in e) {
    const err = e as { message: string; detail?: string | null };
    pushToast("error", err.message || fallback, err.detail ?? undefined);
  } else {
    pushToast("error", fallback, String(e));
  }
}
