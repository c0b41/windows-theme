use std::ptr::null_mut;
use winapi::shared::minwindef::LPARAM;
use winapi::um::winuser::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};
use winreg::{enums::*, RegKey};
use napi_derive::napi;

#[napi]
pub fn theme_toggle() -> String {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
    let value_name_apps = "AppsUseLightTheme";
    let value_name_system = "SystemUsesLightTheme";
    let value_name_accent = "ColorPrevalence";
    let backup_value_name = "ColorPrevalenceBackup";

    let key = hkcu.open_subkey_with_flags(key_path, KEY_READ | KEY_WRITE)
        .unwrap_or_else(|_| panic!("Failed to open registry key"));

    let current_value_system: u32 = key.get_value(value_name_system)
        .unwrap_or(1);
    let mut current_value_apps: u32 = key.get_value(value_name_apps)
        .unwrap_or(1);
    let current_value_accent: u32 = key.get_value(value_name_accent)
        .unwrap_or(1);

    if current_value_apps != current_value_system {
        current_value_apps = current_value_system;
        key.set_value(value_name_apps, &current_value_apps)
            .unwrap();
    }

    let new_theme = if current_value_system == 1 { 0u32 } else { 1u32 };

    if new_theme == 1 {
        key.set_value(backup_value_name, &current_value_accent)
            .unwrap();
        if current_value_accent == 1 {
            key.set_value(value_name_accent, &0u32)
                .unwrap();
        }
    } else {
        if let Ok(stored_accent_value) = key.get_value::<u32, _>(backup_value_name) {
            key.set_value(value_name_accent, &stored_accent_value)
                .unwrap();
        }
    }

    key.set_value(value_name_system, &new_theme)
        .unwrap();
    key.set_value(value_name_apps, &new_theme)
        .unwrap();

    // Broadcast theme change
    let message = "ImmersiveColorSet";
    let wide_message: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            wide_message.as_ptr() as LPARAM,
            SMTO_ABORTIFHUNG,
            50,
            null_mut(),
        );
    }

    if new_theme == 1 {
        "light".to_string()
    } else {
        "dark".to_string()
    }
}

#[napi]
pub fn get_theme() -> String {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key_path = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
    let value_name_system = "SystemUsesLightTheme";

    let key = hkcu.open_subkey_with_flags(key_path, KEY_READ)
        .unwrap_or_else(|_| panic!("Failed to open registry key"));

    let current_value: u32 = key.get_value(value_name_system)
        .unwrap_or(1);

    if current_value == 1 {
        "light".to_string()
    } else {
        "dark".to_string()
    }
}