    /* application.rs
 *
 * Copyright 2026 Dion
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use gettextrs::gettext;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config::VERSION;
use crate::window::PhotopaperWindow;
use crate::preferences::PhotopaperPreferencesWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct PhotopaperApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for PhotopaperApplication {
        const NAME: &'static str = "PhotopaperApplication";
        type Type = super::PhotopaperApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for PhotopaperApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            
            // Global app shortcuts
            obj.set_accels_for_action("app.quit", &["<control>q"]);
            obj.set_accels_for_action("app.shortcuts", &["<control>question"]);
            obj.set_accels_for_action("app.preferences", &["<control>comma"]);
            
            // Window actions (defined in window.rs)
            obj.set_accels_for_action("win.open", &["<control>o"]);
            obj.set_accels_for_action("win.export", &["<control>e"]);
            obj.set_accels_for_action("win.print", &["<control>p"]);
            obj.set_accels_for_action("win.revert", &["<control>r"]);
        }
    }

    impl ApplicationImpl for PhotopaperApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();
            
            if let Some(display) = gtk::gdk::Display::default() {
                let icon_theme = gtk::IconTheme::for_display(&display);
                icon_theme.add_resource_path("/com/github/RuriChoco/Photopaper/icons");
            }

            // Get the current window or create one if necessary
            let window = application.active_window().unwrap_or_else(|| {
                let window = PhotopaperWindow::new(&*application);
                window.upcast()
            });

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for PhotopaperApplication {}
    impl AdwApplicationImpl for PhotopaperApplication {}
}

glib::wrapper! {
    pub struct PhotopaperApplication(ObjectSubclass<imp::PhotopaperApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PhotopaperApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/com/github/RuriChoco/Photopaper")
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        let shortcuts_action = gio::ActionEntry::builder("shortcuts")
            .activate(move |app: &Self, _, _| app.show_shortcuts())
            .build();
        let pref_action = gio::ActionEntry::builder("preferences")
            .activate(move |app: &Self, _, _| app.show_preferences())
            .build();
            
        self.add_action_entries([quit_action, about_action, shortcuts_action, pref_action]);
    }

    fn show_shortcuts(&self) {
        let builder = gtk::Builder::from_resource("/com/github/RuriChoco/Photopaper/shortcuts-dialog.ui");
        let window: gtk::ShortcutsWindow = builder.object("shortcuts_dialog").unwrap();
        if let Some(active_window) = self.active_window() {
            window.set_transient_for(Some(&active_window));
        }
        window.present();
    }

    fn show_preferences(&self) {
        if let Some(active_window) = self.active_window() {
            let pref_window = PhotopaperPreferencesWindow::new(&active_window);
            pref_window.present();
        }
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        
        let raw_license = include_str!("../LICENSE");
        
        // Remove hard wrapping and excessive indentation so the GUI can wrap it naturally
        let mut unwrapped_license = String::new();
        for paragraph in raw_license.replace("\r\n", "\n").split("\n\n") {
            let unwrapped = paragraph.replace("\n", " ");
            // Collapse multiple spaces into a single space to fix awkward gaps from terminal formatting
            let cleaned = unwrapped.split_whitespace().collect::<Vec<_>>().join(" ");
            unwrapped_license.push_str(&cleaned);
            unwrapped_license.push_str("\n\n");
        }
        
        let mut formatted_license = glib::markup_escape_text(&unwrapped_license).to_string();
        
        // Convert escaped URLs back into clickable Pango markup links
        formatted_license = formatted_license.replace(
            "&lt;https://fsf.org/&gt;", 
            "<a href=\"https://fsf.org/\">https://fsf.org/</a>"
        );
        formatted_license = formatted_license.replace(
            "&lt;https://www.gnu.org/licenses/&gt;", 
            "<a href=\"https://www.gnu.org/licenses/\">https://www.gnu.org/licenses/</a>"
        );
        formatted_license = formatted_license.replace(
            "&lt;https://www.gnu.org/licenses/why-not-lgpl.html&gt;", 
            "<a href=\"https://www.gnu.org/licenses/why-not-lgpl.html\">https://www.gnu.org/licenses/why-not-lgpl.html</a>"
        );
        
        let about = adw::AboutDialog::builder()
            .application_name("Photopaper")
            .application_icon("com.github.RuriChoco.Photopaper")
            .developer_name("RuriChoco")
            .version(VERSION)
            .translator_credits(gettext("translator-credits"))
            .copyright("© 2026 RuriChoco")
            .license_type(gtk::License::Custom)
            .license(&formatted_license)
            .build();

        about.add_credit_section(Some("Author"), &["RuriChoco"]);
        about.add_credit_section(Some("Built By"), &["Gemini 3.1 (AI Assistant)"]);
        about.add_acknowledgement_section(Some("image"), &["The image-rs Developers"]);
        about.add_acknowledgement_section(Some("gtk4-rs"), &["The gtk-rs Project Developers"]);
        about.add_acknowledgement_section(Some("libadwaita"), &["Bilal Elmoussaoui", "The GNOME Foundation"]);
        about.add_acknowledgement_section(Some("rayon"), &["Josh Stone", "Niko Matsakis"]);
        about.add_acknowledgement_section(Some("img-parts"), &["Paolo Barbolini"]);
        about.add_acknowledgement_section(Some("kamadak-exif"), &["Ken'ichi KAMADA"]);

        about.present(Some(&window));
    }
}
