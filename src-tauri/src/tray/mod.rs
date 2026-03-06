use tauri::{Manager, Emitter, tray::{TrayIconBuilder, TrayIcon, TrayIconEvent}, menu::{Menu, MenuItem, PredefinedMenuItem}, image::Image};
use tracing::{info, error, warn};
use std::fs;
use std::env;

pub mod notification_handler;

/// Creates and configures the system tray builder
/// Tries to load a custom icon from icons/tray-icon.png
pub fn create_system_tray_builder(app_handle: &tauri::AppHandle) -> TrayIconBuilder<tauri::Wry> {
    let mut builder = TrayIconBuilder::new().tooltip("Device App");

    let icon_path = "icons/tray-icon.png";
    
    // Collect all possible paths to try
    let mut paths_to_try = Vec::new();
    
    // 1. Resource directory (production)
    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        paths_to_try.push((resource_dir.join(icon_path), "resource_dir"));
    }
    
    // 2. Current working directory (development)
    // Try direct icons/ path first (if we're in src-tauri/)
    if let Ok(cwd) = env::current_dir() {
        // If we're in src-tauri/, try icons/ directly
        if cwd.ends_with("src-tauri") {
            let dev_path = cwd.join(icon_path);
            paths_to_try.push((dev_path, "dev_cwd_direct"));
        }
        // Also try going up one level and then src-tauri/icons/
        if let Some(parent) = cwd.parent() {
            let dev_path = parent.join("src-tauri").join(icon_path);
            paths_to_try.push((dev_path, "dev_cwd_parent"));
        }
    }
    
    // 3. Executable directory + src-tauri/icons/ (alternative dev path)
    if let Ok(exe_dir) = env::current_exe() {
        if let Some(exe_parent) = exe_dir.parent() {
            let exe_path = exe_parent.join("src-tauri").join(icon_path);
            paths_to_try.push((exe_path, "exe_dir"));
        }
    }
    
    // Try all paths
    for (path, source) in paths_to_try {
        if path.exists() {
            info!(path = %path.display(), source = %source, "Found tray icon file");
            match load_tray_icon(&path) {
                Ok(image) => {
                    builder = builder.icon(image);
                    info!(icon_path = %path.display(), source = %source, "Loaded custom tray icon successfully");
                    return builder;
                }
                Err(e) => {
                    warn!(error = %e, icon_path = %path.display(), source = %source, "Failed to load tray icon");
                }
            }
        }
    }
    
    warn!("No custom tray icon found at icons/tray-icon.png, using default app icon");
    // If no custom icon found, use default (no icon set - Tauri will use app icon)
    builder
}

/// Loads and decodes a PNG image file for use as tray icon
fn load_tray_icon(path: &std::path::Path) -> Result<Image<'static>, String> {
    // Read file bytes
    let bytes = fs::read(path)
        .map_err(|e| format!("Failed to read icon file: {}", e))?;
    
    info!(file_size = bytes.len(), "Reading icon file");
    
    // Decode PNG using image crate
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Failed to decode PNG: {}", e))?;
    
    // Convert to RGBA and resize to 32x32 if needed (optimal size for tray icons)
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    
    info!(original_size = format!("{}x{}", width, height), "Decoded PNG image");
    
    // Resize if larger than 32x32 (tray icons should be small)
    let final_img = if width > 32 || height > 32 {
        info!("Resizing icon to 32x32");
        image::imageops::resize(&rgba_img, 32, 32, image::imageops::FilterType::Lanczos3)
    } else {
        rgba_img
    };
    
    let (final_width, final_height) = final_img.dimensions();
    
    // Convert to Vec<u8> (RGBA format, row-major order)
    let rgba_data = final_img.into_raw();
    
    info!(
        final_size = format!("{}x{}", final_width, final_height),
        rgba_bytes = rgba_data.len(),
        "Creating Tauri Image"
    );
    
    // Create Tauri Image
    let tauri_image = Image::new_owned(rgba_data, final_width, final_height);
    
    Ok(tauri_image)
}

/// Creates the system tray menu
pub fn create_system_tray_menu(app_handle: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let menu = Menu::new(app_handle)?;
    
    let show_item = MenuItem::with_id(app_handle, "show", "Show", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app_handle, "hide", "Hide", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app_handle)?;
    let quit_item = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)?;
    
    menu.append(&show_item)?;
    menu.append(&hide_item)?;
    menu.append(&separator)?;
    menu.append(&quit_item)?;
    
    Ok(menu)
}

/// Handles system tray icon events
pub fn handle_tray_icon_event(tray_icon: &TrayIcon<tauri::Wry>, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click { .. } => {
            info!("System tray click");
            // Get app handle from tray icon
            let app = tray_icon.app_handle();
            // Toggle window visibility
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
        _ => {}
    }
}

/// Handles menu events from system tray
pub fn handle_menu_event(app: &tauri::AppHandle, event: tauri::menu::MenuEvent) {
    let id = event.id.as_ref();
    info!(menu_id = %id, "System tray menu item clicked");
    
    match id {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "hide" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        "quit" => {
            info!("Quit requested — notifying frontend to clear session before exit");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                if let Err(e) = window.emit("app-before-quit", ()) {
                    error!(error = %e, "Failed to emit app-before-quit event");
                    app.exit(0);
                }
            } else {
                app.exit(0);
            }
        }
        _ => {
            error!(menu_id = %id, "Unknown menu item");
        }
    }
}
