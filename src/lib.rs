use napi::{bindgen_prelude::*, Error, Status};
use napi_derive::napi;
use std::ptr::null_mut;
use tokio::runtime::Runtime;
use winapi::shared::minwindef::{LPARAM, WPARAM};
use winapi::um::winuser::{
    SendMessageTimeoutW, HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG,
    SystemParametersInfoW, SPI_SETCLIENTAREAANIMATION, SPIF_UPDATEINIFILE, SPIF_SENDCHANGE,
};
use winreg::enums::*;
use winreg::RegKey;

// Define an enum for the theme
#[napi]
#[derive(Debug)]
pub enum Theme {
    Light,
    Dark,
}

impl From<u32> for Theme {
    fn from(value: u32) -> Self {
        match value {
            1 => Theme::Light,
            0 => Theme::Dark,
            _ => panic!("Invalid theme value"),
        }
    }
}

impl From<String> for Theme {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => panic!("Invalid theme value"),
        }
    }
}

impl From<Theme> for String {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => "light".to_string(),
            Theme::Dark => "dark".to_string(),
        }
    }
}

#[napi]
fn get_theme(_env: Env) -> AsyncTask<GetThemeTask> {
    AsyncTask::new(GetThemeTask)
}

struct GetThemeTask;

#[async_trait::async_trait(?Send)]
impl Task for GetThemeTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> Result<Self::Output> {
        // Use a Tokio runtime to run the async task
        let runtime = Runtime::new().unwrap();
        runtime.block_on(async {
            tokio::task::spawn_blocking(|| {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let key_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize";
                // Open the registry key
                let key = hkcu.open_subkey_with_flags(key_path, KEY_READ).map_err(|_| {
                    Error::new(
                        Status::GenericFailure,
                        "Failed to open registry key".to_string(),
                    )
                })?;
                // Query the current value of AppsUseLightTheme
                let apps_use_light_theme: u32 = key.get_value("AppsUseLightTheme").map_err(|_| {
                    Error::new(
                        Status::GenericFailure,
                        "Failed to read AppsUseLightTheme".to_string(),
                    )
                })?;
                // Convert the numeric value to the Theme enum and then to a string
                let theme: Theme = apps_use_light_theme.into();
                Ok::<String, Error>(theme.into())
            })
            .await
            .map_err(|_| {
                Error::new(
                    Status::GenericFailure,
                    "Failed to execute async task".to_string(),
                )
            })?
        })
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[napi]
fn set_theme(_env: Env, theme: String) -> AsyncTask<SetThemeTask> {
    AsyncTask::new(SetThemeTask { theme })
}

struct SetThemeTask {
    theme: String,
}

#[async_trait::async_trait(?Send)]
impl Task for SetThemeTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
        // Use a Tokio runtime to run the async task
        let runtime = Runtime::new().unwrap();
        runtime.block_on(async {
            let theme = self.theme.clone();
            tokio::task::spawn_blocking(move || {
                // Convert the input string to the Theme enum
                let theme: Theme = theme.into();
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let key_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize";
                // Map the enum to the corresponding registry value
                let new_value: u32 = match theme {
                    Theme::Light => 1,
                    Theme::Dark => 0,
                };
                // Open the registry key
                let key = hkcu.open_subkey_with_flags(key_path, KEY_READ | KEY_SET_VALUE).map_err(|_| {
                    Error::new(
                        Status::GenericFailure,
                        "Failed to open registry key".to_string(),
                    )
                })?;
                // Update AppsUseLightTheme and SystemUsesLightTheme
                key.set_value("AppsUseLightTheme", &new_value).map_err(|_| {
                    Error::new(
                        Status::GenericFailure,
                        "Failed to update AppsUseLightTheme".to_string(),
                    )
                })?;
                key.set_value("SystemUsesLightTheme", &new_value).map_err(|_| {
                    Error::new(
                        Status::GenericFailure,
                        "Failed to update SystemUsesLightTheme".to_string(),
                    )
                })?;
                // Notify the system of the change using SystemParametersInfoW
                unsafe {
                    SystemParametersInfoW(
                        SPI_SETCLIENTAREAANIMATION, // Action: Set client area animation
                        0,                          // No additional parameter
                        null_mut(),                 // No additional data
                        SPIF_UPDATEINIFILE | SPIF_SENDCHANGE, // Flags: Update INI file and send change notification
                    );
                }
                // Broadcast WM_SETTINGCHANGE to all top-level windows
                unsafe {
                    let setting_change_message = "ImmersiveColorSet\0".encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
                    SendMessageTimeoutW(
                        HWND_BROADCAST,          // Send to all top-level windows
                        WM_SETTINGCHANGE,        // Message: Setting change
                        0 as WPARAM,             // No additional parameter
                        setting_change_message.as_ptr() as LPARAM, // Setting name
                        SMTO_ABORTIFHUNG,        // Timeout if the window is hung
                        5000,                    // Timeout in milliseconds
                        null_mut(),              // No return value needed
                    );
                }
                Ok(())
            })
            .await
            .map_err(|_| {
                Error::new(
                    Status::GenericFailure,
                    "Failed to execute async task".to_string(),
                )
            })?
        })
    }

    fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
        Ok(())
    }
}