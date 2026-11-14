use gpui::{Bounds, Pixels, Window};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    marker::PhantomData,
    rc::Rc,
};

unsafe extern "C" {
    fn ayoru_set_app_icon(bytes: *const u8, length: usize);
    fn ayoru_controls_create() -> *mut c_void;
    fn ayoru_controls_place(
        handle: *mut c_void,
        parent: *mut c_void,
        video: i32,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    );
    fn ayoru_controls_destroy(handle: *mut c_void);
    fn ayoru_search_focus(handle: *mut c_void);
    fn ayoru_search_visible(handle: *mut c_void, visible: bool);
    fn ayoru_search_take(handle: *mut c_void) -> *const c_char;
    fn ayoru_search_clear_submission(handle: *mut c_void);
    fn ayoru_video_load(handle: *mut c_void, url: *const c_char, referrer: *const c_char);
    fn ayoru_video_status(handle: *mut c_void) -> i32;
    fn ayoru_video_error(handle: *mut c_void) -> *const c_char;
    fn ayoru_video_stop(handle: *mut c_void);
}

pub fn set_app_icon() {
    let icon = include_bytes!("../../assets/app-icon.png");
    unsafe { ayoru_set_app_icon(icon.as_ptr(), icon.len()) }
}

pub struct Controls {
    handle: *mut c_void,
    // AppKit objects must never move to a worker thread.
    _main_thread: PhantomData<Rc<()>>,
}

impl Controls {
    pub fn new() -> Self {
        Self {
            handle: unsafe { ayoru_controls_create() },
            _main_thread: PhantomData,
        }
    }

    pub fn place(&self, video: bool, bounds: Bounds<Pixels>, window: &Window) {
        if let Ok(handle) = HasWindowHandle::window_handle(window)
            && let RawWindowHandle::AppKit(handle) = handle.as_raw()
        {
            unsafe {
                ayoru_controls_place(
                    self.handle,
                    handle.ns_view.as_ptr(),
                    i32::from(video),
                    f32::from(bounds.origin.x) as f64,
                    f32::from(bounds.origin.y) as f64,
                    f32::from(bounds.size.width) as f64,
                    f32::from(bounds.size.height) as f64,
                );
            }
        }
    }

    pub fn show_search(&self, visible: bool) {
        unsafe { ayoru_search_visible(self.handle, visible) }
    }

    pub fn focus_search(&self) {
        unsafe { ayoru_search_focus(self.handle) }
    }

    pub fn take_search(&self) -> Option<String> {
        unsafe {
            let text = ayoru_search_take(self.handle);
            if text.is_null() {
                return None;
            }
            let text = CStr::from_ptr(text).to_string_lossy().into_owned();
            ayoru_search_clear_submission(self.handle);
            Some(text)
        }
    }

    pub fn load(&self, url: &str, referrer: Option<&str>) -> Result<(), String> {
        let parsed = reqwest::Url::parse(url).map_err(|_| "Invalid stream URL")?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err("Unsupported stream URL".into());
        }
        let url = CString::new(url).map_err(|_| "Invalid stream URL")?;
        let referrer =
            reqwest::Url::parse(referrer.unwrap_or(crate::provider::allanime::ALLANIME_REFERER))
                .map_err(|_| "Invalid stream referrer")?;
        if !matches!(referrer.scheme(), "http" | "https") || referrer.host_str().is_none() {
            return Err("Unsupported stream referrer".into());
        }
        let referrer = CString::new(referrer.as_str()).map_err(|_| "Invalid stream referrer")?;
        unsafe { ayoru_video_load(self.handle, url.as_ptr(), referrer.as_ptr()) }
        Ok(())
    }

    pub fn status(&self) -> i32 {
        unsafe { ayoru_video_status(self.handle) }
    }

    pub fn error(&self) -> String {
        unsafe {
            let message = ayoru_video_error(self.handle);
            if message.is_null() {
                "The stream could not be played.".into()
            } else {
                CStr::from_ptr(message).to_string_lossy().into_owned()
            }
        }
    }

    pub fn stop(&self) {
        unsafe { ayoru_video_stop(self.handle) }
    }
}

impl Drop for Controls {
    fn drop(&mut self) {
        unsafe { ayoru_controls_destroy(self.handle) }
    }
}
