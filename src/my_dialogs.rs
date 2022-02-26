use fltk::{prelude::*, *};
use std::{rc::Rc, cell::Cell};

use crate::my_enums::PowerOption;


pub fn show_power_dialog() -> PowerDialog {
    PowerDialog::new()
}

pub fn show_alert_dialog(text: &str) -> AlertDialog {
    AlertDialog::new(text)
}


pub struct PowerDialog {
    confirm: bool,
    choice: menu::Choice,
}

impl PowerDialog {
    pub fn new() -> Self {
        let mut win = window::Window::default()
            .with_size(300, 100)
            .with_label("Power")
            .center_screen();

        let mut vpack = group::Pack::default()
            .with_size(240, 90)
            .center_of_parent()
            .with_type(group::PackType::Vertical);
        vpack.set_spacing(20);
        
        let mut choice = menu::Choice::default()
            .with_size(90, 30);
        choice.add_choice("Shutdown");
        choice.add_choice("Reboot");

        let mut hpack = group::Pack::default()
            .with_size(240, 30)
            .center_of_parent()
            .with_type(group::PackType::Horizontal);
        hpack.set_spacing(20);

        let mut button_cancel = button::Button::default()
            .with_label("Cancel");
        let mut button_confirm = button::Button::default()
            .with_label("Confirm");

        hpack.end();
        hpack.auto_layout();
        vpack.end();
        win.end();

        win.set_color(enums::Color::White);
        win.make_modal(true);
        win.show();

        choice.set_value(0);

        let is_confirm = Rc::new(Cell::new(false));
        
        button_cancel.set_callback({
            let mut win = win.clone();
            let is_confirm_copy1 = is_confirm.clone();
            move |_| {
                win.hide();
                is_confirm_copy1.set(false);
            }
        });

        button_confirm.set_callback({
            let mut win = win.clone();
            let is_confirm_copy2 = is_confirm.clone();
            move |_| {
                win.hide();
                is_confirm_copy2.set(true);
            }
        });

        while win.shown() {
            app::wait();
        }

        let confirm = is_confirm.get();

        Self {
            confirm,
            choice,
        }
    }

    pub fn value(&self) -> PowerOption {
        if self.confirm {
            match self.choice.value() {
                0 => PowerOption::Shutdown,
                1 => PowerOption::Reboot,
                _ => PowerOption::Unknown,
            }
        } else {
            PowerOption::Unknown
        }
    }
}


pub struct AlertDialog {}

impl AlertDialog {
    pub fn new(text: &str) -> Self {
        let mut win = window::Window::default()
            .with_size(300, 85)
            .with_label("Alert")
            .center_screen();

        let vpack = group::Pack::default()
            .with_size(240, 85)
            .center_of_parent()
            .with_type(group::PackType::Vertical);       

        let _label = frame::Frame::default()
            .with_size(0, 30)
            .with_label(text);

        frame::Frame::default().with_size(0, 10);
        let mut button_ok = button::Button::default()
            .with_size(0, 30)
            .with_label("OK");

        vpack.end();
        win.end();
        
        win.set_color(enums::Color::White);
        win.make_modal(true);
        win.show();

        button_ok.set_callback({
            let mut win = win.clone();
            move |_| {
                win.hide();
            }
        });

        while win.shown() {
            app::wait();
        }

        Self {}
    }
}
