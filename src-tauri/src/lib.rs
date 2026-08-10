use arboard::Clipboard;
use clipboard_master::{CallbackResult, ClipboardHandler};
use image::RgbaImage;
use rusqlite::Connection;
use std::io;
use std::sync::Mutex;
use std::thread;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use windows::core::w;
use windows::Win32::System::DataExchange::{
    IsClipboardFormatAvailable, RegisterClipboardFormatW,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn position_window_bottom_right(window: &tauri::WebviewWindow) {
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let screen = monitor.size();
        let scale = monitor.scale_factor();
        let screen_w = screen.width as f64 / scale;
        let screen_h = screen.height as f64 / scale;
        let win_w = 350.0;
        let win_h = 500.0;
        let x = screen_w - win_w - 12.0;
        let y = screen_h - win_h - 56.0;
        let _ = window.set_position(tauri::PhysicalPosition::new(
            (x * scale) as i32,
            (y * scale) as i32,
        ));
    }
}

// ── Helpers ──

fn modifier_from_bitmask(bitmask: i64) -> Option<Modifiers> {
    let mut mods = Modifiers::empty();
    if bitmask & 1 != 0 {
        mods |= Modifiers::ALT;
    }
    if bitmask & 2 != 0 {
        mods |= Modifiers::CONTROL;
    }
    if bitmask & 4 != 0 {
        mods |= Modifiers::SHIFT;
    }
    if mods.is_empty() {
        None
    } else {
        Some(mods)
    }
}

fn key_from_index(index: i64) -> Code {
    match index {
        0 => Code::KeyA,
        1 => Code::KeyB,
        2 => Code::KeyC,
        3 => Code::KeyD,
        4 => Code::KeyE,
        5 => Code::KeyF,
        6 => Code::KeyG,
        7 => Code::KeyH,
        8 => Code::KeyI,
        9 => Code::KeyJ,
        10 => Code::KeyK,
        11 => Code::KeyL,
        12 => Code::KeyM,
        13 => Code::KeyN,
        14 => Code::KeyO,
        15 => Code::KeyP,
        16 => Code::KeyQ,
        17 => Code::KeyR,
        18 => Code::KeyS,
        19 => Code::KeyT,
        20 => Code::KeyU,
        21 => Code::KeyV,
        22 => Code::KeyW,
        23 => Code::KeyX,
        24 => Code::KeyY,
        25 => Code::KeyZ,
        _ => Code::KeyV,
    }
}

// ── State ──

struct AppState {
    last_seen: Mutex<Option<String>>,
    conn: Mutex<Connection>,
}

struct ClipboardListener {
    app_handle: AppHandle,
    sensitive_format_id: u32,
}

impl ClipboardHandler for ClipboardListener {
    fn on_clipboard_change(&mut self) -> CallbackResult {
        // Skip sensitive content (e.g., passwords from password managers)
        if self.sensitive_format_id != 0 {
            let is_sensitive =
                unsafe { IsClipboardFormatAvailable(self.sensitive_format_id).is_ok() };
            if is_sensitive {
                return CallbackResult::Next;
            }
        }

        let Ok(mut clipboard) = Clipboard::new() else {
            return CallbackResult::Next;
        };

        let state = self.app_handle.state::<AppState>();

        if let Ok(image_data) = clipboard.get_image() {
            let image_id = format!("image_{}", image_data.bytes.len());

            let Ok(mut last_seen) = state.last_seen.lock() else {
                return CallbackResult::Next;
            };
            if *last_seen == Some(image_id.clone()) {
                return CallbackResult::Next;
            }
            *last_seen = Some(image_id);
            drop(last_seen);

            let Some(img) = RgbaImage::from_raw(
                image_data.width as u32,
                image_data.height as u32,
                image_data.bytes.into_owned(),
            ) else {
                return CallbackResult::Next;
            };

            let Ok(app_data_dir) = self.app_handle.path().app_data_dir() else {
                return CallbackResult::Next;
            };
            let now_str = chrono::Local::now().format("%Y%m%d_%H%M%S_%3f").to_string();
            let file_name = format!("clip_{}.png", now_str);
            let file_path = app_data_dir.join("images").join(&file_name);

            if img.save(&file_path).is_err() {
                return CallbackResult::Next;
            }

            let file_path_str = file_path.to_string_lossy().to_string();
            let now = chrono::Local::now().to_string();
            if let Ok(conn) = state.conn.lock() {
                let _ = conn.execute(
                    "INSERT INTO clipboard (content, created_at, item_type) VALUES (?1, ?2, 'image')",
                    (&file_path_str, &now),
                );
            }
            let _ = self.app_handle.emit("clipboard-changed", ());
        } else if let Ok(text) = clipboard.get_text() {
            if text.is_empty() {
                return CallbackResult::Next;
            }

            let Ok(mut last_seen) = state.last_seen.lock() else {
                return CallbackResult::Next;
            };
            if *last_seen == Some(text.clone()) {
                return CallbackResult::Next;
            }
            *last_seen = Some(text.clone());
            drop(last_seen);

            let now = chrono::Local::now().to_string();
            if let Ok(conn) = state.conn.lock() {
                let _ = conn.execute(
                    "INSERT INTO clipboard (content, created_at, item_type) VALUES (?1, ?2, 'text')",
                    (&text, &now),
                );
            }
            let _ = self.app_handle.emit("clipboard-changed", ());
        }

        CallbackResult::Next
    }

    fn on_clipboard_error(&mut self, error: io::Error) -> CallbackResult {
        eprintln!("Clipboard error: {}", error);
        CallbackResult::Next
    }
}

// ── Tauri Commands ──

#[tauri::command]
fn get_history(state: State<AppState>) -> Result<Vec<(i64, String, String, bool)>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;

    let limit: i64 = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'history_limit'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(100);

    let mut stmt = conn
        .prepare("SELECT id, content, item_type, pinned FROM clipboard ORDER BY pinned DESC, id DESC LIMIT ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)? != 0,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
fn copy_item(content: String, state: State<AppState>) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard
        .set_text(content.clone())
        .map_err(|e| format!("Copy failed: {}", e))?;
    if let Ok(mut l) = state.last_seen.lock() {
        *l = Some(content);
    }
    Ok(())
}

#[tauri::command]
fn delete_item(id: i64, state: State<AppState>) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;

    // Clean up image file if this entry is an image
    if let Ok(path) = conn.query_row(
        "SELECT content FROM clipboard WHERE id = ?1 AND item_type = 'image'",
        [id],
        |row| row.get::<_, String>(0),
    ) {
        let _ = std::fs::remove_file(&path);
    }

    conn.execute("DELETE FROM clipboard WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn toggle_pin(id: i64, state: State<AppState>) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE clipboard SET pinned = CASE WHEN pinned = 1 THEN 0 ELSE 1 END WHERE id = ?1",
        [id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn clear_all(state: State<AppState>) -> Result<(), String> {
    // Reset duplicate detection
    if let Ok(mut last_seen) = state.last_seen.lock() {
        *last_seen = None;
    }

    let conn = state.conn.lock().map_err(|e| e.to_string())?;

    // Collect non-pinned image paths before deleting
    let paths: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT content FROM clipboard WHERE item_type = 'image' AND pinned = 0")
            .map_err(|e| e.to_string())?;
        let result = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        result
    };

    // Delete only non-pinned entries (pinned items survive clear)
    conn.execute("DELETE FROM clipboard WHERE pinned = 0", [])
        .map_err(|e| e.to_string())?;
    drop(conn);

    // Clean up image files for deleted entries
    for path in paths {
        let _ = std::fs::remove_file(&path);
    }

    Ok(())
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<std::collections::HashMap<String, i64>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut settings = std::collections::HashMap::new();
    for row in rows {
        let (key, value) = row.map_err(|e| e.to_string())?;
        settings.insert(key, value);
    }
    Ok(settings)
}

#[tauri::command]
fn set_setting(key: String, value: i64, state: State<AppState>) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn set_shortcut(
    modifier: i64,
    key: i64,
    state: State<AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mods = modifier_from_bitmask(modifier);
    let code = key_from_index(key);
    let shortcut = Shortcut::new(mods, code);

    // Unregister all existing shortcuts, then register the new one
    app_handle
        .global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;
    app_handle
        .global_shortcut()
        .register(shortcut)
        .map_err(|e| e.to_string())?;

    // Persist to DB
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('shortcut_mod', ?1) ON CONFLICT(key) DO UPDATE SET value = ?1",
        [modifier],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('shortcut_key', ?1) ON CONFLICT(key) DO UPDATE SET value = ?1",
        [key],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_autostart() -> Result<bool, String> {
    let output = std::process::Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            "clippeR",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;
    Ok(output.status.success())
}

#[tauri::command]
fn set_autostart(enabled: bool) -> Result<(), String> {
    if enabled {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_str = exe.to_string_lossy().to_string();
        std::process::Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "clippeR",
                "/t",
                "REG_SZ",
                "/d",
                &exe_str,
                "/f",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
    } else {
        std::process::Command::new("reg")
            .args([
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                "clippeR",
                "/f",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ── App Entry Point ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                position_window_bottom_right(&window);
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            std::fs::create_dir_all(app_data_dir.join("images"))?;

            let db_path = app_data_dir.join("history.db");
            let conn = Connection::open(db_path)?;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS clipboard (
                    id INTEGER PRIMARY KEY,
                    content TEXT NOT NULL,
                    created_at TEXT NOT NULL
                )",
                (),
            )?;

            // Add columns for existing databases (silently ignores if already present)
            let _ = conn.execute(
                "ALTER TABLE clipboard ADD COLUMN item_type TEXT DEFAULT 'text'",
                (),
            );
            let _ = conn.execute(
                "ALTER TABLE clipboard ADD COLUMN pinned INTEGER DEFAULT 0",
                (),
            );

            // Settings table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value INTEGER NOT NULL
                )",
                (),
            )?;

            // Default settings
            let _ = conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('history_limit', 100)",
                (),
            );
            let _ = conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('shortcut_mod', 1)",
                (),
            );
            let _ = conn.execute(
                "INSERT OR IGNORE INTO settings (key, value) VALUES ('shortcut_key', 21)",
                (),
            );

            // Read saved shortcut and register it
            let shortcut_mod: i64 = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = 'shortcut_mod'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(1);
            let shortcut_key: i64 = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = 'shortcut_key'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(21);

            let mods = modifier_from_bitmask(shortcut_mod);
            let code = key_from_index(shortcut_key);
            let shortcut = Shortcut::new(mods, code);
            app.global_shortcut().register(shortcut)?;

            app.manage(AppState {
                last_seen: Mutex::new(None),
                conn: Mutex::new(conn),
            });

            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            position_window_bottom_right(&window);
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            let window = app.get_webview_window("main").expect("main window must exist");
            let window_clone = window.clone();
            window.on_window_event(move |event| match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
                tauri::WindowEvent::Focused(false) => {
                    let _ = window_clone.hide();
                }
                _ => {}
            });

            let app_handle = app.handle().clone();
            let sensitive_format_id = unsafe {
                RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing"))
            };
            thread::spawn(move || {
                let listener = ClipboardListener {
                    app_handle,
                    sensitive_format_id,
                };
                let mut master = clipboard_master::Master::new(listener).unwrap();
                master.run().unwrap();
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_history,
            copy_item,
            delete_item,
            toggle_pin,
            clear_all,
            get_settings,
            set_setting,
            set_shortcut,
            get_autostart,
            set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
