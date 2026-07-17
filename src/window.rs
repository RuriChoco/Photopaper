/* window.rs
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

use gtk::prelude::*;
use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib, gdk};
use std::cell::RefCell;
use image::DynamicImage;
use crate::processor::{Processor, IdSize, PaperSize, LayoutMode};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/github/RuriChoco/Photopaper/window.ui")]
    pub struct PhotopaperWindow {
        #[template_child]
        pub open_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub paper_size_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub id_size_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub export_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub print_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub photo_editing_group: TemplateChild<adw::PreferencesGroup>,
        
        #[template_child]
        pub crop_row: TemplateChild<adw::ActionRow>,
        #[template_child]
        pub bg_remove_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub tolerance_row: TemplateChild<adw::PreferencesRow>,
        #[template_child]
        pub tolerance_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub tolerance_spin: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub brightness_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub brightness_spin: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub contrast_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub contrast_spin: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub sharpen_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub sharpen_spin: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub revert_row: TemplateChild<adw::ActionRow>,
        
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub zoom_scale: TemplateChild<gtk::Scale>,
        #[template_child]
        pub preview_picture: TemplateChild<gtk::Picture>,
        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub status_open_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub main_title: TemplateChild<adw::WindowTitle>,

        pub zoom_level: RefCell<f64>,
        pub gesture_start_zoom: RefCell<f64>,
        pub base_width: RefCell<i32>,
        pub base_height: RefCell<i32>,

        pub raw_loaded_photo: RefCell<Option<DynamicImage>>,
        pub bg_removed_photo: RefCell<Option<DynamicImage>>,
        pub crop_rect: RefCell<Option<(f64, f64, f64, f64)>>,
        pub current_photo: RefCell<Option<DynamicImage>>,
        pub current_layout: RefCell<Option<DynamicImage>>,
        pub current_photo_path: RefCell<Option<std::path::PathBuf>>,
        pub debounce_id: RefCell<Option<glib::SourceId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PhotopaperWindow {
        const NAME: &'static str = "PhotopaperWindow";
        type Type = super::PhotopaperWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PhotopaperWindow {
        fn constructed(&self) {
            self.parent_constructed();
            *self.zoom_level.borrow_mut() = 1.0;
            *self.gesture_start_zoom.borrow_mut() = 1.0;
            let obj = self.obj();
            obj.setup_actions();
        }
    }
    impl WidgetImpl for PhotopaperWindow {}
    impl WindowImpl for PhotopaperWindow {}
    impl ApplicationWindowImpl for PhotopaperWindow {}
    impl AdwApplicationWindowImpl for PhotopaperWindow {}
}

glib::wrapper! {
    pub struct PhotopaperWindow(ObjectSubclass<imp::PhotopaperWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PhotopaperWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_actions(&self) {
        let imp = self.imp();
        
        let initial_ai_bg_removal = crate::settings::get_settings().use_ai_bg_removal;
        imp.tolerance_row.set_visible(!initial_ai_bg_removal);

        crate::settings::add_listener(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |settings| {
                let use_ai = settings.use_ai_bg_removal;
                window.imp().tolerance_row.set_visible(!use_ai);
                if window.imp().bg_remove_switch.is_active() {
                    window.queue_update();
                }
            }
        ));

        let action_open = gio::SimpleAction::new("open", None);
        action_open.connect_activate(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, _| window.open_photo()
        ));
        self.add_action(&action_open);

        let action_export = gio::SimpleAction::new("export", None);
        action_export.set_enabled(false);
        action_export.connect_activate(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, _| window.export_layout()
        ));
        self.add_action(&action_export);

        let action_print = gio::SimpleAction::new("print", None);
        action_print.set_enabled(false);
        action_print.connect_activate(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, _| window.print_layout()
        ));
        self.add_action(&action_print);

        let action_revert = gio::SimpleAction::new("revert", None);
        action_revert.connect_activate(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, _| window.revert_to_original()
        ));
        self.add_action(&action_revert);

        imp.zoom_scale.connect_value_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |scale| {
                window.apply_zoom(scale.value(), None, None);
            }
        ));

        let gesture_zoom = gtk::GestureZoom::new();
        imp.preview_picture.add_controller(gesture_zoom.clone());

        gesture_zoom.connect_begin(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, _| {
                let imp = window.imp();
                let current_zoom = *imp.zoom_level.borrow();
                *imp.gesture_start_zoom.borrow_mut() = current_zoom;
                
                if current_zoom == 1.0 {
                    *imp.base_width.borrow_mut() = imp.preview_picture.width();
                    *imp.base_height.borrow_mut() = imp.preview_picture.height();
                }
            }
        ));

        gesture_zoom.connect_scale_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |gesture, scale| {
                let imp = window.imp();
                let start_zoom = *imp.gesture_start_zoom.borrow();
                let target_zoom = start_zoom * scale;
                if let Some((gx, gy)) = gesture.bounding_box_center() {
                    let viewport_x = gx - imp.scrolled_window.hadjustment().value();
                    let viewport_y = gy - imp.scrolled_window.vadjustment().value();
                    window.apply_zoom(target_zoom, Some(viewport_x), Some(viewport_y));
                } else {
                    window.apply_zoom(target_zoom, None, None);
                }
            }
        ));

        let scroll_zoom = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        imp.preview_picture.add_controller(scroll_zoom.clone());

        scroll_zoom.connect_scroll(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |controller, _dx, dy| {
                if controller.current_event_state().contains(gdk::ModifierType::CONTROL_MASK) {
                    let imp = window.imp();
                    let current_zoom = *imp.zoom_level.borrow();
                    
                    if current_zoom == 1.0 {
                        *imp.base_width.borrow_mut() = imp.preview_picture.width();
                        *imp.base_height.borrow_mut() = imp.preview_picture.height();
                    }

                    // dy > 0 means scrolling down (zoom out), dy < 0 means scrolling up (zoom in)
                    let target_zoom = if dy > 0.0 {
                        current_zoom / 1.15
                    } else {
                        current_zoom * 1.15
                    };

                    window.apply_zoom(target_zoom, None, None);
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));

        imp.paper_size_row.connect_selected_notify(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.queue_update();
            }
        ));

        imp.id_size_row.connect_selected_notify(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.queue_update();
            }
        ));

        imp.brightness_scale.connect_value_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.queue_update();
            }
        ));

        imp.brightness_spin.connect_value_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.queue_update();
            }
        ));

        imp.contrast_scale.connect_value_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.queue_update();
            }
        ));

        imp.contrast_spin.connect_value_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.queue_update();
            }
        ));

        imp.sharpen_scale.connect_value_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.queue_update();
            }
        ));

        imp.sharpen_spin.connect_value_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                window.queue_update();
            }
        ));

        imp.tolerance_scale.connect_value_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                if window.imp().bg_remove_switch.is_active() {
                    window.queue_update();
                }
            }
        ));

        imp.tolerance_spin.connect_value_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                if window.imp().bg_remove_switch.is_active() {
                    window.queue_update();
                }
            }
        ));

        imp.bg_remove_switch.connect_active_notify(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |switch| {
                let imp = window.imp();
                let photo_opt = imp.raw_loaded_photo.borrow().clone();
                let is_active = switch.is_active();
                
                if let Some(photo) = photo_opt {
                    if is_active {
                        let use_ai = crate::settings::get_settings().use_ai_bg_removal;
                        if use_ai {
                            if imp.bg_removed_photo.borrow().is_none() {
                                let dialog = window.show_loading_dialog("Removing Background", Some("The AI model is processing your image locally. This may take a moment."));
                                let (sender, receiver) = std::sync::mpsc::channel();
                                
                                std::thread::spawn(move || {
                                    let result = Processor::remove_background_ai(&photo);
                                    let _ = sender.send(result);
                                });
                                
                                glib::timeout_add_local(
                                std::time::Duration::from_millis(100),
                                glib::clone!(
                                    #[weak(rename_to = window)]
                                    window,
                                    #[weak(rename_to = dialog)]
                                    dialog,
                                    #[upgrade_or]
                                    glib::ControlFlow::Break,
                                    move || {
                                        if let Ok(res) = receiver.try_recv() {
                                            match res {
                                                Ok(bg_removed) => {
                                                    *window.imp().bg_removed_photo.borrow_mut() = Some(bg_removed);
                                                    window.queue_update();
                                                }
                                                Err(e) => {
                                                    eprintln!("AI Background removal failed: {}", e);
                                                    window.imp().bg_remove_switch.set_active(false);
                                                }
                                            }
                                            #[allow(deprecated)]
                                            dialog.close();
                                            glib::ControlFlow::Break
                                        } else {
                                            glib::ControlFlow::Continue
                                        }
                                    }
                                )
                            );
                                return;
                            }
                        } else {
                            // Non-AI (Legacy) bg removal updates immediately in update_preview
                            window.queue_update();
                            return;
                        }
                    } else {
                        // Deactivated
                        window.queue_update();
                        return;
                    }
                }
                
                window.queue_update();
            }
        ));

        imp.crop_row.connect_activated(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                let imp = window.imp();
                let photo_opt = imp.raw_loaded_photo.borrow().clone();
                if let Some(photo) = photo_opt {
                    let current_crop = *imp.crop_rect.borrow();
                    crate::crop_dialog::open_crop_dialog(&window, &photo, current_crop, glib::clone!(
                        #[weak]
                        window,
                        move |rect| {
                            *window.imp().crop_rect.borrow_mut() = Some(rect);
                            window.queue_update();
                        }
                    ));
                }
            }
        ));
    }

    fn revert_to_original(&self) {
        let imp = self.imp();
        let raw_opt = imp.raw_loaded_photo.borrow().clone();
        if let Some(raw) = raw_opt {
            *imp.crop_rect.borrow_mut() = None;
            *imp.bg_removed_photo.borrow_mut() = None;
            *imp.current_photo.borrow_mut() = Some(raw);
            imp.bg_remove_switch.set_active(false);
            imp.brightness_scale.set_value(0.0);
            imp.contrast_scale.set_value(0.0);
            imp.sharpen_scale.set_value(0.0);
            *imp.zoom_level.borrow_mut() = 1.0;
            imp.preview_picture.set_size_request(-1, -1);
            self.queue_update();
        }
    }

    #[allow(deprecated)]
    fn show_loading_dialog(&self, heading: &str, body: Option<&str>) -> adw::MessageDialog {
        let mut builder = adw::MessageDialog::builder()
            .transient_for(self)
            .heading(heading);
            
        if let Some(text) = body {
            builder = builder.body(text);
        }
        let dialog = builder.build();
        dialog.set_width_request(350);
            
        let spinner = gtk::Spinner::builder()
            .spinning(true)
            .halign(gtk::Align::Center)
            .margin_top(24)
            .margin_bottom(24)
            // Use a large width request to prevent GTK layout warnings when it measures the dialog's labels against the extra child's width
            .width_request(300)
            .height_request(48)
            .build();
            
        dialog.set_extra_child(Some(&spinner));
        dialog.present();
        
        dialog
    }

    fn open_photo(&self) {
        let dialog = gtk::FileDialog::builder()
            .title("Open Portrait Photo")
            .build();

        dialog.open(
            Some(self),
            gio::Cancellable::NONE,
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            let loading_dialog = window.show_loading_dialog("Opening Photo...", None);
                            
                            let (sender, receiver) = std::sync::mpsc::channel();
                            let path_clone = path.clone();
                            
                            std::thread::spawn(move || {
                                let result = image::open(&path_clone);
                                let _ = sender.send(result);
                            });
                            
                            glib::timeout_add_local(
                                std::time::Duration::from_millis(50),
                                glib::clone!(
                                    #[weak(rename_to = window)]
                                    window,
                                    #[upgrade_or]
                                    glib::ControlFlow::Break,
                                    move || {
                                        match receiver.try_recv() {
                                            Ok(Ok(img)) => {
                                                loading_dialog.close();
                                                let filename = path.file_name().unwrap_or_default().to_str().unwrap_or("Untitled");
                                                window.imp().main_title.set_title(filename);
                                                
                                                *window.imp().raw_loaded_photo.borrow_mut() = Some(img.clone());
                                                *window.imp().bg_removed_photo.borrow_mut() = None;
                                                *window.imp().crop_rect.borrow_mut() = None;
                                                *window.imp().current_photo.borrow_mut() = Some(img);
                                                *window.imp().current_photo_path.borrow_mut() = Some(path.clone());
                                                window.imp().bg_remove_switch.set_active(false);
                                                window.imp().brightness_scale.set_value(0.0);
                                                window.imp().contrast_scale.set_value(0.0);
                                                window.imp().sharpen_scale.set_value(0.0);
                                                *window.imp().zoom_level.borrow_mut() = 1.0;
                                                window.imp().preview_picture.set_size_request(-1, -1);
                                                window.imp().content_stack.set_visible_child_name("editor");
                                                window.imp().export_row.set_sensitive(true);
                                                window.imp().print_row.set_sensitive(true);
                                                window.imp().photo_editing_group.set_sensitive(true);
                                                
                                                if let Some(action) = window.lookup_action("export").and_downcast::<gio::SimpleAction>() {
                                                    action.set_enabled(true);
                                                }
                                                if let Some(action) = window.lookup_action("print").and_downcast::<gio::SimpleAction>() {
                                                    action.set_enabled(true);
                                                }
                                                
                                                window.queue_update();
                                                glib::ControlFlow::Break
                                            }
                                            Ok(Err(err)) => {
                                                loading_dialog.close();
                                                eprintln!("Failed to load image at {:?}: {}", path, err);
                                                glib::ControlFlow::Break
                                            }
                                            Err(std::sync::mpsc::TryRecvError::Empty) => {
                                                glib::ControlFlow::Continue
                                            }
                                            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                                loading_dialog.close();
                                                glib::ControlFlow::Break
                                            }
                                        }
                                    }
                                )
                            );
                        }
                    }
                }
            ),
        );
    }

    fn queue_update(&self) {
        let imp = self.imp();
        if let Some(source_id) = imp.debounce_id.borrow_mut().take() {
            source_id.remove();
        }
        
        let source_id = glib::timeout_add_local(
            std::time::Duration::from_millis(150),
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move || {
                    window.update_preview();
                    window.imp().debounce_id.borrow_mut().take();
                    glib::ControlFlow::Break
                }
            )
        );
        *imp.debounce_id.borrow_mut() = Some(source_id);
    }

    fn update_preview(&self) {
        let imp = self.imp();
        
        let photo_opt = imp.raw_loaded_photo.borrow().clone();
        let Some(raw_photo) = photo_opt else {
            return;
        };
        let bg_removed = imp.bg_remove_switch.is_active();
        let use_ai = crate::settings::get_settings().use_ai_bg_removal;
        let tolerance = imp.tolerance_scale.value();
        
        let base_photo = if bg_removed {
            if use_ai {
                if let Some(cached) = imp.bg_removed_photo.borrow().clone() {
                    cached
                } else {
                    raw_photo
                }
            } else {
                Processor::remove_background(&raw_photo, tolerance)
            }
        } else {
            raw_photo
        };

        let crop_rect = *imp.crop_rect.borrow();
        let brightness = imp.brightness_scale.value() as i32;
        let contrast = imp.contrast_scale.value() as f32;
        let sharpen = imp.sharpen_scale.value() as f32;

        let paper = match imp.paper_size_row.selected() {
            0 => PaperSize::FourR,
            1 => PaperSize::FiveR,
            2 => PaperSize::A4,
            _ => PaperSize::A4,
        };

        let layout = match imp.id_size_row.selected() {
            0 => LayoutMode::Single(IdSize::OneByOne),
            1 => LayoutMode::Single(IdSize::TwoByTwo),
            2 => LayoutMode::Single(IdSize::Passport),
            _ => LayoutMode::Mixed,
        };

        let (sender, receiver) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            let mut edited = base_photo;
            
            if let Some((x, y, w, h)) = crop_rect {
                let px = (x * edited.width() as f64) as u32;
                let py = (y * edited.height() as f64) as u32;
                let pw = (w * edited.width() as f64) as u32;
                let ph = (h * edited.height() as f64) as u32;
                edited = edited.crop(px, py, pw, ph);
            }
            if brightness != 0 || contrast != 0.0 || sharpen > 0.0 {
                edited = Processor::adjust_color(&edited, brightness, contrast, sharpen);
            }
            
            let layout_img = Processor::generate_layout(&edited, &paper, &layout);
            
            let rgba = layout_img.to_rgba8();
            let width = rgba.width() as i32;
            let height = rgba.height() as i32;
            let rowstride = width * 4;
            let bytes = glib::Bytes::from(rgba.as_raw());
            
            let _ = sender.send((edited, layout_img, width, height, rowstride, bytes));
        });

        glib::timeout_add_local(
            std::time::Duration::from_millis(50),
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                #[upgrade_or]
                glib::ControlFlow::Break,
                move || {
                    match receiver.try_recv() {
                        Ok((edited, layout_img, width, height, rowstride, bytes)) => {
                            let imp = window.imp();
                            
                            *imp.current_photo.borrow_mut() = Some(edited);
                            *imp.current_layout.borrow_mut() = Some(layout_img);
                            
                            let texture = gdk::MemoryTexture::new(
                                width,
                                height,
                                gdk::MemoryFormat::R8g8b8a8,
                                &bytes,
                                rowstride as usize,
                            );
                            
                            imp.preview_picture.set_paintable(Some(&texture));
                            
                            glib::ControlFlow::Break
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            glib::ControlFlow::Continue
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            glib::ControlFlow::Break
                        }
                    }
                }
            )
        );
    }

    fn export_layout(&self) {
        let imp = self.imp();
        if imp.current_layout.borrow().is_none() {
            return; // No layout to save
        }

        let settings = crate::settings::get_settings();
        let is_jpeg = matches!(settings.export_format, crate::settings::ExportFormat::Jpeg);
        let keep_metadata = settings.keep_metadata;
        let initial_name = if is_jpeg { "layout.jpg" } else { "layout.png" };

        let dialog = gtk::FileDialog::builder()
            .title("Save Layout")
            .initial_name(initial_name)
            .build();

        dialog.save(
            Some(self),
            gio::Cancellable::NONE,
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |result| {
                    if let Ok(file) = result {
                        if let Some(path) = file.path() {
                            if let Some(layout) = window.imp().current_layout.borrow().as_ref() {
                                let format = if is_jpeg { image::ImageFormat::Jpeg } else { image::ImageFormat::Png };
                                let orig_path = window.imp().current_photo_path.borrow().clone();
                                
                                let loading_dialog = window.show_loading_dialog("Exporting Layout...", None);
                                
                                let (sender, receiver) = std::sync::mpsc::channel();
                                let layout_clone = layout.clone();
                                let path_clone = path.clone();
                                
                                std::thread::spawn(move || {
                                    let success = if keep_metadata && orig_path.is_some() {
                                        use img_parts::ImageEXIF;
                                        let mut buffer = std::io::Cursor::new(Vec::new());
                                        let _ = layout_clone.write_to(&mut buffer, format);
                                        let mut output_bytes = buffer.into_inner();
                                        
                                        if let Some(path) = orig_path {
                                            if let Ok(orig_bytes) = std::fs::read(path) {
                                            let mut exif_data = None;
                                            if let Ok(jpeg) = img_parts::jpeg::Jpeg::from_bytes(orig_bytes.clone().into()) {
                                                exif_data = jpeg.exif();
                                            } else if let Ok(png) = img_parts::png::Png::from_bytes(orig_bytes.into()) {
                                                exif_data = png.exif();
                                            }
                                            
                                            if let Some(exif) = exif_data {
                                                let mut injected = Vec::new();
                                                if is_jpeg {
                                                    if let Ok(mut jpeg) = img_parts::jpeg::Jpeg::from_bytes(output_bytes.clone().into()) {
                                                        jpeg.set_exif(Some(exif));
                                                        if jpeg.encoder().write_to(&mut injected).is_ok() {
                                                            output_bytes = injected;
                                                        }
                                                    }
                                                } else {
                                                    if let Ok(mut png) = img_parts::png::Png::from_bytes(output_bytes.clone().into()) {
                                                        png.set_exif(Some(exif));
                                                        if png.encoder().write_to(&mut injected).is_ok() {
                                                            output_bytes = injected;
                                                        }
                                                    }
                                                }
                                            }
                                            }
                                        }
                                        std::fs::write(&path_clone, output_bytes).is_ok()
                                    } else {
                                        layout_clone.save_with_format(&path_clone, format).is_ok()
                                    };
                                    let _ = sender.send(success);
                                });
                                
                                glib::timeout_add_local(
                                    std::time::Duration::from_millis(50),
                                    glib::clone!(
                                        move || {
                                            match receiver.try_recv() {
                                                Ok(success) => {
                                                    loading_dialog.close();
                                                    if !success {
                                                        eprintln!("Failed to export layout.");
                                                    }
                                                    glib::ControlFlow::Break
                                                }
                                                Err(std::sync::mpsc::TryRecvError::Empty) => {
                                                    glib::ControlFlow::Continue
                                                }
                                                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                                    loading_dialog.close();
                                                    glib::ControlFlow::Break
                                                }
                                            }
                                        }
                                    )
                                );
                            }
                        }
                    }
                }
            ),
        );
    }

    fn print_layout(&self) {
        let imp = self.imp();
        if imp.current_layout.borrow().is_none() {
            return;
        }
        let layout_img = imp.current_layout.borrow().as_ref().unwrap().clone();

        let print_op = gtk::PrintOperation::new();
        print_op.set_n_pages(1);
        print_op.set_use_full_page(true); // Ignore printer margins, we handle layout ourselves
        print_op.set_unit(gtk::Unit::Inch);

        print_op.connect_draw_page(move |_, context, page_nr| {
            if page_nr != 0 { return; }
            let cairo_ctx = context.cairo_context();
            
            let width = layout_img.width() as i32;
            let height = layout_img.height() as i32;
            
            if let Ok(mut surface) = gtk::cairo::ImageSurface::create(gtk::cairo::Format::ARgb32, width, height) {
                {
                    let mut data = surface.data().unwrap();
                    let rgba = layout_img.to_rgba8();
                    let rgba_data = rgba.as_raw();
                    for i in (0..rgba_data.len()).step_by(4) {
                        data[i] = rgba_data[i+2];     // B
                        data[i+1] = rgba_data[i+1];   // G
                        data[i+2] = rgba_data[i];     // R
                        data[i+3] = rgba_data[i+3];   // A
                    }
                }
                
                // Scale Cairo to map pixels to inches based on our 300 DPI standard!
                // This GUARANTEES that a 600px square will print exactly 2.0 inches wide!
                cairo_ctx.scale(1.0 / 300.0, 1.0 / 300.0);
                
                cairo_ctx.set_source_surface(&surface, 0.0, 0.0).unwrap();
                cairo_ctx.paint().unwrap();
            }
        });

        if let Err(e) = print_op.run(
            gtk::PrintOperationAction::PrintDialog,
            Some(self),
        ) {
            eprintln!("Error printing: {}", e);
        }
    }

    fn apply_zoom(&self, target_zoom: f64, focal_x: Option<f64>, focal_y: Option<f64>) {
        let imp = self.imp();
        let new_zoom = target_zoom.clamp(1.0, 10.0);
        let old_zoom = *imp.zoom_level.borrow();
        
        if (new_zoom - old_zoom).abs() < 0.001 {
            return;
        }
        
        *imp.zoom_level.borrow_mut() = new_zoom;
        imp.zoom_scale.set_value(new_zoom);

        if new_zoom == 1.0 {
            imp.preview_picture.set_size_request(-1, -1);
            return;
        }

        let mut bw = *imp.base_width.borrow();
        let mut bh = *imp.base_height.borrow();
        
        if bw == 0 || bh == 0 {
            bw = imp.preview_picture.width();
            bh = imp.preview_picture.height();
            *imp.base_width.borrow_mut() = bw;
            *imp.base_height.borrow_mut() = bh;
        }

        let new_w = bw as f64 * new_zoom;
        let new_h = bh as f64 * new_zoom;
        
        let hadj = imp.scrolled_window.hadjustment();
        let vadj = imp.scrolled_window.vadjustment();
        
        let f_x = focal_x.unwrap_or_else(|| hadj.page_size() / 2.0);
        let f_y = focal_y.unwrap_or_else(|| vadj.page_size() / 2.0);

        let img_focal_x = hadj.value() + f_x;
        let img_focal_y = vadj.value() + f_y;

        let new_img_focal_x = img_focal_x * (new_zoom / old_zoom);
        let new_img_focal_y = img_focal_y * (new_zoom / old_zoom);

        let new_scroll_x = new_img_focal_x - f_x;
        let new_scroll_y = new_img_focal_y - f_y;

        imp.preview_picture.set_size_request(new_w as i32, new_h as i32);

        // Synchronously update the scroll positions in the exact same frame.
        // We temporarily increase the adjustment bounds so our new values aren't clamped
        // before GTK allocates the new size request.
        let h_upper = (new_w + 48.0).max(hadj.upper());
        let v_upper = (new_h + 48.0).max(vadj.upper());
        
        hadj.set_upper(h_upper);
        vadj.set_upper(v_upper);

        let h_max = (h_upper - hadj.page_size()).max(hadj.lower());
        let v_max = (v_upper - vadj.page_size()).max(vadj.lower());

        hadj.set_value(new_scroll_x.clamp(hadj.lower(), h_max));
        vadj.set_value(new_scroll_y.clamp(vadj.lower(), v_max));
    }
}
