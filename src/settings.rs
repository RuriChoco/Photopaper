use std::cell::RefCell;

#[derive(Clone, Debug, PartialEq)]
pub enum ExportFormat {
    Png,
    Jpeg,
}

#[derive(Clone, Debug)]
pub struct AppSettings {
    pub export_format: ExportFormat,
    pub keep_metadata: bool,
    pub multi_core_acceleration: bool,
    pub use_ai_bg_removal: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            export_format: ExportFormat::Png,
            keep_metadata: true,
            multi_core_acceleration: true,
            use_ai_bg_removal: true,
        }
    }
}

type SettingsListener = Box<dyn Fn(&AppSettings)>;

thread_local! {
    pub static SETTINGS: RefCell<AppSettings> = RefCell::new(AppSettings::default());
    pub static LISTENERS: RefCell<Vec<SettingsListener>> = RefCell::new(Vec::new());
}

pub fn get_settings() -> AppSettings {
    SETTINGS.with(|s| s.borrow().clone())
}

pub fn update_settings<F>(f: F)
where
    F: FnOnce(&mut AppSettings),
{
    let new_settings = SETTINGS.with(|s| {
        let mut settings = s.borrow_mut();
        f(&mut settings);
        settings.clone()
    });

    LISTENERS.with(|listeners| {
        for listener in listeners.borrow().iter() {
            listener(&new_settings);
        }
    });
}

pub fn add_listener<F>(f: F)
where
    F: Fn(&AppSettings) + 'static,
{
    LISTENERS.with(|listeners| {
        listeners.borrow_mut().push(Box::new(f));
    });
}
