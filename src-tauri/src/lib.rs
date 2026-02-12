mod git_commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            git_commands::open_repository,
            git_commands::get_commits,
            git_commands::get_commits_filtered,
            git_commands::get_commit_detail,
            git_commands::update_commit,
            git_commands::check_backup,
            git_commands::restore_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
