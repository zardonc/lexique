use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

use image::{DynamicImage, RgbaImage};
use serde::Deserialize;
use tauri::Manager;

use crate::{
    ocr,
    selection::{self, SelectionError, SelectionEvent},
};

const MAX_SCREEN_SIDE: u32 = 8192;
const MAX_SCREEN_PIXELS: u64 = 64_000_000;

struct Screen {
    label: String,
    bitmap: RgbaImage,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    primary: bool,
}

struct Capture {
    request: u64,
    screens: Vec<Screen>,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptureLocale {
    #[default]
    Zh,
    En,
    Fr,
}

impl CaptureLocale {
    fn tag(self) -> &'static str {
        match self {
            Self::Zh => "zh",
            Self::En => "en",
            Self::Fr => "fr",
        }
    }
    fn title(self) -> &'static str {
        match self {
            Self::Zh => "Lexique — 截图框选",
            Self::En => "Lexique — screen selection",
            Self::Fr => "Lexique — sélection de l’écran",
        }
    }
}

#[derive(Default)]
pub struct CaptureState {
    capture: Mutex<Option<Capture>>,
    busy: AtomicBool,
    locale: Mutex<CaptureLocale>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn close_windows(app: &tauri::AppHandle, request: Option<u64>) -> Result<(), String> {
    let prefix = request.map_or_else(|| "ocr-".into(), |value| format!("ocr-{value}-"));
    let errors: Vec<_> = app
        .webview_windows()
        .into_values()
        .filter(|window| window.label().starts_with(&prefix))
        .filter_map(|window| {
            window
                .destroy()
                .err()
                .map(|error| format!("Could not close capture window: {error}"))
        })
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

pub fn clear(app: &tauri::AppHandle) -> Result<(), String> {
    app.state::<CaptureState>()
        .capture
        .lock()
        .map_err(|error| format!("Capture state unavailable: {error}"))?
        .take();
    close_windows(app, None)
}

#[cfg(target_os = "macos")]
fn permission() -> Result<(), SelectionError> {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
        fn CGRequestScreenCaptureAccess() -> bool;
    }
    if unsafe { CGPreflightScreenCaptureAccess() || CGRequestScreenCaptureAccess() } {
        Ok(())
    } else {
        Err(SelectionError::new(
            "screen_permission",
            "Screen recording permission was refused. Allow Lexique in Privacy & Security, then retry.",
        ))
    }
}

#[cfg(not(target_os = "macos"))]
fn permission() -> Result<(), SelectionError> {
    Ok(())
}

#[cfg(any(windows, target_os = "macos"))]
fn capture(request: u64) -> Result<Capture, SelectionError> {
    let failure =
        |error| SelectionError::new("capture_failed", format!("Screen provider failed: {error}"));
    let monitors = xcap::Monitor::all().map_err(failure)?;
    if monitors.is_empty() {
        return Err(SelectionError::new(
            "capture_failed",
            "No screens available",
        ));
    }
    let mut expected_pixels = 0_u64;
    for monitor in &monitors {
        #[cfg(windows)]
        let scale = 1.0_f64;
        #[cfg(target_os = "macos")]
        let scale = f64::from(monitor.scale_factor().map_err(failure)?);
        let width = (f64::from(monitor.width().map_err(failure)?) * scale).round() as u32;
        let height = (f64::from(monitor.height().map_err(failure)?) * scale).round() as u32;
        if width == 0 || height == 0 || width > MAX_SCREEN_SIDE || height > MAX_SCREEN_SIDE {
            return Err(SelectionError::new(
                "image_limit",
                "Screen snapshots require at most 8192 pixels per side",
            ));
        }
        expected_pixels += u64::from(width) * u64::from(height);
        if expected_pixels > MAX_SCREEN_PIXELS {
            return Err(SelectionError::new(
                "image_limit",
                "Screen snapshots exceed 64 million pixels",
            ));
        }
    }
    let mut screens = Vec::new();
    let mut actual_pixels = 0_u64;
    for (index, monitor) in monitors.into_iter().enumerate() {
        let bitmap = monitor.capture_image().map_err(failure)?;
        actual_pixels += u64::from(bitmap.width()) * u64::from(bitmap.height());
        if bitmap.width() == 0
            || bitmap.height() == 0
            || bitmap.width() > MAX_SCREEN_SIDE
            || bitmap.height() > MAX_SCREEN_SIDE
            || actual_pixels > MAX_SCREEN_PIXELS
        {
            return Err(SelectionError::new(
                "image_limit",
                "Screen snapshot dimensions exceed the supported limits",
            ));
        }
        screens.push(Screen {
            label: format!("ocr-{request}-{index}"),
            bitmap,
            x: monitor.x().map_err(failure)?,
            y: monitor.y().map_err(failure)?,
            width: monitor.width().map_err(failure)?,
            height: monitor.height().map_err(failure)?,
            primary: monitor.is_primary().map_err(failure)?,
        });
    }
    Ok(Capture { request, screens })
}

#[cfg(not(any(windows, target_os = "macos")))]
fn capture(_request: u64) -> Result<Capture, SelectionError> {
    Err(SelectionError::new(
        "unsupported",
        "Screen capture requires Windows or macOS",
    ))
}

fn show(app: &tauri::AppHandle, snapshot: Capture) -> Result<(), String> {
    let request = snapshot.request;
    let locale = *app
        .state::<CaptureState>()
        .locale
        .lock()
        .map_err(|error| format!("Capture locale unavailable: {error}"))?;
    let placements: Vec<_> = snapshot
        .screens
        .iter()
        .map(|screen| {
            (
                screen.label.clone(),
                screen.x,
                screen.y,
                screen.width,
                screen.height,
                screen.primary,
            )
        })
        .collect();
    {
        let state = app.state::<CaptureState>();
        let mut current = state
            .capture
            .lock()
            .map_err(|error| format!("Capture state unavailable: {error}"))?;
        if !selection::current(app, request) {
            return Ok(());
        }
        *current = Some(snapshot);
    }
    let mut focus = None;
    for (label, x, y, width, height, primary) in placements {
        if !selection::current(app, request) {
            return Ok(());
        }
        let builder = tauri::WebviewWindowBuilder::new(
            app,
            &label,
            tauri::WebviewUrl::App(
                format!("index.html?capture={request}&locale={}", locale.tag()).into(),
            ),
        )
        .title(locale.title())
        .decorations(false)
        .shadow(false)
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .visible(false);
        #[cfg(target_os = "macos")]
        let builder = builder
            .position(f64::from(x), f64::from(y))
            .inner_size(f64::from(width), f64::from(height));
        let window = builder
            .build()
            .map_err(|error| format!("Could not create capture window: {error}"))?;
        if !selection::current(app, request) {
            window.destroy().map_err(|error| error.to_string())?;
            return Ok(());
        }
        #[cfg(not(target_os = "macos"))]
        {
            window
                .set_position(tauri::PhysicalPosition::new(x, y))
                .map_err(|error| error.to_string())?;
            window
                .set_size(tauri::PhysicalSize::new(width, height))
                .map_err(|error| error.to_string())?;
        }
        window
            .show()
            .map_err(|error| format!("Could not show capture window: {error}"))?;
        if primary {
            focus = Some(window);
        }
    }
    if let Some(window) = focus {
        window
            .set_focus()
            .map_err(|error| format!("Could not focus capture window: {error}"))?;
    }
    Ok(())
}

pub fn start(app: &tauri::AppHandle) -> Result<(), String> {
    let request = selection::begin(app)?;
    if let Err(error) = permission() {
        return selection::finish(app, request, Err(error));
    }
    if app
        .state::<CaptureState>()
        .busy
        .swap(true, Ordering::SeqCst)
    {
        return selection::finish(
            app,
            request,
            Err(SelectionError::new(
                "busy",
                "The previous screen snapshot is still running",
            )),
        );
    }
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window.hide() {
            app.state::<CaptureState>()
                .busy
                .store(false, Ordering::SeqCst);
            return selection::finish(
                app,
                request,
                Err(SelectionError::new(
                    "capture_failed",
                    format!("Could not hide Lexique for capture: {error}"),
                )),
            );
        }
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let result = tauri::async_runtime::spawn_blocking(move || {
            std::thread::sleep(std::time::Duration::from_millis(150));
            capture(request)
        })
        .await
        .unwrap_or_else(|error| {
            Err(SelectionError::new(
                "capture_failed",
                format!("Screen snapshot task failed: {error}"),
            ))
        });
        app.state::<CaptureState>()
            .busy
            .store(false, Ordering::SeqCst);
        if !selection::current(&app, request) {
            return;
        }
        let result = match result {
            Ok(snapshot) => show(&app, snapshot),
            Err(error) => selection::finish(&app, request, Err(error)),
        };
        if let Err(error) = result {
            if selection::current(&app, request) {
                if let Err(cleanup) = clear(&app) {
                    eprintln!("{cleanup}");
                }
                if let Err(delivery) = selection::finish(
                    &app,
                    request,
                    Err(SelectionError::new("capture_failed", error)),
                ) {
                    eprintln!("{delivery}");
                }
            }
        }
    });
    Ok(())
}

pub fn abort(app: &tauri::AppHandle, request: u64) -> Result<(), String> {
    if !selection::current(app, request) {
        return Ok(());
    }
    clear(app)?;
    let result = selection::deliver(app, SelectionEvent::Cancelled { request });
    selection::invalidate_request(app, request);
    result
}

#[tauri::command]
pub fn ocr_locale(app: tauri::AppHandle, locale: CaptureLocale) -> Result<(), String> {
    *app.state::<CaptureState>()
        .locale
        .lock()
        .map_err(|error| format!("Capture locale unavailable: {error}"))? = locale;
    Ok(())
}

#[tauri::command]
pub fn ocr_capture(app: tauri::AppHandle) -> Result<(), String> {
    start(&app)
}

#[tauri::command]
pub fn ocr_abort(app: tauri::AppHandle, request: u64) -> Result<(), String> {
    abort(&app, request)
}

#[tauri::command]
pub async fn ocr_screen(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    request: u64,
) -> Result<ocr::OcrImage, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let bitmap = {
            let state = app.state::<CaptureState>();
            let capture = state
                .capture
                .lock()
                .map_err(|error| format!("Capture state unavailable: {error}"))?;
            let capture = capture
                .as_ref()
                .filter(|capture| capture.request == request)
                .ok_or("Capture expired")?;
            capture
                .screens
                .iter()
                .find(|screen| screen.label == window.label())
                .ok_or("Screen does not belong to this window")?
                .bitmap
                .clone()
        };
        ocr::encode(
            DynamicImage::ImageRgba8(bitmap),
            MAX_SCREEN_SIDE,
            64 * 1024 * 1024,
        )
        .map_err(|error| format!("{}: {}", error.code, error.message))
    })
    .await
    .map_err(|error| format!("Screen preview task failed: {error}"))?
}

fn crop(bitmap: &RgbaImage, region: &Region) -> Result<RgbaImage, String> {
    if region.width == 0
        || region.height == 0
        || region.width > ocr::MAX_SIDE
        || region.height > ocr::MAX_SIDE
        || region
            .x
            .checked_add(region.width)
            .is_none_or(|right| right > bitmap.width())
        || region
            .y
            .checked_add(region.height)
            .is_none_or(|bottom| bottom > bitmap.height())
    {
        return Err("Select a region inside this screen, at most 4096×4096 pixels".into());
    }
    Ok(
        image::imageops::crop_imm(bitmap, region.x, region.y, region.width, region.height)
            .to_image(),
    )
}

#[tauri::command]
pub async fn ocr_region(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    request: u64,
    region: Region,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let bitmap = {
            let state = app.state::<CaptureState>();
            let mut current = state
                .capture
                .lock()
                .map_err(|error| format!("Capture state unavailable: {error}"))?;
            let capture = current
                .as_ref()
                .filter(|capture| capture.request == request)
                .ok_or("Capture expired")?;
            let screen = capture
                .screens
                .iter()
                .find(|screen| screen.label == window.label())
                .ok_or("Screen does not belong to this window")?;
            let bitmap = crop(&screen.bitmap, &region)?;
            current.take();
            bitmap
        };
        close_windows(&app, Some(request))?;
        ocr::finish(
            &app,
            request,
            ocr::encode(
                DynamicImage::ImageRgba8(bitmap),
                ocr::MAX_SIDE,
                ocr::MAX_BYTES,
            ),
            true,
        )
    })
    .await
    .map_err(|error| format!("Screen region task failed: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_boundary_preserves_selected_pixels_and_rejects_invalid_bounds() {
        let mut bitmap = RgbaImage::new(8, 6);
        bitmap.put_pixel(3, 2, image::Rgba([12, 34, 56, 255]));
        let image = crop(
            &bitmap,
            &Region {
                x: 3,
                y: 2,
                width: 2,
                height: 3,
            },
        )
        .unwrap();
        assert_eq!((image.width(), image.height()), (2, 3));
        assert_eq!(image.get_pixel(0, 0), &image::Rgba([12, 34, 56, 255]));
        for region in [
            Region {
                x: 0,
                y: 0,
                width: 0,
                height: 1,
            },
            Region {
                x: 7,
                y: 1,
                width: 2,
                height: 1,
            },
            Region {
                x: u32::MAX,
                y: 0,
                width: 2,
                height: 1,
            },
            Region {
                x: 0,
                y: 0,
                width: 4097,
                height: 1,
            },
        ] {
            assert!(crop(&bitmap, &region).is_err());
        }
    }
}
