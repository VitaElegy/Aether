#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::Manager;

fn main() {
  tauri::Builder::default()
    .setup(|app| {
        let window = app.get_window("main").unwrap();
        
        // Spawn the backend sidecar
        // The binary name in 'new_sidecar' must match 'externalBin' in tauri.conf.json WITHOUT extension/target-triple
        let (mut receiver, _child) = tauri::api::process::Command::new_sidecar("aether_backend")
            .expect("failed to create `aether_backend` binary command")
            .spawn()
            .expect("Failed to spawn sidecar");

        tauri::async_runtime::spawn(async move {
            while let Some(event) = receiver.recv().await {
                 if let tauri::api::process::CommandEvent::Stdout(line) = event {
                      println!("[Backend] {}", line);
                      window.emit("backend-stdout", line).unwrap();
                 } else if let tauri::api::process::CommandEvent::Stderr(line) = event {
                      eprintln!("[Backend Error] {}", line);
                 }
            }
        });
        
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
