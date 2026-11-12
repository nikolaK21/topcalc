mod math;
mod rank;
mod route;
mod search;
mod settings;
mod shell;
mod util;

use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::webview::Color;
use tauri::{
    image::Image, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::math::{present_value, sample_expression_source, sample_job};
use crate::rank::Hit;
use crate::route::{respond, Action, QueryResponse};
use crate::search::{scan_apps, search_scope, search_user_files};
use crate::settings::{
    build_search_url, load_settings, normalize, save_settings_file, window_origin, Settings, ThemeColors,
};
use crate::shell::{
    current_host, open_in_terminal, open_target, present_output, run_inline, suspend_computer, HostCaps,
    ShellFlavor,
};

struct AppState {
    apps: Mutex<Vec<Hit>>,
    settings: Mutex<Settings>,
    hotkey_error: Mutex<Option<String>>,
    desk_lines: Mutex<Vec<DeskLine>>,
    host: HostCaps,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeskLine {
    expression: String,
    commit: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsResponse {
    hotkey: String,
    search_url: String,
    theme: String,
    colors: ThemeColors,
    anchor: String,
    x_percent: u8,
    y_percent: u8,
    hotkey_error: Option<String>,
    config_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ActivateResponse {
    output: Option<String>,
    hide: bool,
    timed_out: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Binding {
    name: String,
    value: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SampleJob {
    expression: String,
    #[serde(default)]
    plot: String,
    #[serde(default)]
    domain: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SampleRequest {
    #[serde(default)]
    expressions: Vec<String>,
    #[serde(default)]
    jobs: Vec<SampleJob>,
    #[serde(default)]
    bindings: Vec<Binding>,
    x_min: f64,
    x_max: f64,
    #[serde(default)]
    y_min: Option<f64>,
    #[serde(default)]
    y_max: Option<f64>,
    samples: usize,
    #[serde(default)]
    degrees: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SampledSeries {
    expression: String,
    lines: Vec<Vec<[f64; 2]>>,
}

#[derive(Serialize)]
struct SampleResponse {
    series: Vec<SampledSeries>,
}

fn restyle_numbers(response: &mut QueryResponse, input: &str, mode: &str) {
    if mode.is_empty() || mode == "decimal" {
        return;
    }
    let graphs = response.math.as_ref().is_some_and(|math| math.auto_graph);
    if graphs {
        return;
    }
    let source = input.trim().trim_start_matches('=').trim();
    let Ok(shown) = present_value(source, &[], false, mode) else {
        return;
    };
    if let Some(math) = response.math.as_mut() {
        math.large_value = Some(shown.clone());
        math.result_latex = Some(format!(
            "\\text{{{}}}",
            shown.replace('\\', "").replace('{', "").replace('}', "")
        ));
    }
    if let Some(row) = response.rows.iter_mut().find(|row| row.kind == "Calculate") {
        row.title = shown.clone();
        if let Action::Copy { text } = &mut row.action {
            *text = shown;
        }
    }
}

#[tauri::command]
async fn query(
    state: tauri::State<'_, AppState>,
    input: String,
    mode: Option<String>,
) -> Result<QueryResponse, String> {
    let apps = state
        .apps
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clone();
    let host = state.host.clone();
    let mode = mode.unwrap_or_else(|| "decimal".into());
    tauri::async_runtime::spawn_blocking(move || {
        let mut response = respond(&input, &apps, &host, search_scope, search_user_files);
        restyle_numbers(&mut response, &input, &mode);
        response
    })
    .await
    .map_err(|err| err.to_string())
}

#[tauri::command]
fn sample_graph(request: SampleRequest) -> SampleResponse {
    let samples = request.samples.clamp(32, 1200);
    let x_min = request.x_min;
    let x_max = request.x_max;
    let y_min = request.y_min.unwrap_or(-6.0);
    let y_max = request.y_max.unwrap_or(6.0);
    let plain = request.jobs.is_empty();
    let owned = request
        .bindings
        .iter()
        .map(|binding| (binding.name.to_ascii_lowercase(), binding.value))
        .collect::<Vec<_>>();
    let bindings = owned
        .iter()
        .map(|(name, value)| (name.as_str(), *value))
        .collect::<Vec<_>>();
    let jobs = if plain {
        request
            .expressions
            .into_iter()
            .map(|expression| SampleJob {
                expression,
                plot: "function".into(),
                domain: None,
            })
            .collect::<Vec<_>>()
    } else {
        request.jobs
    };
    let series = jobs
        .into_iter()
        .map(|job| {
            let lines = if job.plot == "vertical" {
                Vec::new()
            } else if plain && job.plot == "function" && bindings.is_empty() && !request.degrees {
                sample_expression_source(&job.expression, x_min, x_max, samples)
            } else {
                sample_job(
                    &job.plot,
                    &job.expression,
                    job.domain.as_deref(),
                    &bindings,
                    x_min,
                    x_max,
                    y_min,
                    y_max,
                    samples,
                    request.degrees,
                )
            };
            SampledSeries {
                expression: job.expression,
                lines,
            }
        })
        .collect();
    SampleResponse { series }
}

#[tauri::command]
fn eval_graph(
    expression: String,
    bindings: Vec<Binding>,
    degrees: bool,
    mode: Option<String>,
) -> Result<String, String> {
    let pairs = bindings
        .iter()
        .map(|binding| (binding.name.as_str(), binding.value))
        .collect::<Vec<_>>();
    let mode = mode.unwrap_or_else(|| "decimal".into());
    present_value(&expression, &pairs, degrees, &mode)
}

#[tauri::command]
async fn activate(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    action: Action,
) -> Result<ActivateResponse, String> {
    match action {
        Action::None | Action::ShowGraph => Ok(ActivateResponse {
            output: None,
            hide: false,
            timed_out: false,
        }),
        Action::Copy { text } => Ok(ActivateResponse {
            output: Some(text),
            hide: false,
            timed_out: false,
        }),
        Action::Open { path } => {
            open_target(&path)?;
            Ok(ActivateResponse {
                output: None,
                hide: true,
                timed_out: false,
            })
        }
        Action::Web { query } => {
            let template = state
                .settings
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .search_url
                .clone();
            let url = build_search_url(&template, &query)?;
            open_target(&url)?;
            Ok(ActivateResponse {
                output: None,
                hide: true,
                timed_out: false,
            })
        }
        Action::RunShell { command, flavor } => {
            let flavor = ShellFlavor::parse(&flavor).ok_or_else(|| format!("Unknown shell '{flavor}'"))?;
            let shown = command.clone();
            let output = tauri::async_runtime::spawn_blocking(move || {
                run_inline(&command, flavor, Duration::from_secs(8))
            })
            .await
            .map_err(|err| err.to_string())??;
            Ok(ActivateResponse {
                timed_out: output.timed_out,
                output: Some(present_output(&shown, &output)),
                hide: false,
            })
        }
        Action::OpenTerminal { command, flavor } => {
            let flavor = ShellFlavor::parse(&flavor).ok_or_else(|| format!("Unknown shell '{flavor}'"))?;
            open_in_terminal(&command, flavor)?;
            let _ = app;
            Ok(ActivateResponse {
                output: None,
                hide: true,
                timed_out: false,
            })
        }
        Action::Sleep => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
            let slept = tauri::async_runtime::spawn_blocking(suspend_computer)
                .await
                .map_err(|err| err.to_string())?;
            if let Err(err) = slept {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                return Err(err);
            }
            Ok(ActivateResponse {
                output: None,
                hide: true,
                timed_out: false,
            })
        }
    }
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<SettingsResponse, String> {
    let path = config_file(&app)?;
    Ok(settings_view(&state, &path))
}

#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    settings: Settings,
) -> Result<SettingsResponse, String> {
    if settings.hotkey.trim().is_empty() {
        return Err("Enter a hotkey".into());
    }
    let settings = normalize(settings);
    build_search_url(&settings.search_url, "")?;
    let path = config_file(&app)?;
    let previous = state
        .settings
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clone();
    if let Err(err) = register_hotkey(&app, &settings.hotkey) {
        let _ = register_hotkey(&app, &previous.hotkey);
        return Err(err);
    }
    if let Err(err) = save_settings_file(&path, &settings) {
        let _ = register_hotkey(&app, &previous.hotkey);
        return Err(err);
    }
    *state.settings.lock().unwrap_or_else(|err| err.into_inner()) = settings;
    *state.hotkey_error.lock().unwrap_or_else(|err| err.into_inner()) = None;
    Ok(settings_view(&state, &path))
}

#[tauri::command]
fn suspend_hotkey(app: tauri::AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn resume_hotkey(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let hotkey = state
        .settings
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .hotkey
        .clone();
    register_hotkey(&app, &hotkey)
}

fn desk_page(app: &tauri::AppHandle) -> Option<tauri::Url> {
    let main = app.get_webview_window("main")?;
    let mut url = main.url().ok()?;
    if url.as_str() == "about:blank" {
        return None;
    }
    url.set_path("/index.html");
    url.set_query(Some("desk=1"));
    Some(url)
}

fn ensure_desk_page(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    let current = window.url().ok();
    let ready = current.as_ref().is_some_and(|url| {
        url.as_str() != "about:blank" && url.query().unwrap_or("").contains("desk=1")
    });
    if ready {
        return;
    }
    if let Some(url) = desk_page(app) {
        let _ = window.navigate(url);
    }
}

#[tauri::command]
fn open_desk(app: tauri::AppHandle, expression: String, commit: bool) -> Result<(), String> {
    let expression = expression.trim().to_string();
    if !expression.is_empty() {
        app.state::<AppState>()
            .desk_lines
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .push(DeskLine { expression, commit });
    }
    let window = if let Some(window) = app.get_webview_window("desk") {
        window
    } else {
        WebviewWindowBuilder::new(&app, "desk", WebviewUrl::App("index.html?desk=1".into()))
            .title("TopCalc Graph")
            .inner_size(1080.0, 700.0)
            .min_inner_size(760.0, 480.0)
            .decorations(true)
            .resizable(true)
            .transparent(false)
            .background_color(Color(28, 28, 32, 255))
            .skip_taskbar(false)
            .always_on_top(false)
            .center()
            .focused(false)
            .build()
            .map_err(|err| err.to_string())?
    };
    ensure_desk_page(&app, &window);
    let visible = window.is_visible().unwrap_or(false);
    if !visible {
        let _ = window.set_size(tauri::LogicalSize::new(1080.0, 700.0));
        let _ = window.center();
        let _ = window.show();
        let _ = window.unminimize();
    }
    let _ = window.set_focus();
    let _ = window.emit("desk-refresh", ());
    Ok(())
}

#[tauri::command]
fn take_desk_lines(state: tauri::State<'_, AppState>) -> Vec<DeskLine> {
    std::mem::take(
        &mut *state
            .desk_lines
            .lock()
            .unwrap_or_else(|err| err.into_inner()),
    )
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn quit(app: tauri::AppHandle) {
    app.exit(0);
}

fn settings_view(state: &AppState, path: &std::path::Path) -> SettingsResponse {
    let settings = state
        .settings
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clone();
    let hotkey_error = state
        .hotkey_error
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clone();
    SettingsResponse {
        hotkey: settings.hotkey,
        search_url: settings.search_url,
        theme: settings.theme,
        colors: settings.colors,
        anchor: settings.anchor,
        x_percent: settings.x_percent,
        y_percent: settings.y_percent,
        hotkey_error,
        config_path: path.display().to_string(),
    }
}

fn config_file(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|err| err.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    Ok(dir.join("settings.json"))
}

fn register_hotkey(app: &tauri::AppHandle, hotkey: &str) -> Result<(), String> {
    let shortcut: Shortcut = hotkey.parse().map_err(|err| {
        format!("Could not parse hotkey '{hotkey}': {err}. Try Alt+Space or Ctrl+Alt+Space.")
    })?;
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();
    shortcuts
        .register(shortcut)
        .map_err(|err| format!("Could not register '{hotkey}'. It may already be in use. {err}"))
}

fn reveal(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let (anchor, x_percent, y_percent) = app
            .try_state::<AppState>()
            .map(|state| {
                let settings = state
                    .settings
                    .lock()
                    .unwrap_or_else(|err| err.into_inner())
                    .clone();
                (settings.anchor, settings.x_percent, settings.y_percent)
            })
            .unwrap_or_else(|| ("top".into(), 50, 12));
        position_window(&window, &anchor, x_percent, y_percent);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        let _ = window.emit("shown", ());
    }
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let Ok(apps) = tauri::async_runtime::spawn_blocking(scan_apps).await else {
            return;
        };
        if let Some(state) = handle.try_state::<AppState>() {
            *state.apps.lock().unwrap_or_else(|err| err.into_inner()) = apps;
        }
    });
}

fn toggle_window(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        reveal(app);
    }
}

#[tauri::command]
fn place_window(app: tauri::AppHandle, anchor: String, x_percent: u8, y_percent: u8) {
    if let Some(window) = app.get_webview_window("main") {
        position_window(&window, &anchor, x_percent.min(100), y_percent.min(100));
    }
}

fn position_window(window: &tauri::WebviewWindow, anchor: &str, x_percent: u8, y_percent: u8) {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return;
    };
    let size = monitor.size();
    let origin = monitor.position();
    let win = window.outer_size().unwrap_or(PhysicalSize::new(640, 120));
    let (x, y) = window_origin(
        origin.x,
        origin.y,
        size.width as i32,
        size.height as i32,
        win.width as i32,
        win.height as i32,
        anchor,
        x_percent,
        y_percent,
    );
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

fn install_close_to_hide(app: &tauri::AppHandle, label: &str) {
    let Some(window) = app.get_webview_window(label) else {
        return;
    };
    let hide_target = window.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = hide_target.hide();
        }
    });
}

fn setup_tray(app: &tauri::AppHandle) -> Result<(), String> {
    let show = MenuItem::with_id(app, "show", "Show TopCalc", true, None::<&str>)
        .map_err(|err| err.to_string())?;
    let quit = MenuItem::with_id(app, "quit", "Quit TopCalc", true, None::<&str>)
        .map_err(|err| err.to_string())?;
    let menu = Menu::with_items(app, &[&show, &quit]).map_err(|err| err.to_string())?;
    let icon = tray_icon();
    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("TopCalc")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => reveal(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                reveal(tray.app_handle());
            }
        })
        .build(app)
        .map_err(|err| err.to_string())?;
    Ok(())
}

fn tray_icon() -> Image<'static> {
    let size = 32u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let index = ((y * size + x) * 4) as usize;
            let dx = x as i32 - 16;
            let dy = y as i32 - 16;
            if dx * dx + dy * dy > 14 * 14 {
                continue;
            }
            rgba[index] = 22;
            rgba[index + 1] = 24;
            rgba[index + 2] = 31;
            rgba[index + 3] = 255;
            let wave = ((x as f64 / 31.0) * std::f64::consts::PI * 2.0).sin();
            let wave_y = (16.0 - wave * 7.0).round() as i32;
            if (y as i32 - wave_y).abs() <= 1 {
                rgba[index] = 110;
                rgba[index + 1] = 168;
                rgba[index + 2] = 254;
            }
        }
    }
    Image::new_owned(rgba, size, size)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            reveal(&app);
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        toggle_window(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let path = config_file(app.handle()).unwrap_or_else(|_| {
                std::env::temp_dir().join("topcalc-settings.json")
            });
            let settings = load_settings(&path);
            let host = current_host();
            let apps = scan_apps();
            app.manage(AppState {
                apps: Mutex::new(apps),
                settings: Mutex::new(settings.clone()),
                hotkey_error: Mutex::new(None),
                desk_lines: Mutex::new(Vec::new()),
                host,
            });
            install_close_to_hide(app.handle(), "main");
            install_close_to_hide(app.handle(), "desk");
            if let Err(err) = setup_tray(app.handle()) {
                eprintln!("TopCalc tray icon: {err}");
            }
            if let Err(err) = register_hotkey(app.handle(), &settings.hotkey) {
                eprintln!("TopCalc hotkey: {err}");
                if let Some(state) = app.try_state::<AppState>() {
                    *state.hotkey_error.lock().unwrap_or_else(|err| err.into_inner()) = Some(err);
                }
                reveal(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            query,
            sample_graph,
            eval_graph,
            activate,
            get_settings,
            save_settings,
            place_window,
            open_desk,
            take_desk_lines,
            suspend_hotkey,
            resume_hotkey,
            hide_window,
            quit
        ])
        .run(tauri::generate_context!())
        .expect("TopCalc failed to start");
}
