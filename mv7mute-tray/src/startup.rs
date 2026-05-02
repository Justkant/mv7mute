#[cfg(target_os = "windows")]
mod imp {
    use std::env;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};

    const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const RUN_VALUE_NAME: &str = "MV7MuteTray";

    fn parse_run_command_path(command: &str) -> Option<PathBuf> {
        let command = command.trim();
        if command.is_empty() {
            return None;
        }

        if let Some(quoted) = command.strip_prefix('"') {
            let end = quoted.find('"')?;
            return Some(PathBuf::from(&quoted[..end]));
        }

        Some(PathBuf::from(
            command.split_whitespace().next().unwrap_or_default(),
        ))
    }

    fn normalize_path(path: &Path) -> Result<PathBuf, String> {
        fs::canonicalize(path).map_err(|error| error.to_string())
    }

    fn run_value_matches_current_exe(value: &str, current_exe: &Path) -> Result<bool, String> {
        let Some(configured_path) = parse_run_command_path(value) else {
            return Ok(false);
        };
        if !configured_path.exists() {
            return Ok(false);
        }

        Ok(normalize_path(&configured_path)? == normalize_path(current_exe)?)
    }

    pub fn is_enabled() -> Result<bool, String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let key = match hkcu.open_subkey_with_flags(RUN_KEY_PATH, KEY_READ) {
            Ok(key) => key,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.to_string()),
        };
        match key.get_value::<String, _>(RUN_VALUE_NAME) {
            Ok(value) => {
                let current_exe = env::current_exe().map_err(|error| error.to_string())?;
                run_value_matches_current_exe(&value, &current_exe)
            }
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

    #[cfg(test)]
    mod tests {
        use super::{parse_run_command_path, run_value_matches_current_exe};
        use std::fs;
        use std::path::PathBuf;
        use std::time::{SystemTime, UNIX_EPOCH};

        fn unique_test_dir() -> PathBuf {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos();
            std::env::temp_dir().join(format!("mv7mute-startup-tests-{nanos}"))
        }

        #[test]
        fn parse_run_command_path_handles_empty_string() {
            assert_eq!(parse_run_command_path(""), None);
            assert_eq!(parse_run_command_path("   "), None);
        }

        #[test]
        fn parse_run_command_path_handles_quoted_commands() {
            assert_eq!(
                parse_run_command_path("\"C:\\\\Program Files\\\\mv7mute\\\\tray.exe\" --minimized"),
                Some(PathBuf::from(r"C:\\Program Files\\mv7mute\\tray.exe"))
            );
        }

        #[test]
        fn parse_run_command_path_handles_unquoted_commands() {
            assert_eq!(
                parse_run_command_path(r"C:\mv7mute\tray.exe --minimized"),
                Some(PathBuf::from(r"C:\mv7mute\tray.exe"))
            );
        }

        #[test]
        fn run_value_matches_current_exe_rejects_missing_target() {
            let missing = PathBuf::from(r"C:\definitely-missing\mv7mute-tray.exe");
            assert!(!run_value_matches_current_exe("\"C:\\definitely-missing\\mv7mute-tray.exe\"", &missing)
                .expect("missing target should return false"));
        }

        #[test]
        fn run_value_matches_current_exe_accepts_current_exe_path() {
            let dir = unique_test_dir();
            fs::create_dir_all(&dir).expect("create temp dir");

            let exe = dir.join("mv7mute-tray.exe");
            fs::write(&exe, b"test").expect("create exe placeholder");

            let command = format!("\"{}\" --background", exe.display());
            assert!(run_value_matches_current_exe(&command, &exe).expect("matching exe should pass"));

            fs::remove_dir_all(&dir).expect("remove temp dir");
        }

        #[test]
        fn run_value_matches_current_exe_rejects_different_existing_exe() {
            let dir = unique_test_dir();
            fs::create_dir_all(&dir).expect("create temp dir");

            let current = dir.join("current.exe");
            let stale = dir.join("stale.exe");
            fs::write(&current, b"current").expect("create current exe placeholder");
            fs::write(&stale, b"stale").expect("create stale exe placeholder");

            let command = format!("\"{}\"", stale.display());
            assert!(
                !run_value_matches_current_exe(&command, &current)
                    .expect("different exe should be rejected")
            );

            fs::remove_dir_all(&dir).expect("remove temp dir");
        }
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
