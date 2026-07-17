use gtk::prelude::*;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use image::DynamicImage;

#[derive(Clone, Copy)]
enum DragAction {
    None,
    Move,
    ResizeTopLeft,
    ResizeTopRight,
    ResizeBottomLeft,
    ResizeBottomRight,
}

struct CropState {
    surface: gtk::cairo::ImageSurface,
    img_w: f64,
    img_h: f64,
    crop_rect: (f64, f64, f64, f64), // x, y, w, h normalized
    action: DragAction,
    start_rect: (f64, f64, f64, f64),
}

pub fn open_crop_dialog(
    parent: &impl IsA<gtk::Window>,
    photo: &DynamicImage,
    initial_crop: Option<(f64, f64, f64, f64)>,
    on_cropped: impl Fn((f64, f64, f64, f64)) + 'static,
) {
    let rgba = photo.to_rgba8();
    let img_w = rgba.width() as i32;
    let img_h = rgba.height() as i32;
    let mut surface = gtk::cairo::ImageSurface::create(gtk::cairo::Format::ARgb32, img_w, img_h).unwrap();
    {
        let mut data = surface.data().unwrap();
        let rgba_data = rgba.as_raw();
        for i in (0..rgba_data.len()).step_by(4) {
            data[i] = rgba_data[i+2];     // B
            data[i+1] = rgba_data[i+1];   // G
            data[i+2] = rgba_data[i];     // R
            data[i+3] = rgba_data[i+3];   // A
        }
    }

    let initial_w = 0.6;
    let initial_h = (initial_w * img_w as f64) / img_h as f64;
    let starting_rect = initial_crop.unwrap_or(((1.0 - initial_w) / 2.0, (1.0 - initial_h) / 2.0, initial_w, initial_h));

    let state = Rc::new(RefCell::new(CropState {
        surface,
        img_w: img_w as f64,
        img_h: img_h as f64,
        crop_rect: starting_rect,
        action: DragAction::None,
        start_rect: starting_rect,
    }));

    let dialog = adw::Window::builder()
        .title("Crop Photo")
        .modal(true)
        .transient_for(parent)
        .default_width(800)
        .default_height(600)
        .build();

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    
    let header = adw::HeaderBar::new();
    let apply_btn = gtk::Button::with_label("Apply Crop");
    apply_btn.add_css_class("suggested-action");
    header.pack_end(&apply_btn);
    vbox.append(&header);

    let da = gtk::DrawingArea::new();
    da.set_vexpand(true);
    da.set_hexpand(true);
    
    let state_draw = state.clone();
    da.set_draw_func(move |_, cr, width, height| {
        let state = state_draw.borrow();
        let canvas_w = width as f64;
        let canvas_h = height as f64;
        
        let img_aspect = state.img_w / state.img_h;
        let canvas_aspect = canvas_w / canvas_h;
        
        let (draw_w, draw_h, offset_x, offset_y) = if canvas_aspect > img_aspect {
            let h = canvas_h;
            let w = h * img_aspect;
            (w, h, (canvas_w - w) / 2.0, 0.0)
        } else {
            let w = canvas_w;
            let h = w / img_aspect;
            (w, h, 0.0, (canvas_h - h) / 2.0)
        };
        
        cr.save().unwrap();
        cr.translate(offset_x, offset_y);
        cr.scale(draw_w / state.img_w, draw_h / state.img_h);
        cr.set_source_surface(&state.surface, 0.0, 0.0).unwrap();
        cr.paint().unwrap();
        cr.restore().unwrap();
        
        let cx = offset_x + state.crop_rect.0 * draw_w;
        let cy = offset_y + state.crop_rect.1 * draw_h;
        let cw = state.crop_rect.2 * draw_w;
        let ch = state.crop_rect.3 * draw_h;
        
        cr.set_source_rgba(0.0, 0.0, 0.0, 0.6);
        cr.rectangle(offset_x, offset_y, draw_w, cy - offset_y); 
        cr.rectangle(offset_x, cy + ch, draw_w, (offset_y + draw_h) - (cy + ch)); 
        cr.rectangle(offset_x, cy, cx - offset_x, ch); 
        cr.rectangle(cx + cw, cy, (offset_x + draw_w) - (cx + cw), ch); 
        cr.fill().unwrap();
        
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.8);
        cr.set_line_width(1.0);
        cr.move_to(cx + cw / 3.0, cy); cr.line_to(cx + cw / 3.0, cy + ch);
        cr.move_to(cx + 2.0 * cw / 3.0, cy); cr.line_to(cx + 2.0 * cw / 3.0, cy + ch);
        cr.move_to(cx, cy + ch / 3.0); cr.line_to(cx + cw, cy + ch / 3.0);
        cr.move_to(cx, cy + 2.0 * ch / 3.0); cr.line_to(cx + cw, cy + 2.0 * ch / 3.0);
        cr.stroke().unwrap();
        
        cr.set_line_width(2.0);
        cr.rectangle(cx, cy, cw, ch);
        cr.stroke().unwrap();
    });

    let drag = gtk::GestureDrag::new();
    
    let state_drag = state.clone();
    let da_ref1 = da.clone();
    drag.connect_drag_begin(move |_, x, y| {
        let mut s = state_drag.borrow_mut();
        s.start_rect = s.crop_rect;
        
        let canvas_w = da_ref1.width() as f64;
        let canvas_h = da_ref1.height() as f64;
        let img_aspect = s.img_w / s.img_h;
        let canvas_aspect = canvas_w / canvas_h;
        let (draw_w, draw_h, offset_x, offset_y) = if canvas_aspect > img_aspect {
            let h = canvas_h; (h * img_aspect, h, (canvas_w - (h * img_aspect)) / 2.0, 0.0)
        } else {
            let w = canvas_w; (w, w / img_aspect, 0.0, (canvas_h - (w / img_aspect)) / 2.0)
        };
        
        let cx = offset_x + s.crop_rect.0 * draw_w;
        let cy = offset_y + s.crop_rect.1 * draw_h;
        let cw = s.crop_rect.2 * draw_w;
        let ch = s.crop_rect.3 * draw_h;
        
        let margin = 20.0;
        let in_tl = (x - cx).abs() < margin && (y - cy).abs() < margin;
        let in_tr = (x - (cx + cw)).abs() < margin && (y - cy).abs() < margin;
        let in_bl = (x - cx).abs() < margin && (y - (cy + ch)).abs() < margin;
        let in_br = (x - (cx + cw)).abs() < margin && (y - (cy + ch)).abs() < margin;
        let in_box = x > cx && x < cx + cw && y > cy && y < cy + ch;
        
        if in_tl { s.action = DragAction::ResizeTopLeft; }
        else if in_tr { s.action = DragAction::ResizeTopRight; }
        else if in_bl { s.action = DragAction::ResizeBottomLeft; }
        else if in_br { s.action = DragAction::ResizeBottomRight; }
        else if in_box { s.action = DragAction::Move; }
        else { s.action = DragAction::None; }
    });

    let state_update = state.clone();
    let da_ref2 = da.clone();
    drag.connect_drag_update(move |_, dx, dy| {
        let mut s = state_update.borrow_mut();
        let canvas_w = da_ref2.width() as f64;
        let canvas_h = da_ref2.height() as f64;
        let img_aspect = s.img_w / s.img_h;
        let canvas_aspect = canvas_w / canvas_h;
        let (draw_w, draw_h, _, _) = if canvas_aspect > img_aspect {
            let h = canvas_h; (h * img_aspect, h, 0.0, 0.0)
        } else {
            let w = canvas_w; (w, w / img_aspect, 0.0, 0.0)
        };
        
        let ndx = dx / draw_w;
        let ndy = dy / draw_h;
        
        let (mut x, mut y, mut w, mut h) = s.start_rect;
        
        match s.action {
            DragAction::Move => { x += ndx; y += ndy; }
            DragAction::ResizeTopLeft => { x += ndx; y += ndy; w -= ndx; h -= ndy; }
            DragAction::ResizeTopRight => { y += ndy; w += ndx; h -= ndy; }
            DragAction::ResizeBottomLeft => { x += ndx; w -= ndx; h += ndy; }
            DragAction::ResizeBottomRight => { w += ndx; h += ndy; }
            DragAction::None => {}
        }
        
        if w < 0.1 { w = 0.1; }
        if h < 0.1 { h = 0.1; }
        if x < 0.0 { x = 0.0; }
        if y < 0.0 { y = 0.0; }
        if x + w > 1.0 { x = 1.0 - w; }
        if y + h > 1.0 { y = 1.0 - h; }
        if x < 0.0 { w += x; x = 0.0; }
        if y < 0.0 { h += y; y = 0.0; }
        if x + w > 1.0 { w = 1.0 - x; }
        if y + h > 1.0 { h = 1.0 - y; }
        
        s.crop_rect = (x, y, w, h);
        da_ref2.queue_draw();
    });

    da.add_controller(drag);
    vbox.append(&da);
    dialog.set_content(Some(&vbox));
    
    let dialog_weak = dialog.downgrade();
    apply_btn.connect_clicked(move |_| {
        let s = state.borrow();
        on_cropped(s.crop_rect);
        
        if let Some(d) = dialog_weak.upgrade() {
            d.close();
        }
    });

    dialog.present();
}
