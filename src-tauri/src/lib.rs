mod git_commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            git_commands::open_repository,
            git_commands::get_commits,
            git_commands::get_commit_detail,
            git_commands::update_commit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
