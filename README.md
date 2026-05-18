# QUERYON

A retro-terminal (phosphor-on-black) desktop client for managing your
PostGIS / PostgreSQL databases. Tauri v2 + Svelte 5 + Rust.

## Features

- **Connections** — multiple saved profiles; passwords stored in Windows
  Credential Manager (never on disk, never sent to the UI). SSL modes
  disable / prefer / require.
- **SQL editor** — CodeMirror with PostgreSQL syntax; run all / run selection
  (Ctrl+Enter), multi-statement results, timing, errors with SQLSTATE,
  cancel a running query.
- **Schema browser** — lazy tree of schemas → tables / views / matviews /
  functions, with a column / index inspector.
- **Table view** — paginated browsing with sort and a simple filter builder;
  geometry columns rendered as EWKT (GeoJSON toggle).
- **Roles** — list / create / alter / drop roles and grant / revoke
  privileges; destructive actions show the exact SQL first.
- **Schema ops** — create / drop schema (CASCADE confirmed).
- **History & saved queries** — auto-recorded per connection, searchable;
  name and re-open saved queries.
- **Export** — query/table results to CSV or JSON.
- **PostGIS** — geometry/geography shown as text; one-click EWKT decode for
  free-form query results.

## Develop

```sh
pnpm install
pnpm tauri dev      # run the app
```

## Build a Windows installer

```sh
pnpm tauri build    # produces MSI + NSIS in src-tauri/target/release/bundle
```

(First run of an unsigned installer triggers SmartScreen — "More info →
Run anyway", or sign the binary for distribution.)

## Layout

- `src/` — Svelte 5 frontend. `src/lib/ipc.ts` is the only file that calls
  the backend; `src/lib/types.ts` mirrors the Rust DTOs.
- `src-tauri/src/` — Rust backend: `db/` (Postgres access, generic value
  stringifier, introspection), `store/` (SQLite: connections, history, saved
  queries), `secrets.rs` (keychain), `commands/` (IPC surface).

Local store + config: `%APPDATA%\com.queryon.app\queryon.db`.
