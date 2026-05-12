#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tauri::Manager;
use tauri_plugin_updater::Builder as UpdaterBuilder;

mod commands;
mod epub_protocol;
mod spellcheck;

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        let path = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("xcalibre")
            .join("crash.log");
        let _ = std::fs::write(&path, &msg);
        eprintln!("panic: {}", msg);
    }));

    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path().app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("jobs.db");
            let pool = tauri::async_runtime::block_on(async {
                let pool: SqlitePool = SqlitePoolOptions::new()
                    .connect_with(
                        SqliteConnectOptions::new()
                            .filename(&db_path)
                            .create_if_missing(true),
                    )
                    .await
                    .expect("failed to open DB");
                sqlx::migrate!("../processing/src/db/migrations")
                    .run(&pool)
                    .await
                    .expect("failed to run migrations");
                pool
            });
            app.manage(Arc::new(pool));
            app.manage(commands::EditorSessions::default());
            Ok(())
        })
        .register_uri_scheme_protocol("epub", |ctx, req| {
            epub_protocol::epub_handler(ctx.app_handle(), req)
        })
        .plugin(UpdaterBuilder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::list_books,
            commands::get_library,
            commands::get_book_details,
            commands::filter_library,
            commands::search_library,
            commands::search_books,
            commands::filter_books,
            commands::ingest_file,
            commands::get_spine,
            commands::get_epub_chapter_html,
            commands::update_position,
            commands::update_progress,
            commands::add_bookmark,
            commands::list_bookmarks,
            commands::delete_bookmark,
            commands::create_collection_cmd,
            commands::list_collections_cmd,
            commands::add_book_to_collection_cmd,
            commands::remove_book_from_collection_cmd,
            commands::get_collection_books_cmd,
            commands::bulk_reingest,
            commands::bulk_delete,
            commands::export_metadata,
            commands::write_file,
            commands::import_calibre,
            commands::list_tags,
            commands::list_series,
            commands::list_authors,
            commands::create_collection,
            commands::list_collections,
            commands::delete_collection,
            commands::add_book_to_collection,
            commands::remove_book_from_collection,
            commands::get_books_in_collection,
            commands::bulk_delete_books,
            commands::bulk_reingest_books,
            commands::bulk_export_metadata,
            commands::update_book_details,
            commands::repair_books,
            commands::convert_book_to_epub,
            commands::check_library_integrity,
            commands::export_library_csv,
            commands::export_library_html,
            commands::create_annotation,
            commands::get_annotations,
            commands::delete_annotation,
            commands::update_annotation_note,
            commands::open_in_os,
            commands::list_comic_pages,
            commands::sync_annotations,
            commands::save_config,
            commands::get_xs_url,
            commands::has_token,
            commands::list_libraries_cmd,
            commands::create_library_cmd,
            commands::set_active_library_cmd,
            commands::get_active_library_cmd,
            commands::get_app_data_dir,
            commands::delete_library_cmd,
            commands::search_library_advanced,
            commands::list_plugins_cmd,
            commands::install_plugin_from_zip,
            commands::set_plugin_enabled_cmd,
            commands::uninstall_plugin_cmd,
            commands::update_book_metadata,
            commands::check_word,
            commands::get_suggestions,
            commands::add_to_dictionary,
            commands::set_spell_check_language,
            commands::get_ai_config_cmd,
            commands::save_ai_config_cmd,
            commands::get_ai_context_chunks,
            commands::ai_chat,
            commands::save_ai_response_as_note_cmd,
            commands::convert_book,
            commands::list_virtual_libraries_cmd,
            commands::create_virtual_library_cmd,
            commands::update_virtual_library_cmd,
            commands::delete_virtual_library_cmd,
            commands::run_virtual_library_cmd,
            commands::list_notes_cmd,
            commands::get_note_cmd,
            commands::create_note_cmd,
            commands::update_note_cmd,
            commands::delete_note_cmd,
            commands::search_notes_cmd,
            commands::find_similar_books_cmd,
            commands::editor_open_epub,
            commands::editor_read_item,
            commands::editor_write_item,
            commands::editor_save_epub,
            commands::editor_update_metadata,
            commands::editor_set_cover,
            commands::editor_close,
            commands::ai_list_models,
            commands::export_library_backup_cmd,
            commands::restore_library_backup_cmd,
            commands::copy_book_to_library_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}
