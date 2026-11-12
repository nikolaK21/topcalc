use std::path::PathBuf;

pub fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(profile) = std::env::var_os("USERPROFILE") {
            return Some(PathBuf::from(profile));
        }
        let drive = std::env::var_os("HOMEDRIVE")?;
        let path = std::env::var_os("HOMEPATH")?;
        return Some(PathBuf::from(drive).join(path));
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

pub fn which(exe: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(exe);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        if !exe.to_ascii_lowercase().ends_with(".exe") {
            let with_exe = dir.join(format!("{exe}.exe"));
            if with_exe.is_file() {
                return Some(with_exe);
            }
        }
    }
    None
}
