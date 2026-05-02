#[cfg(target_os = "windows")]
mod imp {
    use std::env;
    use std::io;
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};

    const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const RUN_VALUE_NAME: &str = "MV7MuteTray";

    pub fn is_enabled() -> Result<bool, String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = match hkcu.open_subkey_with_flags(RUN_KEY_PATH, KEY_READ) {
            Ok(key) => key,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.to_string()),
        };
        match key.get_value::<String, _>(RUN_VALUE_NAME) {
            Ok(value) => Ok(!value.trim().is_empty()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn set_enabled(enabled: bool) -> Result<bool, String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(RUN_KEY_PATH)
            .map_err(|error| error.to_string())?;

        if enabled {
            let exe = env::current_exe().map_err(|error| error.to_string())?;
            let exe = exe.to_string_lossy().replace('"', "\"\"");
            key.set_value(RUN_VALUE_NAME, &format!("\"{exe}\""))
                .map_err(|error| error.to_string())?;
        } else {
            match key.delete_value(RUN_VALUE_NAME) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
        }

        is_enabled()
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    pub fn is_enabled() -> Result<bool, String> {
        Ok(false)
    }

    pub fn set_enabled(_enabled: bool) -> Result<bool, String> {
        Err("Launch at startup is only supported on Windows".to_string())
    }
}

pub fn is_enabled() -> Result<bool, String> {
    imp::is_enabled()
}

pub fn set_enabled(enabled: bool) -> Result<bool, String> {
    imp::set_enabled(enabled)
}
