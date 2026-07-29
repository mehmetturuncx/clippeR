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

struct AppState {
    last_seen: Mutex<Option<String>>,
    conn: Mutex<Connection>,
}

struct ClipboardListener {
    app_handle: AppHandle,
}

impl ClipboardHandler for ClipboardListener {
    fn on_clipboard_change(&mut self) -> CallbackResult {
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
fn get_history(state: State<AppState>) -> Result<Vec<(i64, String, String)>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, content, item_type FROM clipboard ORDER BY id DESC LIMIT 100")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
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
fn clear_all(state: State<AppState>, app_handle: AppHandle) -> Result<(), String> {
    // Reset duplicate detection
    if let Ok(mut last_seen) = state.last_seen.lock() {
        *last_seen = None;
    }

    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM clipboard", [])
        .map_err(|e| e.to_string())?;
    drop(conn);

    // Clean up all saved image files
    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        let images_dir = app_data_dir.join("images");
        if let Ok(entries) = std::fs::read_dir(&images_dir) {
            for entry in entries.flatten() {
                let _ = std::fs::remove_file(entry.path());
            }
        }
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
            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyV);
            app.global_shortcut().register(shortcut)?;

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

            // Add item_type column for existing databases (silently ignores if already present)
            let _ = conn.execute(
                "ALTER TABLE clipboard ADD COLUMN item_type TEXT DEFAULT 'text'",
                (),
            );

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
            thread::spawn(move || {
                let listener = ClipboardListener { app_handle };
                let mut master = clipboard_master::Master::new(listener).unwrap();
                master.run().unwrap();
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_history,
            copy_item,
            delete_item,
            clear_all
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
