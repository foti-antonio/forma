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
    if agent_provider::watchdog_entry() {
        return;
    }
    use tauri::Manager;
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&directory)?;
            let instance = std::fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(directory.join("forma-instance.lock"))?;
            instance.try_lock().map_err(|_| {
                std::io::Error::other("Forma is already running with this app-data directory.")
            })?;
            app.manage(instance);
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

mod agent_provider;
mod agent_repository;
mod agent_service;
pub mod agent_types;
mod execution_service;
mod test_execution;
mod worktree_service;

#[cfg(test)]
mod agent_tests;

#[cfg(test)]
mod execution_tests;

mod onboarding;

#[cfg(test)]
mod onboarding_tests;
