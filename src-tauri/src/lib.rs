use clipboard_win::{get_clipboard_string, set_clipboard_string};
use rusqlite::Connection;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, State};
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
    history: Mutex<Vec<(i64, String)>>,
    last_seen: Mutex<Option<String>>,
    conn: Mutex<Connection>,
}

#[tauri::command]
fn get_history(state: State<AppState>) -> Vec<(i64, String)> {
    let h = state.history.lock().unwrap();
    let mut result: Vec<(i64, String)> = h.clone();
    result.reverse();
    result
}

#[tauri::command]
fn copy_item(content: String, state: State<AppState>) -> Result<(), String> {
    match set_clipboard_string(&content) {
        Ok(_) => {
            let mut l = state.last_seen.lock().unwrap();
            *l = Some(content);
            Ok(())
        }
        Err(e) => Err(format!("Copy failed: {}", e)),
    }
}

#[tauri::command]
fn delete_item(id: i64, state: State<AppState>) -> Result<(), String> {
    let mut h = state.history.lock().unwrap();
    if let Some(pos) = h.iter().position(|(item_id, _)| *item_id == id) {
        h.remove(pos);
    }
    state
        .conn
        .lock()
        .unwrap()
        .execute("DELETE FROM clipboard WHERE id = ?1", [id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

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
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
       	let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::KeyV);
        app.global_shortcut().register(shortcut)?;
            let app_data_dir = app.path().app_data_dir().unwrap();
            std::fs::create_dir_all(&app_data_dir).unwrap();
            let db_path = app_data_dir.join("history.db");
            let conn = Connection::open(db_path).unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS clipboard (
                    id INTEGER PRIMARY KEY,
                    content TEXT NOT NULL,
                    created_at TEXT NOT NULL
                )",
                (),
            )
            .unwrap();

            let mut temp_vec: Vec<(i64, String)> = Vec::new();
            {
                let mut stmt = conn
                    .prepare("SELECT id, content FROM clipboard ORDER BY id DESC LIMIT 100")
                    .unwrap();
                let rows = stmt
                    .query_map([], |row| {
                        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
                    })
                    .unwrap();
                for row in rows {
                    temp_vec.push(row.unwrap());
                }
            }
            temp_vec.reverse();

            app.manage(AppState {
                history: Mutex::new(temp_vec),
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
                        rect,
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

            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                    tauri::WindowEvent::Focused(false) => {
                        let _ = window_clone.hide();
                    }
                    _ => {}
                }
            });

            let app_handle = app.handle().clone();
            thread::spawn(move || loop {
                if let Ok(text) = get_clipboard_string() {
                    let state = app_handle.state::<AppState>();
                    let mut last_seen = state.last_seen.lock().unwrap();
                    if *last_seen != Some(text.clone()) && !text.is_empty() {
                        *last_seen = Some(text.clone());
                        drop(last_seen);

                        let mut h = state.history.lock().unwrap();
                        let now = chrono::Local::now().to_string();
                        let conn = state.conn.lock().unwrap();
                        conn.execute(
                            "INSERT INTO clipboard (content, created_at) VALUES (?1, ?2)",
                            (&text, &now),
                        )
                        .unwrap();
                        let new_id = conn.last_insert_rowid();
                        drop(conn);
                        h.push((new_id, text));
                        if h.len() > 100 {
                            h.remove(0);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(500));
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_history,
            copy_item,
            delete_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
