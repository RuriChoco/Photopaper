#![allow(deprecated)]

use adw::subclass::prelude::*;
use adw::prelude::*;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/github/RuriChoco/Photopaper/preferences.ui")]
    pub struct PhotopaperPreferencesWindow {
        #[template_child]
        pub theme_combo_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub pref_export_format: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub pref_keep_metadata: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub pref_acceleration: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub pref_ai_bg_removal: TemplateChild<adw::SwitchRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PhotopaperPreferencesWindow {
        const NAME: &'static str = "PhotopaperPreferencesWindow";
        type Type = super::PhotopaperPreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PhotopaperPreferencesWindow {
        fn constructed(&self) {
            self.parent_constructed();
            
            let style_manager = adw::StyleManager::default();

            // Set initial state based on current style manager
            let current_scheme = style_manager.color_scheme();
            let selected_index = match current_scheme {
                adw::ColorScheme::ForceLight => 1,
                adw::ColorScheme::ForceDark => 2,
                _ => 0,
            };
            self.theme_combo_row.set_selected(selected_index);

            self.theme_combo_row.connect_selected_notify(move |combo| {
                let scheme = match combo.selected() {
                    1 => adw::ColorScheme::ForceLight,
                    2 => adw::ColorScheme::ForceDark,
                    _ => adw::ColorScheme::Default,
                };
                adw::StyleManager::default().set_color_scheme(scheme);
            });

            let settings = crate::settings::get_settings();
            
            let format_idx = match settings.export_format {
                crate::settings::ExportFormat::Png => 0,
                crate::settings::ExportFormat::Jpeg => 1,
            };
            self.pref_export_format.set_selected(format_idx);
            self.pref_keep_metadata.set_active(settings.keep_metadata);
            self.pref_acceleration.set_active(settings.multi_core_acceleration);
            self.pref_ai_bg_removal.set_active(settings.use_ai_bg_removal);

            self.pref_export_format.connect_selected_notify(move |combo| {
                let format = match combo.selected() {
                    0 => crate::settings::ExportFormat::Png,
                    _ => crate::settings::ExportFormat::Jpeg,
                };
                crate::settings::update_settings(|s| s.export_format = format);
            });

            self.pref_keep_metadata.connect_active_notify(move |switch| {
                let active = switch.is_active();
                crate::settings::update_settings(|s| s.keep_metadata = active);
            });

            self.pref_acceleration.connect_active_notify(move |switch| {
                let active = switch.is_active();
                crate::settings::update_settings(|s| s.multi_core_acceleration = active);
            });

            self.pref_ai_bg_removal.connect_active_notify(move |switch| {
                let active = switch.is_active();
                crate::settings::update_settings(|s| s.use_ai_bg_removal = active);
            });
        }
    }
    
    impl WidgetImpl for PhotopaperPreferencesWindow {}
    impl WindowImpl for PhotopaperPreferencesWindow {}
    impl AdwWindowImpl for PhotopaperPreferencesWindow {}
    impl PreferencesWindowImpl for PhotopaperPreferencesWindow {}
}

glib::wrapper! {
    pub struct PhotopaperPreferencesWindow(ObjectSubclass<imp::PhotopaperPreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;
}

impl PhotopaperPreferencesWindow {
    pub fn new(parent: &impl IsA<gtk::Window>) -> Self {
        let window: Self = glib::Object::builder()
            .property("transient-for", parent)
            .build();
        window
    }
}
