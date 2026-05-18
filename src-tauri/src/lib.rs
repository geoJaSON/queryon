mod commands;
mod db;
mod error;
mod models;
mod secrets;
mod state;
mod store;

use tauri::Manager;

use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "queryon=info,warn".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .expect("could not resolve app config dir");
            let conn = store::open(&config_dir).expect("could not open local store");
            app.manage(AppState::new(conn, config_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connections::list_connections,
            commands::connections::save_connection,
            commands::connections::update_connection,
            commands::connections::delete_connection,
            commands::connections::test_connection,
            commands::connections::open_connection,
            commands::connections::close_connection,
            commands::query::run_sql,
            commands::query::cancel_query,
            commands::introspect::list_databases,
            commands::introspect::list_schemas,
            commands::introspect::list_objects,
            commands::introspect::describe_table,
            commands::introspect::fetch_table_page,
            commands::roles::list_roles,
            commands::roles::create_role,
            commands::roles::alter_role,
            commands::roles::drop_role,
            commands::roles::grant_privilege,
            commands::roles::revoke_privilege,
            commands::schema::create_schema,
            commands::schema::drop_schema,
            commands::history::list_history,
            commands::history::clear_history,
            commands::history::save_query,
            commands::history::list_saved_queries,
            commands::history::delete_saved_query,
            commands::export::export_result,
            commands::export::decode_geometry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
