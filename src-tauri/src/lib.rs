mod backup;
#[cfg(feature = "desktop")]
mod commands;
pub mod domain;
mod issue_repository;
mod issue_service;
mod productivity;
pub mod repository;
pub mod service;
#[cfg(feature = "desktop")]
pub fn run() {
    use tauri::Manager;
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&directory)?;
            let service = service::Service::open(&directory.join("forma.sqlite3"))
                .map_err(|error| std::io::Error::other(error.message))?;
            app.manage(std::sync::Mutex::new(service));
            Ok(())
        })
        .invoke_handler(commands::handler())
        .run(tauri::generate_context!())
        .expect("Unable to start Forma");
}
#[cfg(test)]
mod issue_tests;
#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "ipc-tests"))]
mod ipc_tests;

mod git_engine;
mod git_repository;
mod git_service;
#[cfg(test)]
mod git_tests;
