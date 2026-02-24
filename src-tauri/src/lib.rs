mod auth;
mod handler;
use auth::official::add_new_account;
use auth::official::check_account_time;
use auth::official::download_player_skin;
use auth::littleskin::useMethod;
use auth::yggdrasil::thirdPartyLogin;
use handler::authSearcher::get_all_accounts;
use handler::authSearcher::get_all_littleskin_users;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![add_new_account,check_account_time,download_player_skin,useMethod,thirdPartyLogin,get_all_accounts,get_all_littleskin_users])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
