use std::thread::sleep;
use std::time::Duration;
use winapi::shared::minwindef::{LPARAM, WPARAM};
use winapi::um::winuser::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG,
    WM_SETTINGCHANGE, WM_THEMECHANGED,
};
use winreg::enums::*;
use winreg::RegKey;
use napi_derive::napi;

/// Broadcast theme change messages to all windows
fn broadcast_theme_change() {
    let msg = "ImmersiveColorSet";
    let wide: Vec<u16> = msg.encode_utf16().chain([0]).collect();

    unsafe {
        let mut result: usize = 0;
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0 as WPARAM,
            wide.as_ptr() as LPARAM,
            SMTO_ABORTIFHUNG,
            5000,
            &mut result as *mut usize,
        );

        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_THEMECHANGED,
            0 as WPARAM,
            0 as LPARAM,
            SMTO_ABORTIFHUNG,
            5000,
            &mut result as *mut usize,
        );
    }
}

/// Perform full DWM refresh by temporarily flipping the accent color
fn refresh_dwm_full() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Software\\Microsoft\\Windows\\DWM", KEY_READ | KEY_WRITE)
        .expect("Unable to open DWM registry key");

    // Get current accent color
    let original: u32 = key.get_value("ColorizationColor").unwrap_or(0xffd77800);

    // Flip last hex digit to force DWM refresh
    let new_color = if original & 0xF < 0xF {
        original + 1
    } else {
        original - 1
    };

    // Write temporary color
    key.set_value("ColorizationColor", &new_color).unwrap();

    // Broadcast refresh
    broadcast_theme_change();

    sleep(Duration::from_millis(1000));

    // Restore original color
    key.set_value("ColorizationColor", &original).unwrap();

    broadcast_theme_change();
}

/// Toggle Windows light/dark theme
#[napi]
pub fn theme_toggle() -> String {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
    let value_name_apps = "AppsUseLightTheme";
    let value_name_system = "SystemUsesLightTheme";

    let key = hkcu.open_subkey_with_flags(key_path, KEY_READ | KEY_WRITE)
        .expect("Failed to open registry key");

    let current_value_system: u32 = key.get_value(value_name_system).unwrap_or(1);
    let new_theme = if current_value_system == 1 { 0u32 } else { 1u32 };

    // Apply new theme
    key.set_value(value_name_system, &new_theme).unwrap();
    key.set_value(value_name_apps, &new_theme).unwrap();

    // Broadcast & refresh
    broadcast_theme_change();
    refresh_dwm_full();

    if new_theme == 1 { "light".to_string() } else { "dark".to_string() }
}

/// Get current Windows theme
#[napi]
pub fn get_theme() -> String {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
    let value_name_system = "SystemUsesLightTheme";

    let key = hkcu.open_subkey_with_flags(key_path, KEY_READ)
        .expect("Failed to open registry key");

    let current_value: u32 = key.get_value(value_name_system).unwrap_or(1);

    if current_value == 1 { "light".to_string() } else { "dark".to_string() }
}
