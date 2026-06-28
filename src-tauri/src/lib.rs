use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri_plugin_autostart::{AutoLaunchManager, MacosLauncher};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

#[derive(Serialize, Deserialize, Default)]
struct Config {
    api_key: String,
}

fn config_path() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("GrammarFix");
    fs::create_dir_all(&dir).ok();
    dir.join("config.json")
}

fn load_config() -> Config {
    let path = config_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(config: &Config) -> Result<(), String> {
    let path = config_path();
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_api_key() -> Result<String, String> {
    Ok(load_config().api_key)
}

#[tauri::command]
async fn save_api_key(key: String) -> Result<(), String> {
    let mut config = load_config();
    config.api_key = key;
    save_config(&config)
}

#[tauri::command]
async fn get_autostart(app: tauri::AppHandle) -> Result<bool, String> {
    let manager = app.state::<AutoLaunchManager>();
    manager.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.state::<AutoLaunchManager>();
    if enabled {
        manager.enable().map_err(|e| e.to_string())
    } else {
        manager.disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
async fn fix_grammar(text: String) -> Result<String, String> {
    let config = load_config();
    let api_key = config.api_key;

    if api_key.is_empty() {
        return Err("No API key set. Click the settings icon to add one.".into());
    }

    let url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-flash-lite:generateContent";

    let body = serde_json::json!({
        "systemInstruction": {
            "parts": [{
                "text": "You are a grammar correction tool. Fix grammatical errors, spelling mistakes, and punctuation in the given text. Return ONLY the corrected text. Do not explain, do not add commentary, do not rephrase for style — only fix actual errors. If the text has no errors, return it unchanged."
            }]
        },
        "contents": [{
            "role": "user",
            "parts": [{ "text": text }]
        }],
        "generationConfig": {
            "temperature": 0.2,
            "maxOutputTokens": 4096
        }
    });

    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("x-goog-api-key", &api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = response.status();
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !status.is_success() {
        let msg = data["error"]["message"]
            .as_str()
            .unwrap_or("Unknown API error");
        return Err(format!("API error ({}): {}", status, msg));
    }

    let result = data["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();

    if result.is_empty() {
        return Err("Empty response from Gemini. The text may have been filtered.".into());
    }

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // ── System Tray ──
            let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let close_item = MenuItemBuilder::with_id("close", "Close").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&close_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                    "close" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── Global Shortcut: Alt+P ──
            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyP);
            let _ = app.global_shortcut().on_shortcut(
                shortcut,
                |app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                },
            );

            // ── Minimize to tray on close ──
            let window = app.get_webview_window("main").unwrap();
            let win_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win_clone.hide();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            fix_grammar,
            save_api_key,
            get_api_key,
            get_autostart,
            set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("error while running GrammarFix");
}
