use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::util::{home_dir, which};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellFlavor {
    PowerShell,
    Bash,
    User,
}

impl ShellFlavor {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PowerShell => "powershell",
            Self::Bash => "bash",
            Self::User => "user",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "powershell" => Some(Self::PowerShell),
            "bash" => Some(Self::Bash),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostCaps {
    pub bash_available: bool,
    pub terminal_name: String,
}

pub fn current_host() -> HostCaps {
    HostCaps {
        bash_available: bash_available(),
        terminal_name: terminal_name(),
    }
}

#[derive(Debug, Clone)]
pub struct ShellOutput {
    pub timed_out: bool,
    pub code: Option<i32>,
    pub output: String,
}

pub fn present_output(command: &str, out: &ShellOutput) -> String {
    let mut text = String::new();
    text.push_str("$ ");
    text.push_str(command);
    text.push('\n');
    if out.timed_out {
        text.push_str("Timed out after 8s. Open in terminal to keep it running.\n");
    } else if let Some(code) = out.code {
        if code != 0 {
            text.push_str(&format!("Exit code {code}\n"));
        }
    }
    let body = out.output.trim();
    if body.is_empty() {
        text.push_str("(no output)");
    } else {
        text.push_str(body);
    }
    text
}

pub fn run_inline(command: &str, flavor: ShellFlavor, timeout: Duration) -> Result<ShellOutput, String> {
    let command = command.trim();
    if command.is_empty() {
        return Err("Type a command first".into());
    }
    if command.len() > 8000 {
        return Err("That command is too long".into());
    }
    let mut cmd = inline_command(command, flavor)?;
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    if let Some(home) = home_dir() {
        if home.is_dir() {
            cmd.current_dir(home);
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }

    let mut child = cmd.spawn().map_err(|err| err.to_string())?;
    let mut stdout = child.stdout.take().ok_or("Could not read stdout")?;
    let mut stderr = child.stderr.take().ok_or("Could not read stderr")?;
    let (out_tx, out_rx) = mpsc::channel();
    let (err_tx, err_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = out_tx.send(read_capped(&mut stdout, 16_000));
    });
    std::thread::spawn(move || {
        let _ = err_tx.send(read_capped(&mut stderr, 8_000));
    });

    let started = Instant::now();
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if started.elapsed() > timeout => {
                timed_out = true;
                force_kill(&mut child);
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(30)),
            Err(err) => return Err(err.to_string()),
        }
    };

    let grace = Duration::from_secs(2);
    let stdout = out_rx.recv_timeout(grace).unwrap_or_default();
    let stderr = err_rx.recv_timeout(grace).unwrap_or_default();
    if timed_out {
        let deadline = Instant::now() + Duration::from_secs(1);
        while Instant::now() < deadline {
            if child.try_wait().ok().flatten().is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(30));
        }
    } else {
        let _ = child.wait();
    }
    let code = status.and_then(|status| status.code());
    let mut output = String::new();
    if !stdout.trim().is_empty() {
        output.push_str(stdout.trim_end());
    }
    if !stderr.trim().is_empty() {
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(stderr.trim_end());
    }
    Ok(ShellOutput {
        timed_out,
        code,
        output,
    })
}

fn inline_command(command: &str, flavor: ShellFlavor) -> Result<Command, String> {
    match flavor {
        ShellFlavor::PowerShell => {
            let mut cmd = Command::new(powershell_bin());
            let wrapped = format!(
                "$OutputEncoding = [Console]::OutputEncoding = New-Object System.Text.UTF8Encoding $false; {command}"
            );
            cmd.args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &wrapped,
            ]);
            Ok(cmd)
        }
        ShellFlavor::Bash => match find_bash() {
            Some(BashHost::Exe(path)) => {
                let mut cmd = Command::new(path);
                cmd.args(["-c", command]);
                Ok(cmd)
            }
            Some(BashHost::Wsl) => {
                let mut cmd = Command::new("wsl.exe");
                cmd.args(["-e", "bash", "-c", command]);
                Ok(cmd)
            }
            None => Err("Bash was not found. Use > for PowerShell, or install Git Bash or WSL.".into()),
        },
        ShellFlavor::User => {
            #[cfg(windows)]
            {
                return inline_command(command, ShellFlavor::PowerShell);
            }
            #[cfg(not(windows))]
            {
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
                let mut cmd = Command::new(shell);
                cmd.args(["-c", command]);
                Ok(cmd)
            }
        }
    }
}

fn powershell_bin() -> String {
    which("powershell.exe")
        .or_else(|| which("pwsh.exe"))
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "powershell.exe".into())
}

fn force_kill(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let pid = child.id();
        let _ = Command::new("kill")
            .args(["-9", &format!("-{pid}")])
            .output();
    }
    let _ = child.kill();
}

fn read_capped(reader: &mut impl Read, cap: usize) -> String {
    let mut kept = Vec::new();
    let mut extra = 0usize;
    let mut buf = [0u8; 2048];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(count) => {
                if kept.len() < cap {
                    let take = count.min(cap - kept.len());
                    kept.extend_from_slice(&buf[..take]);
                    extra += count - take;
                } else {
                    extra += count;
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    let mut text = decode_console(&kept);
    if extra > 0 {
        text.push_str("\n… output truncated");
    }
    text
}

fn decode_console(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        return decode_utf16(&bytes[2..]);
    }
    if looks_utf16_le(bytes) {
        return decode_utf16(bytes);
    }
    String::from_utf8_lossy(bytes).into_owned()
}

fn looks_utf16_le(bytes: &[u8]) -> bool {
    if bytes.len() < 8 || bytes.len() % 2 != 0 {
        return false;
    }
    let zeros = bytes.iter().skip(1).step_by(2).filter(|byte| **byte == 0).count();
    zeros * 4 > (bytes.len() / 2) * 3
}

fn decode_utf16(bytes: &[u8]) -> String {
    let mut units = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0;
    while index + 1 < bytes.len() {
        units.push(u16::from_le_bytes([bytes[index], bytes[index + 1]]));
        index += 2;
    }
    String::from_utf16_lossy(&units)
        .trim_end_matches('\u{0}')
        .to_string()
}

#[derive(Debug, Clone)]
enum BashHost {
    Exe(PathBuf),
    Wsl,
}

pub fn bash_available() -> bool {
    find_bash().is_some()
}

fn find_bash() -> Option<BashHost> {
    #[cfg(windows)]
    {
        for candidate in [
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files (x86)\Git\bin\bash.exe",
        ] {
            if Path::new(candidate).is_file() {
                return Some(BashHost::Exe(PathBuf::from(candidate)));
            }
        }
        if let Some(path) = which("bash.exe").or_else(|| which("bash")) {
            let lower = path.to_string_lossy().to_ascii_lowercase();
            if !lower.contains("\\windows\\system32\\")
                && !lower.contains("\\windows\\sysnative\\")
                && !lower.contains("\\windowsapps\\")
            {
                return Some(BashHost::Exe(path));
            }
        }
        if wsl_has_distro() {
            return Some(BashHost::Wsl);
        }
        return None;
    }
    #[cfg(not(windows))]
    {
        if Path::new("/bin/bash").is_file() {
            Some(BashHost::Exe(PathBuf::from("/bin/bash")))
        } else {
            which("bash").map(BashHost::Exe)
        }
    }
}

#[cfg(windows)]
fn wsl_has_distro() -> bool {
    if which("wsl.exe").is_none() && which("wsl").is_none() {
        return false;
    }
    use std::os::windows::process::CommandExt;
    let output = Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Lxss",
            "/s",
            "/f",
            "DistributionName",
        ])
        .creation_flags(0x0800_0000)
        .stdin(Stdio::null())
        .output();
    match output {
        Ok(output) => {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout)
                    .to_ascii_lowercase()
                    .contains("distributionname")
        }
        Err(_) => false,
    }
}

pub fn terminal_name() -> String {
    #[cfg(windows)]
    {
        if which("wt.exe").is_some() || which("wt").is_some() {
            return "Windows Terminal".into();
        }
        return "PowerShell".into();
    }
    #[cfg(target_os = "macos")]
    {
        return "Terminal".into();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for program in [
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "alacritty",
            "kitty",
            "wezterm",
            "x-terminal-emulator",
            "xterm",
        ] {
            if which(program).is_some() {
                return program.to_string();
            }
        }
        "terminal".into()
    }
}

pub fn open_target(target: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    if target.ends_with(".desktop") {
        return launch_desktop(target);
    }
    open::that_detached(target).map_err(|err| err.to_string())
}

#[cfg(target_os = "linux")]
fn launch_desktop(path: &str) -> Result<(), String> {
    if which("gio").is_some() {
        return spawn_detached(Command::new("gio").args(["launch", path]));
    }
    if which("gtk-launch").is_some() {
        let id = Path::new(path)
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());
        return spawn_detached(Command::new("gtk-launch").arg(id));
    }
    if let Some(exec) = desktop_exec_line(Path::new(path)) {
        let parts = split_command(&exec);
        if let Some((bin, args)) = parts.split_first() {
            let mut cmd = Command::new(bin);
            cmd.args(args);
            return spawn_detached(&mut cmd);
        }
    }
    Err("Could not launch that application".into())
}

#[cfg(target_os = "linux")]
fn desktop_exec_line(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Exec=") {
            let parts: Vec<_> = split_command(rest)
                .into_iter()
                .filter(|token| !token.starts_with('%'))
                .collect();
            if parts.is_empty() {
                return None;
            }
            return Some(parts.join(" "));
        }
    }
    None
}

pub fn open_in_terminal(command: &str, flavor: ShellFlavor) -> Result<(), String> {
    let home = home_dir().unwrap_or_else(|| PathBuf::from("."));
    #[cfg(windows)]
    {
        return open_terminal_windows(&home, command, flavor);
    }
    #[cfg(target_os = "macos")]
    {
        let _ = flavor;
        return open_terminal_macos(command);
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = flavor;
        return open_terminal_linux(&home, command);
    }
}

#[cfg(windows)]
fn open_terminal_windows(home: &Path, command: &str, flavor: ShellFlavor) -> Result<(), String> {
    let home_s = home.display().to_string();
    if which("wt.exe").is_some() || which("wt").is_some() {
        let mut cmd = Command::new("wt.exe");
        cmd.arg("-d").arg(&home_s);
        match flavor {
            ShellFlavor::Bash => {
                let bash = find_bash().ok_or("Bash was not found")?;
                match bash {
                    BashHost::Exe(path) => {
                        cmd.arg(path);
                        if !command.is_empty() {
                            let script = format!("{command}; exec bash");
                            cmd.arg("-lc").arg(script);
                        }
                    }
                    BashHost::Wsl => {
                        cmd.arg("wsl.exe").arg("-e").arg("bash");
                        if !command.is_empty() {
                            let script = format!("{command}; exec bash");
                            cmd.arg("-lc").arg(script);
                        }
                    }
                }
            }
            _ => {
                cmd.arg("powershell.exe").arg("-NoExit");
                if !command.is_empty() {
                    cmd.arg("-Command").arg(command);
                }
            }
        }
        return spawn_detached(&mut cmd);
    }

    match flavor {
        ShellFlavor::Bash => {
            let bash = find_bash().ok_or("Bash was not found")?;
            match bash {
                BashHost::Exe(path) => {
                    let mut cmd = Command::new(path);
                    cmd.current_dir(home);
                    if !command.is_empty() {
                        let script = format!("{command}; exec bash");
                        cmd.args(["-lc", &script]);
                    }
                    spawn_detached(&mut cmd)
                }
                BashHost::Wsl => {
                    let mut cmd = Command::new("wsl.exe");
                    if command.is_empty() {
                        cmd.args(["-e", "bash"]);
                    } else {
                        let script = format!("{command}; exec bash");
                        cmd.args(["-e", "bash", "-lc", &script]);
                    }
                    spawn_detached(&mut cmd)
                }
            }
        }
        _ => {
            let mut cmd = Command::new(powershell_bin());
            cmd.current_dir(home);
            cmd.arg("-NoExit");
            if !command.is_empty() {
                cmd.arg("-Command").arg(command);
            }
            spawn_detached(&mut cmd)
        }
    }
}

#[cfg(target_os = "macos")]
fn open_terminal_macos(command: &str) -> Result<(), String> {
    if command.is_empty() {
        return spawn_detached(Command::new("open").args(["-a", "Terminal"]));
    }
    let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("tell application \"Terminal\" to do script \"{escaped}\"");
    spawn_detached(Command::new("osascript").args(["-e", &script]))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_terminal_linux(home: &Path, command: &str) -> Result<(), String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| {
        if Path::new("/bin/bash").exists() {
            "/bin/bash".into()
        } else {
            "/bin/sh".into()
        }
    });
    let script = if command.is_empty() {
        shell.clone()
    } else {
        format!("{command}; exec {shell}")
    };
    let programs = [
        "x-terminal-emulator",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "alacritty",
        "kitty",
        "wezterm",
        "xterm",
    ];
    let program = programs
        .into_iter()
        .find(|program| which(program).is_some())
        .unwrap_or("x-terminal-emulator");
    let mut cmd = Command::new(program);
    cmd.current_dir(home);
    match program {
        "gnome-terminal" => {
            cmd.arg(format!("--working-directory={}", home.display()));
            cmd.arg("--").arg(&shell).arg("-c").arg(&script);
        }
        "konsole" => {
            cmd.arg("--workdir").arg(home);
            cmd.arg("-e").arg(&shell).arg("-c").arg(&script);
        }
        "xfce4-terminal" => {
            cmd.arg(format!("--working-directory={}", home.display()));
            cmd.arg("--command").arg(format!("{shell} -c {script}"));
        }
        "kitty" | "alacritty" => {
            cmd.arg("--working-directory").arg(home);
            cmd.arg(&shell).arg("-c").arg(&script);
        }
        "wezterm" => {
            cmd.arg("start").arg("--cwd").arg(home);
            cmd.arg(&shell).arg("-c").arg(&script);
        }
        _ => {
            cmd.arg("-e").arg(&shell).arg("-c").arg(&script);
        }
    }
    spawn_detached(&mut cmd)
}

pub fn suspend_computer() -> Result<(), String> {
    #[cfg(windows)]
    {
        return suspend_windows();
    }
    #[cfg(target_os = "macos")]
    {
        return run_program("pmset", &["sleepnow"]);
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if run_program("loginctl", &["suspend"]).is_ok() {
            return Ok(());
        }
        return run_program("systemctl", &["suspend"]);
    }
}

#[cfg(windows)]
fn suspend_windows() -> Result<(), String> {
    #[link(name = "powrprof")]
    extern "system" {
        fn SetSuspendState(hibernate: u8, force_critical: u8, disable_wake_event: u8) -> i32;
    }
    let ok = unsafe { SetSuspendState(0, 0, 0) };
    if ok == 0 {
        Err("Windows could not sleep".into())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn run_program(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|err| format!("{program}: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} could not sleep ({status})"))
    }
}

pub fn spawn_detached(cmd: &mut Command) -> Result<(), String> {
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    let child = cmd.spawn().map_err(|err| err.to_string())?;
    std::thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });
    Ok(())
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn split_command(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for ch in input.chars() {
        match quote {
            Some(mark) if ch == mark => quote = None,
            Some(_) => current.push(ch),
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch.is_whitespace() => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            None => current.push(ch),
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_quoted_commands() {
        assert_eq!(
            split_command(r#""C:\Program Files\App.exe" --flag "a b""#),
            vec!["C:\\Program Files\\App.exe", "--flag", "a b"]
        );
    }

    #[test]
    fn presents_timeout_and_output() {
        let text = present_output(
            "sleep",
            &ShellOutput {
                timed_out: true,
                code: None,
                output: String::new(),
            },
        );
        assert!(text.contains("sleep"));
        assert!(text.contains("Timed out"));
    }

    #[test]
    fn rejects_empty_inline_command() {
        let err = run_inline("   ", ShellFlavor::User, Duration::from_secs(1)).unwrap_err();
        assert!(err.contains("command"));
    }

    #[cfg(windows)]
    #[test]
    fn powershell_echo_is_captured() {
        let out = run_inline(
            "Write-Output hi-topcalc",
            ShellFlavor::PowerShell,
            Duration::from_secs(8),
        )
        .unwrap();
        assert!(!out.timed_out, "{out:?}");
        assert!(out.output.contains("hi-topcalc"), "{out:?}");
    }

    #[cfg(unix)]
    #[test]
    fn user_shell_echo_is_captured() {
        let out = run_inline("echo hi-topcalc", ShellFlavor::User, Duration::from_secs(8)).unwrap();
        assert!(!out.timed_out, "{out:?}");
        assert!(out.output.contains("hi-topcalc"), "{out:?}");
    }

    #[test]
    fn long_command_times_out() {
        let (command, flavor) = if cfg!(windows) {
            ("Start-Sleep -Seconds 30", ShellFlavor::PowerShell)
        } else {
            ("sleep 30", ShellFlavor::User)
        };
        let started = Instant::now();
        let out = run_inline(command, flavor, Duration::from_millis(1200)).unwrap();
        assert!(out.timed_out, "{out:?}");
        assert!(started.elapsed() < Duration::from_secs(8));
    }
}
