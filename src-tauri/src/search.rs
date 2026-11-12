use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(all(unix, not(target_os = "macos")))]
use std::sync::mpsc;
#[cfg(all(unix, not(target_os = "macos")))]
use std::time::Duration;

use crate::rank::{score_name, Hit, HitKind, path_key};
use crate::util::home_dir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOut {
    pub hits: Vec<Hit>,
    pub truncated: bool,
}

pub fn search_scope(input: &str) -> SearchOut {
    let Some(scope) = resolve_scope(input) else {
        return SearchOut {
            hits: Vec::new(),
            truncated: false,
        };
    };
    if scope.list_only {
        SearchOut {
            hits: list_dir(&scope.root, 40),
            truncated: false,
        }
    } else {
        find_named(&scope.root, &scope.filter, 3, 4000, 30)
    }
}

pub fn search_user_files(query: &str) -> SearchOut {
    if query.trim().chars().count() < 3 {
        return SearchOut {
            hits: Vec::new(),
            truncated: false,
        };
    }
    let mut hits = Vec::new();
    let mut seen_roots = HashSet::new();
    let mut stats = WalkStats {
        visits: 0,
        truncated: false,
    };
    for dir in quick_dirs() {
        let canon = fs::canonicalize(&dir).unwrap_or_else(|_| dir.clone());
        if !seen_roots.insert(canon) {
            continue;
        }
        if stats.visits >= 4000 {
            stats.truncated = true;
            break;
        }
        let mut seen = HashSet::new();
        walk(
            &dir,
            Some(query),
            3,
            4000,
            40,
            &mut stats,
            &mut seen,
            &mut hits,
        );
    }
    hits.sort_by(|a, b| {
        let left = score_name(query, &a.name).map(|item| item.0).unwrap_or(0);
        let right = score_name(query, &b.name).map(|item| item.0).unwrap_or(0);
        right
            .cmp(&left)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    if hits.len() > 20 {
        hits.truncate(20);
        stats.truncated = true;
    }
    SearchOut {
        hits: dedupe(hits),
        truncated: stats.truncated,
    }
}

struct Scope {
    root: PathBuf,
    filter: String,
    list_only: bool,
}

fn resolve_scope(input: &str) -> Option<Scope> {
    let expanded = expand_tilde(input.trim());
    let normalized = normalize_slashes(&expanded);
    if let Some((root, filter)) = drive_with_filter(&normalized) {
        return Some(Scope {
            root,
            filter,
            list_only: false,
        });
    }
    if let Some(root) = bare_drive(&normalized) {
        return Some(Scope {
            root,
            filter: String::new(),
            list_only: true,
        });
    }

    let path = PathBuf::from(&normalized);
    let ends_sep = normalized.ends_with('/') || normalized.ends_with('\\');
    if path.is_dir() && (ends_sep || path.file_name().is_some() || is_filesystem_root(&path)) {
        return Some(Scope {
            root: path,
            filter: String::new(),
            list_only: true,
        });
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && parent.is_dir() {
            let filter = path.file_name()?.to_string_lossy().to_string();
            return Some(Scope {
                root: parent.to_path_buf(),
                filter,
                list_only: false,
            });
        }
    }

    let mut cursor = path.as_path();
    loop {
        let parent = cursor.parent()?;
        if parent == cursor {
            return None;
        }
        if parent.is_dir() {
            let rest = path.strip_prefix(parent).ok()?;
            let filter = rest.components().next()?.as_os_str().to_string_lossy().to_string();
            return Some(Scope {
                root: parent.to_path_buf(),
                filter,
                list_only: false,
            });
        }
        cursor = parent;
    }
}

fn is_filesystem_root(path: &Path) -> bool {
    path.parent().is_none()
}

fn drive_with_filter(normalized: &str) -> Option<(PathBuf, String)> {
    let bytes = normalized.as_bytes();
    if bytes.len() > 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && bytes.get(2) != Some(&b'\\')
        && bytes.get(2) != Some(&b'/')
    {
        let root = format!("{}:\\", (bytes[0] as char).to_ascii_uppercase());
        return Some((PathBuf::from(root), normalized[2..].to_string()));
    }
    None
}

fn bare_drive(normalized: &str) -> Option<PathBuf> {
    let trimmed = normalized.trim_end_matches(['\\', '/']);
    let bytes = trimmed.as_bytes();
    if bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return Some(PathBuf::from(format!(
            "{}:\\",
            (bytes[0] as char).to_ascii_uppercase()
        )));
    }
    None
}

fn expand_tilde(input: &str) -> String {
    if input == "~" {
        return home_dir()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| input.to_string());
    }
    if let Some(rest) = input.strip_prefix("~/").or_else(|| input.strip_prefix("~\\")) {
        if let Some(home) = home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    }
    input.to_string()
}

fn normalize_slashes(input: &str) -> String {
    #[cfg(windows)]
    {
        input.replace('/', "\\")
    }
    #[cfg(not(windows))]
    {
        input.to_string()
    }
}

pub fn list_dir(root: &Path, limit: usize) -> Vec<Hit> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut hits = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "." || name == ".." {
            continue;
        }
        let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
        hits.push(hit_from(&name, &entry.path(), is_dir));
        if hits.len() >= limit.saturating_mul(4) {
            break;
        }
    }
    hits.sort_by(|a, b| {
        let left_dir = matches!(a.kind, HitKind::Folder);
        let right_dir = matches!(b.kind, HitKind::Folder);
        right_dir
            .cmp(&left_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    hits.truncate(limit);
    hits
}

pub fn find_named(
    root: &Path,
    filter: &str,
    max_depth: usize,
    max_visits: usize,
    max_results: usize,
) -> SearchOut {
    let mut hits = Vec::new();
    let mut seen = HashSet::new();
    let mut stats = WalkStats {
        visits: 0,
        truncated: false,
    };
    walk(
        root,
        Some(filter),
        max_depth,
        max_visits,
        max_results.saturating_mul(3).max(max_results),
        &mut stats,
        &mut seen,
        &mut hits,
    );
    hits.sort_by(|a, b| {
        let left = score_name(filter, &a.name).map(|item| item.0).unwrap_or(0);
        let right = score_name(filter, &b.name).map(|item| item.0).unwrap_or(0);
        right
            .cmp(&left)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    if hits.len() > max_results {
        hits.truncate(max_results);
        stats.truncated = true;
    }
    SearchOut {
        hits,
        truncated: stats.truncated,
    }
}

struct WalkStats {
    visits: usize,
    truncated: bool,
}

fn walk(
    dir: &Path,
    filter: Option<&str>,
    depth_left: usize,
    max_visits: usize,
    max_hits: usize,
    stats: &mut WalkStats,
    seen: &mut HashSet<String>,
    hits: &mut Vec<Hit>,
) {
    if stats.visits >= max_visits || hits.len() >= max_hits {
        stats.truncated = true;
        return;
    }
    let canon = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let canon_key = path_key(&canon.to_string_lossy());
    if !seen.insert(canon_key) {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut children = Vec::new();
    for entry in entries.flatten() {
        stats.visits += 1;
        if stats.visits >= max_visits {
            stats.truncated = true;
            break;
        }
        children.push(entry);
    }
    for entry in children {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "." || name == ".." {
            continue;
        }
        let path = entry.path();
        let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
        let matched = match filter {
            None => true,
            Some(filter) => score_name(filter, &name).is_some(),
        };
        if matched {
            hits.push(hit_from(&name, &path, is_dir));
            if hits.len() >= max_hits {
                stats.truncated = true;
                return;
            }
        }
        if is_dir && depth_left > 0 && !skip_dir_name(&name) {
            walk(
                &path,
                filter,
                depth_left - 1,
                max_visits,
                max_hits,
                stats,
                seen,
                hits,
            );
        }
    }
}

fn hit_from(name: &str, path: &Path, is_dir: bool) -> Hit {
    Hit {
        name: name.to_string(),
        path: path.display().to_string(),
        subtitle: path.display().to_string(),
        kind: if is_dir { HitKind::Folder } else { HitKind::File },
    }
}

fn skip_dir_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "windows"
            | "windows.old"
            | "node_modules"
            | ".git"
            | "$recycle.bin"
            | "system volume information"
            | "proc"
            | "sys"
            | "dev"
    )
}

fn dedupe(hits: Vec<Hit>) -> Vec<Hit> {
    let mut seen = HashSet::new();
    hits.into_iter()
        .filter(|hit| seen.insert(path_key(&hit.path)))
        .collect()
}

fn quick_dirs() -> Vec<PathBuf> {
    let Some(home) = home_dir() else {
        return Vec::new();
    };
    let mut dirs = Vec::new();
    for name in ["Desktop", "Documents", "Downloads", "Pictures", "Music", "Videos"] {
        let path = home.join(name);
        if path.is_dir() {
            dirs.push(path);
        }
    }
    #[cfg(windows)]
    for name in ["Desktop", "Documents", "Downloads"] {
        let path = home.join("OneDrive").join(name);
        if path.is_dir() {
            dirs.push(path);
        }
    }
    if dirs.is_empty() {
        dirs.push(home);
    }
    dirs
}

pub fn scan_apps() -> Vec<Hit> {
    let mut hits = Vec::new();
    #[cfg(windows)]
    scan_start_menu(&mut hits);
    #[cfg(target_os = "macos")]
    scan_mac_apps(&mut hits);
    #[cfg(all(unix, not(target_os = "macos")))]
    scan_desktop_files(&mut hits);
    dedupe(hits)
}

#[cfg(windows)]
fn scan_start_menu(out: &mut Vec<Hit>) {
    let mut roots = Vec::new();
    if let Some(program_data) = std::env::var_os("ProgramData") {
        roots.push(PathBuf::from(program_data).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    if let Some(app_data) = std::env::var_os("APPDATA") {
        roots.push(PathBuf::from(app_data).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    let mut seen = HashSet::new();
    for root in roots {
        walk_shortcuts(&root, 8, &mut 0, &mut seen, out);
    }
}

#[cfg(windows)]
fn walk_shortcuts(
    dir: &Path,
    depth: usize,
    visits: &mut usize,
    seen: &mut HashSet<PathBuf>,
    out: &mut Vec<Hit>,
) {
    if depth == 0 || *visits > 8000 {
        return;
    }
    let canon = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if !seen.insert(canon) {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        *visits += 1;
        if *visits > 8000 {
            return;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.eq_ignore_ascii_case("desktop.ini") {
            continue;
        }
        let is_dir = entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false);
        if is_dir {
            walk_shortcuts(&path, depth - 1, visits, seen, out);
            continue;
        }
        if !name.to_ascii_lowercase().ends_with(".lnk") {
            continue;
        }
        let stem = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
            .unwrap_or(name);
        let target = read_lnk_target(&path);
        out.push(Hit {
            name: stem,
            path: path.display().to_string(),
            subtitle: target.unwrap_or_else(|| path.display().to_string()),
            kind: HitKind::App,
        });
    }
}

#[cfg(target_os = "macos")]
fn scan_mac_apps(out: &mut Vec<Hit>) {
    let mut roots = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];
    if let Some(home) = home_dir() {
        roots.push(home.join("Applications"));
    }
    for root in roots {
        walk_apps(&root, 3, out);
    }
}

#[cfg(target_os = "macos")]
fn walk_apps(dir: &Path, depth: usize, out: &mut Vec<Hit>) {
    if depth == 0 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.to_ascii_lowercase().ends_with(".app") {
            let stem = name.trim_end_matches(".app").trim_end_matches(".APP");
            let display = if stem.is_empty() { name.as_str() } else { stem };
            out.push(Hit {
                name: display.to_string(),
                path: path.display().to_string(),
                subtitle: path.display().to_string(),
                kind: HitKind::App,
            });
            continue;
        }
        if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            walk_apps(&path, depth - 1, out);
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn scan_desktop_files(out: &mut Vec<Hit>) {
    let mut roots = vec![
        PathBuf::from("/usr/share/applications"),
        PathBuf::from("/usr/local/share/applications"),
        PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        PathBuf::from("/var/lib/snapd/desktop/applications"),
    ];
    if let Some(home) = home_dir() {
        roots.push(home.join(".local/share/applications"));
        roots.push(home.join(".local/share/flatpak/exports/share/applications"));
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_DIRS") {
        for dir in xdg.split(':') {
            if !dir.is_empty() {
                roots.push(PathBuf::from(dir).join("applications"));
            }
        }
    }
    for root in roots {
        out.extend(desktop_hits_from(&root));
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn desktop_hits_from(root: &Path) -> Vec<Hit> {
    let root = root.to_path_buf();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(read_desktop_dir(&root));
    });
    rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn read_desktop_dir(root: &Path) -> Vec<Hit> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut hits = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("desktop") {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if !meta.is_file() || meta.len() > 1_000_000 {
            continue;
        }
        if let Some(hit) = parse_desktop_file(&path) {
            hits.push(hit);
        }
    }
    hits
}

#[cfg(all(unix, not(target_os = "macos")))]
fn parse_desktop_file(path: &Path) -> Option<Hit> {
    let text = fs::read_to_string(path).ok()?;
    parse_desktop_text(&text, &path.display().to_string())
}

#[cfg(any(test, all(unix, not(target_os = "macos"))))]
fn parse_desktop_text(text: &str, path: &str) -> Option<Hit> {
    let mut name = None;
    let mut exec = None;
    let mut no_display = false;
    let mut hidden = false;
    let mut type_ok = true;
    let mut in_entry = false;
    let mut saw_header = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_entry = line.eq_ignore_ascii_case("[Desktop Entry]");
            saw_header = true;
            continue;
        }
        if !saw_header {
            in_entry = true;
        }
        if !in_entry || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if key == "Name" {
            name = Some(value.to_string());
        } else if name.is_none() && key.starts_with("Name[") {
            name = Some(value.to_string());
        } else if key == "Exec" && exec.is_none() {
            exec = Some(value.to_string());
        } else if key.eq_ignore_ascii_case("NoDisplay") && value.eq_ignore_ascii_case("true") {
            no_display = true;
        } else if key.eq_ignore_ascii_case("Hidden") && value.eq_ignore_ascii_case("true") {
            hidden = true;
        } else if key == "Type" && !value.eq_ignore_ascii_case("Application") {
            type_ok = false;
        }
    }
    if no_display || hidden || !type_ok || exec.is_none() {
        return None;
    }
    Some(Hit {
        name: name?,
        path: path.to_string(),
        subtitle: path.to_string(),
        kind: HitKind::App,
    })
}

#[cfg(any(test, windows))]
pub fn read_lnk_target(path: &Path) -> Option<String> {
    let data = fs::read(path).ok()?;
    read_lnk_bytes(&data)
}

#[cfg(any(test, windows))]
fn read_lnk_bytes(data: &[u8]) -> Option<String> {
    if data.len() < 76 {
        return None;
    }
    let header_size = u32::from_le_bytes(data[0..4].try_into().ok()?) as usize;
    if header_size < 76 || data.len() < header_size {
        return None;
    }
    let flags = u32::from_le_bytes(data[20..24].try_into().ok()?);
    let mut offset = header_size;
    if flags & 0x1 != 0 {
        if offset + 2 > data.len() {
            return None;
        }
        let id_size = u16::from_le_bytes(data[offset..offset + 2].try_into().ok()?) as usize;
        offset = offset.checked_add(2)?.checked_add(id_size)?;
        if offset > data.len() {
            return None;
        }
    }
    if flags & 0x2 == 0 || offset + 28 > data.len() {
        return None;
    }
    let info = &data[offset..];
    let header = u32::from_le_bytes(info[4..8].try_into().ok()?) as usize;
    let mut unicode_offset = 0usize;
    if header >= 0x24 && info.len() >= 32 {
        unicode_offset = u32::from_le_bytes(info[28..32].try_into().ok()?) as usize;
    }
    let local_offset = u32::from_le_bytes(info[16..20].try_into().ok()?) as usize;
    if unicode_offset > 0 {
        if let Some(path) = read_utf16_z(info, unicode_offset) {
            return Some(path);
        }
    }
    if local_offset > 0 {
        return read_ansi_z(info, local_offset);
    }
    None
}

#[cfg(any(test, windows))]
fn read_ansi_z(buf: &[u8], offset: usize) -> Option<String> {
    if offset >= buf.len() {
        return None;
    }
    let end = buf[offset..]
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(buf.len() - offset);
    let bytes = &buf[offset..offset + end];
    if bytes.is_empty() {
        return None;
    }
    Some(String::from_utf8_lossy(bytes).into_owned())
}

#[cfg(any(test, windows))]
fn read_utf16_z(buf: &[u8], offset: usize) -> Option<String> {
    if offset >= buf.len() {
        return None;
    }
    let mut units = Vec::new();
    let mut index = offset;
    while index + 1 < buf.len() {
        let unit = u16::from_le_bytes([buf[index], buf[index + 1]]);
        if unit == 0 {
            break;
        }
        units.push(unit);
        index += 2;
    }
    if units.is_empty() {
        return None;
    }
    Some(String::from_utf16_lossy(&units))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempTree(PathBuf);
    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str) -> TempTree {
        let path = std::env::temp_dir().join(format!(
            "topcalc-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        TempTree(path)
    }

    #[test]
    fn skips_heavy_directories_unless_search_starts_inside_them() {
        let tree = scratch("walk");
        fs::create_dir_all(tree.0.join("keep")).unwrap();
        fs::write(tree.0.join("keep").join("alpha.txt"), b"a").unwrap();
        fs::write(tree.0.join("alpha-root.txt"), b"d").unwrap();
        fs::create_dir_all(tree.0.join("node_modules").join("pkg")).unwrap();
        fs::write(tree.0.join("node_modules").join("pkg").join("alpha.txt"), b"b").unwrap();
        fs::create_dir_all(tree.0.join(".git")).unwrap();
        fs::write(tree.0.join(".git").join("alpha.txt"), b"c").unwrap();
        fs::create_dir_all(tree.0.join("Windows").join("System32")).unwrap();
        fs::write(tree.0.join("Windows").join("alpha.txt"), b"w").unwrap();
        fs::write(tree.0.join("Windows").join("System32").join("alpha.txt"), b"s").unwrap();

        let found = find_named(&tree.0, "alpha", 4, 1000, 30);
        let paths: Vec<_> = found.hits.iter().map(|hit| hit.path.clone()).collect();
        assert!(paths.iter().any(|path| path.contains("keep") && path.ends_with("alpha.txt")));
        assert!(paths.iter().any(|path| path.ends_with("alpha-root.txt")));
        assert!(paths.iter().all(|path| !path.contains("node_modules")));
        assert!(paths.iter().all(|path| !path.contains(".git")));
        assert!(paths.iter().all(|path| !path.contains("Windows")));

        let windows = find_named(&tree.0, "windows", 3, 1000, 10);
        assert!(windows.hits.iter().any(|hit| hit.name.eq_ignore_ascii_case("Windows")));
        assert!(windows.hits.iter().all(|hit| hit.name != "alpha.txt"));

        let inside = find_named(&tree.0.join("Windows"), "alpha", 3, 1000, 10);
        assert!(inside.hits.iter().any(|hit| hit.name == "alpha.txt"));
    }

    #[test]
    fn partial_path_searches_the_parent() {
        let tree = scratch("scope");
        fs::create_dir_all(tree.0.join("projects")).unwrap();
        fs::write(tree.0.join("projects").join("alpha.txt"), b"1").unwrap();
        fs::create_dir_all(tree.0.join("projects").join("node_modules").join("pkg")).unwrap();
        fs::write(
            tree.0.join("projects").join("node_modules").join("pkg").join("alpha.txt"),
            b"2",
        )
        .unwrap();
        let query = tree.0.join("projects").join("alp");
        let found = search_scope(&query.to_string_lossy());
        assert!(found.hits.iter().any(|hit| hit.name == "alpha.txt"));
        assert!(found.hits.iter().all(|hit| !hit.path.contains("node_modules")));

        let mut listed = tree.0.to_string_lossy().to_string();
        if !listed.ends_with(std::path::MAIN_SEPARATOR) {
            listed.push(std::path::MAIN_SEPARATOR);
        }
        let children = search_scope(&listed);
        assert!(children.hits.iter().any(|hit| hit.name == "projects"));
        assert!(children.hits.iter().all(|hit| hit.name != "alpha.txt"));
    }

    #[test]
    fn shortcut_parser_reads_local_path_after_an_id_list() {
        let simple = craft_lnk(r"C:\Windows\notepad.exe", false);
        assert_eq!(
            read_lnk_bytes(&simple).as_deref(),
            Some(r"C:\Windows\notepad.exe")
        );
        let with_list = craft_lnk(r"D:\Apps\TopCalc.exe", true);
        assert_eq!(
            read_lnk_bytes(&with_list).as_deref(),
            Some(r"D:\Apps\TopCalc.exe")
        );
        assert!(read_lnk_bytes(&[0, 1, 2]).is_none());
    }

    fn craft_lnk(target: &str, with_id_list: bool) -> Vec<u8> {
        let mut data = vec![0u8; 76];
        data[0..4].copy_from_slice(&76u32.to_le_bytes());
        let flags: u32 = if with_id_list { 0x3 } else { 0x2 };
        data[20..24].copy_from_slice(&flags.to_le_bytes());
        if with_id_list {
            data.extend_from_slice(&2u16.to_le_bytes());
            data.extend_from_slice(&[0, 0]);
        }
        let path = target.as_bytes();
        let header_size = 28u32;
        let local_off = header_size;
        let suffix_off = local_off + path.len() as u32 + 1;
        let total = suffix_off + 1;
        let mut info = vec![0u8; total as usize];
        info[0..4].copy_from_slice(&total.to_le_bytes());
        info[4..8].copy_from_slice(&header_size.to_le_bytes());
        info[8..12].copy_from_slice(&1u32.to_le_bytes());
        info[16..20].copy_from_slice(&local_off.to_le_bytes());
        info[24..28].copy_from_slice(&suffix_off.to_le_bytes());
        info[local_off as usize..local_off as usize + path.len()].copy_from_slice(path);
        data.extend(info);
        data
    }

    #[test]
    fn desktop_entries_skip_hidden_and_non_apps() {
        let app = parse_desktop_text(
            "[Desktop Entry]\nType=Application\nName=Files\nExec=nautilus %U\n",
            "/usr/share/applications/org.gnome.Nautilus.desktop",
        )
        .unwrap();
        assert_eq!(app.name, "Files");
        assert_eq!(app.kind, HitKind::App);
        assert!(parse_desktop_text(
            "[Desktop Entry]\nType=Application\nName=Secret\nExec=secret\nNoDisplay=true\n",
            "/tmp/secret.desktop",
        )
        .is_none());
        assert!(parse_desktop_text(
            "[Desktop Entry]\nType=Link\nName=Site\nExec=true\n",
            "/tmp/site.desktop",
        )
        .is_none());
    }

    #[test]
    fn scan_apps_stays_bounded() {
        let apps = scan_apps();
        assert!(apps.len() < 20_000);
    }
}
