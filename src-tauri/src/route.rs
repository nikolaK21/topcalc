use serde::{Deserialize, Serialize};

use crate::math::{self, format_number, MathPiece};
use crate::rank::{rank_hits, Hit, HitKind};
use crate::search::SearchOut;
use crate::shell::{HostCaps, ShellFlavor};

const COLORS: [&str; 6] = ["#6ea8fe", "#3dd68c", "#ffb020", "#ff6b81", "#c084fc", "#22d3ee"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    None,
    Open { path: String },
    Web { query: String },
    RunShell { command: String, flavor: String },
    OpenTerminal { command: String, flavor: String },
    Copy { text: String },
    ShowGraph,
    Sleep,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub subtitle: String,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SeriesMeta {
    pub label: String,
    pub expression: String,
    pub color: String,
    pub plot: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlotMarker {
    pub x: String,
    pub y: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlotFrame {
    pub x_min: String,
    pub x_max: String,
    pub y_min: String,
    pub y_max: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MathPayload {
    pub large_value: Option<String>,
    pub auto_graph: bool,
    pub interpretation_latex: String,
    pub result_label: Option<String>,
    pub result_latex: Option<String>,
    pub note: Option<String>,
    pub series: Vec<SeriesMeta>,
    pub markers: Vec<PlotMarker>,
    pub frame: Option<PlotFrame>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    pub mode: String,
    pub rows: Vec<Row>,
    pub math: Option<MathPayload>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OsKind {
    Windows,
    Unix,
}

fn current_os() -> OsKind {
    if cfg!(windows) {
        OsKind::Windows
    } else {
        OsKind::Unix
    }
}

#[derive(Debug)]
enum Class {
    Empty,
    Shell {
        flavor: ShellFlavor,
        command: String,
    },
    Math(Vec<MathPiece>),
    MathError(String),
    Path(String),
    Search(String),
    Website { shown: String, url: String },
    WebSearch(String),
    Sleep,
}

pub fn respond(
    input: &str,
    apps: &[Hit],
    host: &HostCaps,
    scope_search: impl Fn(&str) -> SearchOut,
    user_search: impl Fn(&str) -> SearchOut,
) -> QueryResponse {
    respond_on(input, apps, host, current_os(), scope_search, user_search)
}

fn respond_on(
    input: &str,
    apps: &[Hit],
    host: &HostCaps,
    os: OsKind,
    scope_search: impl Fn(&str) -> SearchOut,
    user_search: impl Fn(&str) -> SearchOut,
) -> QueryResponse {
    let class = classify_on(input, os);
    let response = match class {
        Class::Empty => QueryResponse {
            mode: "empty".into(),
            rows: Vec::new(),
            math: None,
        },
        Class::Shell { flavor, command } => shell_response(flavor, command, host),
        Class::Math(pieces) => math_response(pieces),
        Class::MathError(message) => QueryResponse {
            mode: "math".into(),
            rows: vec![Row {
                id: String::new(),
                kind: "Calculate".into(),
                title: "Can't calculate".into(),
                subtitle: message,
                action: Action::None,
            }],
            math: None,
        },
        Class::Path(raw) => path_response(&raw, scope_search(&raw)),
        Class::Website { shown, url } => {
            let mut rows = search_rows(&shown, apps, user_search);
            rows.insert(0, site_row(&shown, &url));
            QueryResponse {
                mode: "search".into(),
                rows,
                math: None,
            }
        }
        Class::WebSearch(query) => {
            let mut rows = search_rows(&query, apps, user_search);
            rows.retain(|row| row.kind != "Web");
            rows.insert(0, web_row(&query));
            QueryResponse {
                mode: "search".into(),
                rows,
                math: None,
            }
        }
        Class::Sleep => {
            let mut rows = search_rows("sleep", apps, user_search);
            rows.insert(0, sleep_row());
            QueryResponse {
                mode: "search".into(),
                rows,
                math: None,
            }
        }
        Class::Search(query) => {
            let mut hits = apps.to_vec();
            let exact = rank_hits(&query, &hits, 8).iter().any(|item| item.exact);
            if query.chars().count() >= 3 && !exact {
                hits.extend(user_search(&query).hits);
            }
            QueryResponse {
                mode: "search".into(),
                rows: rows_for_search(&query, &hits),
                math: None,
            }
        }
    };
    finish(response)
}

fn classify_on(input: &str, os: OsKind) -> Class {
    if input.starts_with(|ch: char| ch.is_whitespace()) {
        let query = input.trim();
        if query.is_empty() {
            return Class::Empty;
        }
        return Class::WebSearch(query.to_string());
    }
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Class::Empty;
    }
    if trimmed.eq_ignore_ascii_case("sleep") {
        return Class::Sleep;
    }
    if let Some((prefix, command)) = shell_split(trimmed) {
        let flavor = match (prefix, os) {
            ('>', OsKind::Windows) => ShellFlavor::PowerShell,
            ('$', OsKind::Windows) => ShellFlavor::Bash,
            _ => ShellFlavor::User,
        };
        return Class::Shell { flavor, command };
    }
    if math::looks_like_math(trimmed) {
        match math::parse_math_input(trimmed) {
            Ok(pieces) if !pieces.is_empty() => return Class::Math(pieces),
            Err(err)
                if trimmed.starts_with('=') || has_equation_sign(trimmed) || trimmed.contains("_{") =>
            {
                return Class::MathError(err);
            }
            _ => {}
        }
    }
    if is_path_query(trimmed) {
        return Class::Path(trimmed.to_string());
    }
    if let Some(url) = website_url(trimmed) {
        return Class::Website {
            shown: trimmed.to_string(),
            url,
        };
    }
    Class::Search(trimmed.to_string())
}

fn search_rows(query: &str, apps: &[Hit], user_search: impl Fn(&str) -> SearchOut) -> Vec<Row> {
    let mut hits = apps.to_vec();
    let exact = rank_hits(query, &hits, 8).iter().any(|item| item.exact);
    if query.chars().count() >= 3 && !exact {
        hits.extend(user_search(query).hits);
    }
    rows_for_search(query, &hits)
}

fn website_url(input: &str) -> Option<String> {
    if input.chars().any(|ch| ch.is_whitespace() || ch == '\\' || ch == '@') {
        return None;
    }
    let (bare, rest) = if let Some(rest) = strip_scheme(input, "https://") {
        (false, rest)
    } else if let Some(rest) = strip_scheme(input, "http://") {
        (false, rest)
    } else {
        (true, input)
    };
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = host_without_port(&rest[..authority_end])?;
    if !navigable_host(host) {
        return None;
    }
    if bare {
        Some(format!("https://{input}"))
    } else {
        Some(input.to_string())
    }
}

fn strip_scheme<'a>(input: &'a str, scheme: &str) -> Option<&'a str> {
    input.get(..scheme.len())
        .filter(|prefix| prefix.eq_ignore_ascii_case(scheme))
        .map(|_| &input[scheme.len()..])
}

fn host_without_port(authority: &str) -> Option<&str> {
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (authority, None),
    };
    if host.is_empty() || host.contains(':') {
        return None;
    }
    if let Some(port) = port {
        if port.is_empty() || !port.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
    }
    Some(host)
}

fn navigable_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if is_ipv4(host) {
        return true;
    }
    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() < 2 || labels.iter().any(|label| !valid_dns_label(label)) {
        return false;
    }
    let tld = labels[labels.len() - 1];
    tld.len() >= 2 && tld.chars().all(|ch| ch.is_ascii_alphabetic()) && !is_file_extension(tld)
}

fn valid_dns_label(label: &str) -> bool {
    let bytes = label.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 63
        && bytes.iter().all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
        && bytes[0] != b'-'
        && bytes[bytes.len() - 1] != b'-'
}

fn is_ipv4(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    parts.len() == 4
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.len() <= 3
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && part.parse::<u16>().ok().is_some_and(|value| value <= 255)
                && (part.len() == 1 || !part.starts_with('0'))
        })
}

fn is_file_extension(tld: &str) -> bool {
    matches!(
        tld.to_ascii_lowercase().as_str(),
        "txt"
            | "pdf"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "webp"
            | "svg"
            | "zip"
            | "rar"
            | "7z"
            | "exe"
            | "msi"
            | "lnk"
            | "dll"
            | "doc"
            | "docx"
            | "xls"
            | "xlsx"
            | "ppt"
            | "pptx"
            | "csv"
            | "json"
            | "md"
            | "html"
            | "htm"
            | "css"
            | "js"
            | "ts"
            | "jsx"
            | "tsx"
            | "rs"
            | "py"
            | "go"
            | "java"
            | "c"
            | "cpp"
            | "h"
            | "hpp"
            | "xml"
            | "yml"
            | "yaml"
            | "toml"
            | "log"
            | "mp3"
            | "mp4"
            | "wav"
            | "avi"
            | "mov"
            | "mkv"
            | "iso"
            | "dmg"
            | "sh"
            | "bat"
            | "ps1"
            | "cmd"
    )
}

fn shell_split(input: &str) -> Option<(char, String)> {
    let mut chars = input.chars();
    let prefix = chars.next()?;
    if prefix != '>' && prefix != '$' {
        return None;
    }
    Some((prefix, chars.as_str().trim().to_string()))
}

fn has_equation_sign(input: &str) -> bool {
    let mut depth = 0i32;
    for ch in input.chars() {
        match ch {
            '(' | '{' => depth += 1,
            ')' | '}' => depth -= 1,
            '=' if depth == 0 => return true,
            _ => {}
        }
    }
    false
}

pub fn is_path_query(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.starts_with('~') || trimmed.starts_with('/') || trimmed.starts_with('\\') {
        return true;
    }
    let bytes = trimmed.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn shell_response(flavor: ShellFlavor, command: String, host: &HostCaps) -> QueryResponse {
    if flavor == ShellFlavor::Bash && !host.bash_available {
        return QueryResponse {
            mode: "shell".into(),
            rows: vec![Row {
                id: String::new(),
                kind: "Run".into(),
                title: "Bash was not found".into(),
                subtitle: "Install Git Bash or WSL, or use > for PowerShell".into(),
                action: Action::None,
            }],
            math: None,
        };
    }
    if command.is_empty() {
        let hint = match flavor {
            ShellFlavor::PowerShell => "PowerShell — type a command after >",
            ShellFlavor::Bash => "bash — type a command after $",
            ShellFlavor::User => "Type a command after > or $",
        };
        return QueryResponse {
            mode: "shell".into(),
            rows: vec![Row {
                id: String::new(),
                kind: "Run".into(),
                title: "Type a command".into(),
                subtitle: hint.into(),
                action: Action::None,
            }],
            math: None,
        };
    }
    let flavor_name = flavor.as_str().to_string();
    QueryResponse {
        mode: "shell".into(),
        rows: vec![
            Row {
                id: String::new(),
                kind: "Run".into(),
                title: "Run in shell".into(),
                subtitle: command.clone(),
                action: Action::RunShell {
                    command: command.clone(),
                    flavor: flavor_name.clone(),
                },
            },
            Row {
                id: String::new(),
                kind: "Terminal".into(),
                title: "Open in terminal".into(),
                subtitle: format!("{} · {command}", host.terminal_name),
                action: Action::OpenTerminal {
                    command,
                    flavor: flavor_name,
                },
            },
        ],
        math: None,
    }
}

fn math_response(pieces: Vec<MathPiece>) -> QueryResponse {
    let auto_graph = pieces.iter().any(|piece| piece.uses_x);
    let constants: Vec<_> = pieces.iter().filter(|piece| piece.value.is_some()).collect();
    let large_value = if constants.len() == 1 {
        constants[0].value.map(format_number)
    } else {
        None
    };
    let mut rows = Vec::new();
    for piece in &pieces {
        if let Some(value) = piece.value {
            let value = format_number(value);
            rows.push(Row {
                id: String::new(),
                kind: "Calculate".into(),
                title: value.clone(),
                subtitle: piece.label.clone(),
                action: Action::Copy { text: value },
            });
        } else if !piece.result_plain.is_empty() {
            let title = if piece.solutions.len() > 1 {
                format!("{} solutions", piece.solutions.len())
            } else {
                piece.result_plain.clone()
            };
            let subtitle = if piece.solutions.len() > 1 {
                piece.result_plain.clone()
            } else {
                piece.label.clone()
            };
            rows.push(Row {
                id: String::new(),
                kind: "Solution".into(),
                title,
                subtitle,
                action: Action::Copy {
                    text: piece.result_plain.clone(),
                },
            });
        }
    }
    if pieces.iter().any(|piece| !piece.plots.is_empty()) {
        let labels = pieces
            .iter()
            .filter(|piece| !piece.plots.is_empty())
            .map(|piece| piece.label.as_str())
            .collect::<Vec<_>>()
            .join("  ·  ");
        rows.push(Row {
            id: String::new(),
            kind: "Graph".into(),
            title: "Graph".into(),
            subtitle: labels,
            action: Action::ShowGraph,
        });
    }
    let mut series = Vec::new();
    let mut markers = Vec::new();
    for piece in &pieces {
        for plot in &piece.plots {
            let color = COLORS[series.len() % COLORS.len()].to_string();
            series.push(SeriesMeta {
                label: plot.label.clone(),
                expression: plot.expression.clone(),
                color,
                plot: plot.plot.clone(),
            });
        }
        for marker in &piece.markers {
            markers.push(PlotMarker {
                x: format_number(marker.x),
                y: format_number(marker.y),
            });
        }
    }
    let interpretation_latex = pieces
        .iter()
        .map(|piece| piece.latex.as_str())
        .collect::<Vec<_>>()
        .join(",\\ ");
    let result_parts: Vec<&str> = pieces
        .iter()
        .filter(|piece| !piece.result_latex.is_empty())
        .map(|piece| piece.result_latex.as_str())
        .collect();
    let result_latex = match result_parts.as_slice() {
        [] => None,
        [one] => Some((*one).to_string()),
        many => Some(format!("\\begin{{gathered}} {} \\end{{gathered}}", many.join(" \\\\ "))),
    };
    let solution_rows = pieces.iter().filter(|piece| !piece.solutions.is_empty()).count();
    let result_label = if pieces.iter().any(|piece| piece.result_label == "Solutions") || solution_rows > 1
    {
        Some("Solutions".to_string())
    } else if pieces.iter().any(|piece| piece.result_label == "Solution") {
        Some("Solution".to_string())
    } else if pieces.iter().any(|piece| piece.result_label == "Derivative") {
        Some("Derivative".to_string())
    } else if pieces.iter().any(|piece| piece.result_label == "Integral") {
        Some("Integral".to_string())
    } else if result_latex.is_some() {
        Some("Result".to_string())
    } else {
        None
    };
    let note = pieces.iter().find_map(|piece| piece.note.clone());
    QueryResponse {
        mode: "math".into(),
        rows,
        math: Some(MathPayload {
            large_value,
            auto_graph,
            interpretation_latex,
            result_label,
            result_latex,
            note,
            series,
            markers,
            frame: union_frame(&pieces),
        }),
    }
}

fn union_frame(pieces: &[MathPiece]) -> Option<PlotFrame> {
    let mut combined: Option<(f64, f64, f64, f64)> = None;
    for piece in pieces {
        if !piece.uses_x {
            continue;
        }
        let frame = piece.frame?;
        combined = Some(match combined {
            None => frame,
            Some((x0, x1, y0, y1)) => (x0.min(frame.0), x1.max(frame.1), y0.min(frame.2), y1.max(frame.3)),
        });
    }
    combined.map(|(x0, x1, y0, y1)| PlotFrame {
        x_min: format_number(x0),
        x_max: format_number(x1),
        y_min: format_number(y0),
        y_max: format_number(y1),
    })
}

fn path_response(query: &str, searched: SearchOut) -> QueryResponse {
    if searched.hits.is_empty() {
        return QueryResponse {
            mode: "search".into(),
            rows: vec![
                web_row(query),
                Row {
                    id: String::new(),
                    kind: "Hint".into(),
                    title: "No matching files".into(),
                    subtitle: "Check the path, or press Enter to search the web".into(),
                    action: Action::None,
                },
            ],
            math: None,
        };
    }
    let mut rows: Vec<_> = searched.hits.iter().map(hit_to_row).collect();
    if searched.truncated {
        rows.push(Row {
            id: String::new(),
            kind: "Hint".into(),
            title: "Results capped".into(),
            subtitle: "Type more of the path to narrow the search".into(),
            action: Action::None,
        });
    }
    rows.push(web_row(query));
    QueryResponse {
        mode: "search".into(),
        rows,
        math: None,
    }
}

fn rows_for_search(query: &str, hits: &[Hit]) -> Vec<Row> {
    let ranked = rank_hits(query, hits, 12);
    let mut strong = Vec::new();
    let mut weak = Vec::new();
    for item in ranked {
        if item.strong && strong.len() < 8 {
            strong.push(item);
        } else if !item.strong && weak.len() < 5 {
            weak.push(item);
        }
    }
    let mut rows = Vec::new();
    if strong.is_empty() {
        rows.push(web_row(query));
        rows.extend(weak.into_iter().map(|item| hit_to_row(&item.hit)));
    } else {
        rows.extend(strong.into_iter().map(|item| hit_to_row(&item.hit)));
        rows.push(web_row(query));
    }
    rows
}

fn hit_to_row(hit: &Hit) -> Row {
    let kind = match hit.kind {
        HitKind::App => "App",
        HitKind::Folder => "Folder",
        HitKind::File => "File",
    };
    Row {
        id: String::new(),
        kind: kind.into(),
        title: hit.name.clone(),
        subtitle: if hit.subtitle.is_empty() {
            hit.path.clone()
        } else {
            hit.subtitle.clone()
        },
        action: Action::Open {
            path: hit.path.clone(),
        },
    }
}

fn site_row(shown: &str, url: &str) -> Row {
    Row {
        id: String::new(),
        kind: "Web".into(),
        title: format!("Open {shown}"),
        subtitle: url.to_string(),
        action: Action::Open { path: url.to_string() },
    }
}

fn sleep_row() -> Row {
    Row {
        id: String::new(),
        kind: "Sleep".into(),
        title: "Sleep".into(),
        subtitle: "Put the computer to sleep".into(),
        action: Action::Sleep,
    }
}

fn web_row(query: &str) -> Row {
    Row {
        id: String::new(),
        kind: "Web".into(),
        title: "Search the web".into(),
        subtitle: query.to_string(),
        action: Action::Web {
            query: query.to_string(),
        },
    }
}

fn finish(mut response: QueryResponse) -> QueryResponse {
    for (index, row) in response.rows.iter_mut().enumerate() {
        row.id = format!("r{index}");
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_search() -> SearchOut {
        SearchOut {
            hits: Vec::new(),
            truncated: false,
        }
    }

    fn caps(bash: bool) -> HostCaps {
        HostCaps {
            bash_available: bash,
            terminal_name: "Windows Terminal".into(),
        }
    }

    fn no_files(_: &str) -> SearchOut {
        empty_search()
    }

    #[test]
    fn routes_in_priority_order() {
        assert!(matches!(classify_on("   ", OsKind::Windows), Class::Empty));
        match classify_on("> Get-Process", OsKind::Windows) {
            Class::Shell { flavor, command } => {
                assert_eq!(flavor, ShellFlavor::PowerShell);
                assert_eq!(command, "Get-Process");
            }
            other => panic!("{other:?}"),
        }
        match classify_on("$ ls", OsKind::Windows) {
            Class::Shell { flavor, command } => {
                assert_eq!(flavor, ShellFlavor::Bash);
                assert_eq!(command, "ls");
            }
            other => panic!("{other:?}"),
        }
        match classify_on("> ls", OsKind::Unix) {
            Class::Shell { flavor, .. } => assert_eq!(flavor, ShellFlavor::User),
            other => panic!("{other:?}"),
        }
        match classify_on("=2+2", OsKind::Windows) {
            Class::Math(pieces) => assert_eq!(pieces[0].value.unwrap() as i64, 4),
            other => panic!("{other:?}"),
        }
        match classify_on("sin(pi/2)", OsKind::Unix) {
            Class::Math(pieces) => {
                assert!((pieces[0].value.unwrap() - 1.0).abs() < 1e-9);
                assert!(!pieces[0].uses_x);
            }
            other => panic!("{other:?}"),
        }
        match classify_on("y = x^2", OsKind::Windows) {
            Class::Math(pieces) => {
                assert!(pieces[0].uses_x);
                assert_eq!(pieces[0].expression, "x^2");
            }
            other => panic!("{other:?}"),
        }
        match classify_on("x^2; sin(x)", OsKind::Windows) {
            Class::Math(pieces) => {
                assert_eq!(pieces.len(), 2);
                assert!(pieces.iter().all(|piece| piece.uses_x));
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(classify_on("=2+", OsKind::Windows), Class::MathError(_)));
        for sample in ["c:", "d:\\", "e:", "c:\\Windows", "~", "/usr/bin"] {
            assert!(
                matches!(classify_on(sample, OsKind::Windows), Class::Path(_)),
                "{sample}"
            );
        }
        assert!(matches!(classify_on("e", OsKind::Windows), Class::Math(_)));
        assert!(matches!(classify_on("sleep", OsKind::Windows), Class::Sleep));
        assert!(matches!(classify_on("SLEEP", OsKind::Unix), Class::Sleep));
        assert!(matches!(classify_on(" sleep", OsKind::Windows), Class::WebSearch(_)));
        assert!(matches!(classify_on("sleeper", OsKind::Windows), Class::Search(_)));
        assert!(matches!(classify_on("chrome", OsKind::Windows), Class::Search(_)));
        assert!(matches!(classify_on("hello world", OsKind::Unix), Class::Search(_)));
    }

    #[test]
    fn sleep_is_the_first_row() {
        let response = respond_on("Sleep", &[], &caps(true), OsKind::Windows, no_files, no_files);
        assert_eq!(response.rows[0].kind, "Sleep");
        assert_eq!(response.rows[0].title, "Sleep");
        assert!(matches!(response.rows[0].action, Action::Sleep));
    }

    #[test]
    fn shell_requires_a_visible_command_and_bash_must_exist() {
        let missing = respond_on("$ ls", &[], &caps(false), OsKind::Windows, no_files, no_files);
        assert_eq!(missing.rows[0].title, "Bash was not found");
        assert!(matches!(missing.rows[0].action, Action::None));

        let ready = respond_on("> echo hi", &[], &caps(true), OsKind::Windows, no_files, no_files);
        assert_eq!(ready.rows[0].kind, "Run");
        assert_eq!(ready.rows[0].subtitle, "echo hi");
        assert_eq!(ready.rows[1].kind, "Terminal");
        match &ready.rows[0].action {
            Action::RunShell { command, flavor } => {
                assert_eq!(command, "echo hi");
                assert_eq!(flavor, "powershell");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn math_response_shows_a_value_and_a_graph_series() {
        let response = respond("sqrt(9)", &[], &caps(true), no_files, no_files);
        assert_eq!(response.mode, "math");
        assert_eq!(response.math.as_ref().unwrap().large_value.as_deref(), Some("3"));
        assert!(!response.math.as_ref().unwrap().auto_graph);
        assert_eq!(response.rows[0].kind, "Calculate");

        let graph = respond("sin(x)", &[], &caps(true), no_files, no_files);
        assert!(graph.math.as_ref().unwrap().auto_graph);
        assert_eq!(graph.rows[0].kind, "Graph");
        assert_eq!(graph.math.as_ref().unwrap().series[0].expression, "sin(x)");
        assert!(graph.math.as_ref().unwrap().interpretation_latex.contains("\\sin"));

        let solved = respond("x=3", &[], &caps(true), no_files, no_files);
        let solved_math = solved.math.as_ref().unwrap();
        assert!(solved_math.auto_graph);
        assert_eq!(solved_math.series[0].plot, "vertical");
        assert_eq!(solved_math.series[0].expression, "3");
        assert_eq!(solved_math.result_latex.as_deref(), Some("x = 3"));
        assert_eq!(solved.rows[0].kind, "Solution");
        assert_eq!(solved.rows[0].title, "x = 3");

        let mixed = respond("2+2; y=x", &[], &caps(true), no_files, no_files);
        assert_eq!(mixed.math.as_ref().unwrap().large_value.as_deref(), Some("4"));
        assert!(mixed.math.as_ref().unwrap().auto_graph);
        assert_eq!(mixed.math.as_ref().unwrap().series.len(), 1);
        assert_eq!(mixed.math.as_ref().unwrap().series[0].expression, "x");
    }

    #[test]
    fn web_is_primary_only_without_a_strong_local_hit() {
        let apps = vec![Hit {
            name: "Chrome".into(),
            path: "C:\\Chrome.lnk".into(),
            subtitle: "C:\\Program Files\\Chrome\\chrome.exe".into(),
            kind: HitKind::App,
        }];
        let found = respond("chrome", &apps, &caps(true), no_files, no_files);
        assert_eq!(found.rows[0].kind, "App");
        assert_eq!(found.rows.last().unwrap().kind, "Web");

        let missing = respond("zzzz-not-a-thing", &apps, &caps(true), no_files, no_files);
        assert_eq!(missing.rows[0].kind, "Web");
        assert_eq!(missing.rows[0].title, "Search the web");
    }

    #[test]
    fn a_domain_opens_and_a_leading_space_searches() {
        let site = respond("google.com", &[], &caps(true), no_files, no_files);
        assert_eq!(site.rows[0].title, "Open google.com");
        assert_eq!(
            site.rows[0].action,
            Action::Open {
                path: "https://google.com".into()
            }
        );

        let explicit = respond("https://example.com/docs", &[], &caps(true), no_files, no_files);
        assert_eq!(
            explicit.rows[0].action,
            Action::Open {
                path: "https://example.com/docs".into()
            }
        );

        let apps = vec![Hit {
            name: "Chrome".into(),
            path: "C:\\Chrome.lnk".into(),
            subtitle: String::new(),
            kind: HitKind::App,
        }];
        let search = respond(" google.com", &apps, &caps(true), no_files, no_files);
        assert_eq!(search.rows[0].title, "Search the web");
        assert_eq!(search.rows[0].subtitle, "google.com");
        assert_eq!(
            search.rows[0].action,
            Action::Web {
                query: "google.com".into()
            }
        );
        assert!(search.rows.iter().filter(|row| row.kind == "Web").count() == 1);

        let file = respond("notes.txt", &[], &caps(true), no_files, no_files);
        assert_eq!(file.rows[0].title, "Search the web");
    }

    #[test]
    fn path_results_come_from_the_scope_search() {
        let response = respond_on(
            "d:\\games",
            &[],
            &caps(true),
            OsKind::Windows,
            |query| {
                assert_eq!(query, "d:\\games");
                SearchOut {
                    hits: vec![Hit {
                        name: "Games".into(),
                        path: "D:\\Games".into(),
                        subtitle: "D:\\Games".into(),
                        kind: HitKind::Folder,
                    }],
                    truncated: false,
                }
            },
            |_| panic!("user file search should not run for a path query"),
        );
        assert_eq!(response.rows[0].kind, "Folder");
        assert_eq!(response.rows[0].title, "Games");
        assert_eq!(response.rows.last().unwrap().kind, "Web");
    }
}
