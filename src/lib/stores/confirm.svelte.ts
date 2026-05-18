// Promise-based confirm dialog for destructive actions. The dialog shows the
// exact SQL/effect so the user sees what will run.

interface ConfirmReq {
  title: string;
  body: string;
  /** Exact statement that will be executed, shown verbatim. */
  sql?: string;
  danger: boolean;
  resolve: (ok: boolean) => void;
}

export const confirmState = $state<{ req: ConfirmReq | null }>({ req: null });

export function confirmAction(opts: {
  title: string;
  body: string;
  sql?: string;
  danger?: boolean;
}): Promise<boolean> {
  return new Promise((resolve) => {
    confirmState.req = {
      title: opts.title,
      body: opts.body,
      sql: opts.sql,
      danger: opts.danger ?? true,
      resolve,
    };
  });
}

export function answerConfirm(ok: boolean) {
  const r = confirmState.req;
  confirmState.req = null;
  r?.resolve(ok);
}
