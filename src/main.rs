#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod ws;
pub mod my_enums;
pub mod my_dialogs;

use ws::connect_to_ws;
use my_enums::{MyAppMessage, WSEventValue, PowerOption};
use my_dialogs::{show_power_dialog, show_alert_dialog};

use fltk::{prelude::*, *};
use fltk_theme::{WidgetScheme, SchemeType};
use tokio::task;
use futures::channel::mpsc::{unbounded, UnboundedSender};
use tokio_tungstenite::tungstenite::protocol::Message;
use std::cell::RefCell;


#[allow(dead_code)]
struct MyApp {
    app: app::App,
    main_win: window::Window,
    input_address: input::Input,
    button_connect: button::Button,
    button_prev: button::Button,
    button_play_pause: button::Button,
    button_next: button::Button,
    button_shuffle: button::Button,
    button_repeat: button::Button,
    button_power: button::Button,
    spinner_volume: misc::Spinner,

    app_msg_sender: app::Sender<MyAppMessage>,
    app_msg_receiver: app::Receiver<MyAppMessage>,
    
    ws_input_sender: RefCell<Option<UnboundedSender<Message>>>,
}

impl MyApp {
    pub fn new() -> Self {
        let app = app::App::default();
        let widget_scheme = WidgetScheme::new(SchemeType::Fluent);
        widget_scheme.apply();

        let mut main_win = window::Window::default()
            .with_size(550, 220)
            .with_label("SpotifyPi Control Panel")
            .center_screen();

        let (app_msg_sender, app_msg_receiver) = app::channel::<MyAppMessage>();

        let mut main_panel = group::Flex::default_fill().column(); 
        let spacer_top = frame::Frame::default();

        let mut row1 = group::Flex::default().row();
        let (mut input_address, mut button_connect) = Self::row1_panel(&mut row1);
        input_address.set_value("spotifypi.local:9487");
        button_connect.emit(app_msg_sender, MyAppMessage::ClickConnect);

        let spacer1 = frame::Frame::default();

        let mut row2 = group::Flex::default().row();
        let (mut button_prev, mut button_play_pause, mut button_next) = Self::row2_panel(&mut row2);
        button_prev.emit(app_msg_sender, MyAppMessage::PrevTrack);
        button_play_pause.emit(app_msg_sender, MyAppMessage::PlayPause);
        button_next.emit(app_msg_sender, MyAppMessage::NextTrack);

        let spacer2 = frame::Frame::default();
    
        let mut row3 = group::Flex::default().row();
        let (mut button_shuffle, mut button_repeat) = Self::row3_panel(&mut row3);
        button_shuffle.emit(app_msg_sender, MyAppMessage::ToggleShuffle);
        button_repeat.emit(app_msg_sender, MyAppMessage::ToggleRepeat);

        let _spacer3 = frame::Frame::default();
        
        let mut row4 = group::Flex::default().row();
        let (mut button_power, mut spinner_volume) = Self::row4_panel(&mut row4);
        button_power.emit(app_msg_sender, MyAppMessage::ClickPower);
        spinner_volume.emit(app_msg_sender, MyAppMessage::ChangeVolume);

        let spacer_bottom = frame::Frame::default();
    
        main_panel.set_size(&spacer_top, 10);
        main_panel.set_size(&row1, 32);
        main_panel.set_size(&spacer1, 8);
        main_panel.set_size(&row2, 32);
        main_panel.set_size(&spacer2, 8);
        main_panel.set_size(&row3, 32);
        main_panel.set_size(&row4, 32);
        main_panel.set_size(&spacer_bottom, 10);
        main_panel.end();

        main_win.resizable(&main_panel);
        main_win.set_color(enums::Color::White);
        main_win.end();
        main_win.show();
        main_win.size_range(500, 215, 0, 0);

        let ws_input_sender = RefCell::new(None);

        Self {
            app,
            main_win,
            input_address,
            button_connect,
            button_prev,
            button_play_pause,
            button_next,
            button_shuffle,
            button_repeat,
            button_power,
            spinner_volume,
            app_msg_sender,
            app_msg_receiver,
            ws_input_sender,
        }
    }

    pub fn run(&mut self) {
        self.control_widgets_enable(false);
        while self.app.wait() {
            if let Some(msg) = self.app_msg_receiver.recv() {
                match msg {
                    MyAppMessage::ClickConnect => {
                        let address = self.input_address.value();
                        let ws_addr = format!("ws://{}", address);
                        let url = match url::Url::parse(ws_addr.as_str()) {
                            Ok(url) => url,
                            Err(e) => {
                                eprintln!("Url::parse failed: {}", e);
                                show_alert_dialog(&format!("{}", e));
                                return;
                            }
                        };
                        println!("Connecting to {}", ws_addr);
                        
                        self.connect_widgets_enable(false, "Connecting...");

                        let (input_tx, input_rx)  = unbounded::<Message>();
                        self.ws_input_sender.replace(Some(input_tx));
                        let output_tx = self.app_msg_sender.clone();

                        task::spawn(async move {
                            connect_to_ws(url, input_rx, output_tx).await;
                        });
                    }
                    MyAppMessage::PrevTrack => {
                        println!("prev");
                        self.send_command_to_ws("prev_track");
                    }
                    MyAppMessage::PlayPause => {
                        println!("play/pause");
                        self.send_command_to_ws("toggle_play_pause");
                    }
                    MyAppMessage::NextTrack => {
                        println!("next");
                        self.send_command_to_ws("next_track");
                    }
                    MyAppMessage::ToggleShuffle => {
                        println!("shuffle");
                        self.send_command_to_ws("toggle_shuffle");
                    }
                    MyAppMessage::ToggleRepeat => {
                        println!("repeat");
                        self.send_command_to_ws("toggle_repeat_state");
                    }
                    MyAppMessage::ClickPower => {
                        let dialog = show_power_dialog();
                        match dialog.value() {
                            PowerOption::Shutdown => {
                                println!("PowerOption: Shutdown");
                                self.send_command_to_ws("shutdown");
                            }
                            PowerOption::Reboot => {
                                println!("PowerOption: Reboot");
                                self.send_command_to_ws("reboot");
                            }
                            PowerOption::Unknown => println!("PowerOption: Unknown"),
                        }
                    }
                    MyAppMessage::ChangeVolume => {
                        let volume = self.spinner_volume.value() as u32;
                        println!("volume: {}", volume);
                        self.send_command_to_ws(&format!("set_volume {}", volume));
                    }
                    MyAppMessage::WSEventValue(value) => {
                        match value {
                            WSEventValue::Unknown => println!("WSEvent: Unknown."),
                            WSEventValue::Missing => println!("WSEvent: Missing."),
                            WSEventValue::NotFound => println!("WSEvent: NotFound."),
                            WSEventValue::Connect(success) => {
                                println!("WSEvent: Connect: {}.", success);
                                if success {
                                    self.control_widgets_enable(true);
                                    self.send_command_to_ws("get_volume");
                                } else {
                                    self.control_widgets_enable(false);
                                    show_alert_dialog("Connect failed.");
                                }
                            }
                            WSEventValue::Disconnect => {
                                println!("WSEvent: Disconnect");
                                self.control_widgets_enable(false);
                                show_alert_dialog("WebSocket connection closed.");
                            }
                            WSEventValue::PrevTrack(success) => println!("WSEvent: PrevTrack ({}).", success),
                            WSEventValue::NextTrack(success) => println!("WSEvent: NextTrack ({}).", success),
                            WSEventValue::TogglePlayPause(success) => println!("WSEvent: TogglePlayPause ({}).", success),
                            WSEventValue::ToggleShuffle(success) => println!("WSEvent: ToggleShuffle ({}).", success),
                            WSEventValue::ToggleRepeatState(success) => println!("WSEvent: ToggleRepeatState ({}).", success),
                            WSEventValue::Volume(volume) => {
                                println!("WSEvent: Volume ({})", volume);
                                let value = volume as f64; 
                                self.spinner_volume.set_value(value);
                            }
                        }
                    }
                }
            }
        }
    }

    fn row1_panel(parent: &mut group::Flex) -> (input::Input, button::Button) {
        let spacer_left = frame::Frame::default();
        let label = frame::Frame::default()
            .with_label("ws://")
            .with_align(enums::Align::Inside | enums::Align::Right);
        let input_address = input::Input::default();
        let button_connect = Self::create_button("Connect");
        let spacer_right = frame::Frame::default();

        parent.set_size(&label, 45);
        parent.set_size(&button_connect, 90);
        parent.set_size(&spacer_left, 10);
        parent.set_size(&spacer_right, 10);
        parent.end();

        (input_address, button_connect)
    }

    fn row2_panel(parent: &mut group::Flex) -> (button::Button, button::Button, button::Button) {
        let spacer_left = frame::Frame::default();
        let button_prev = Self::create_button("Prev track");
        let button_play_pause = Self::create_button("Play / Pause");
        let button_next = Self::create_button("Next track");
        let spacer_right = frame::Frame::default();

        parent.set_size(&button_prev, 95);
        parent.set_size(&button_next, 95);
        parent.set_size(&spacer_left, 10);
        parent.set_size(&spacer_right, 10);
        parent.end();

        (button_prev, button_play_pause, button_next)
    }

    fn row3_panel(parent: &mut group::Flex) -> (button::Button, button::Button) {
        let spacer_left = frame::Frame::default();
        let button_shuffle = Self::create_button("Toggle Shuffle");
        let button_repeat = Self::create_button("Toggle Repeat off / Single song / Whole playlist");
        let spacer_right = frame::Frame::default();

        parent.set_size(&button_shuffle, 130);
        parent.set_size(&spacer_left, 10);
        parent.set_size(&spacer_right, 10);
        parent.end();

        (button_shuffle, button_repeat)
    }

    fn row4_panel(parent: &mut group::Flex) -> (button::Button, misc::Spinner) {
        let spacer_left = frame::Frame::default();
        let button_power = Self::create_button("Power");
        frame::Frame::default();
        let spinner_volume = Self::create_spinner("Volume:  ", 1.0, 0.0, 100.0, 50.0);
        let spacer_right = frame::Frame::default();

        parent.set_size(&button_power, 90);
        parent.set_size(&spinner_volume, 80);
        parent.set_size(&spacer_left, 10);
        parent.set_size(&spacer_right, 10);
        parent.end();

        (button_power, spinner_volume)
    }

    fn create_button(title: &str) -> button::Button {
        let mut btn = button::Button::default().with_label(title);
        btn.set_color(enums::Color::from_rgb(225, 225, 225));
        btn
    }
    
    fn create_spinner(title: &str, step: f64, min: f64, max: f64, value: f64) -> misc::Spinner {
        let mut spin = misc::Spinner::default().with_label(title);
        spin.set_step(step);
        spin.set_range(min, max);
        spin.set_value(value);
        spin.visible_focus(false);
        spin
    }

    fn send_command_to_ws(&mut self, command: &str) {
        if let Some(s) = self.ws_input_sender.borrow().as_ref() {
            s.unbounded_send(Message::text(command)).expect("Could not send through channel");
        }
    }
    
    fn connect_widgets_enable(&mut self, enable: bool, button_text: &str) {
        self.button_connect.set_label(button_text);
        if enable {
            self.input_address.activate();
            self.button_connect.activate();
        } else {
            self.input_address.deactivate();
            self.button_connect.deactivate();
        }
    }

    fn control_widgets_enable(&mut self, enable: bool) {
        if enable {
            self.connect_widgets_enable(false, "Connected");
            self.button_prev.activate();
            self.button_play_pause.activate();
            self.button_next.activate();
            self.button_shuffle.activate();
            self.button_repeat.activate();
            self.button_power.activate();
            self.spinner_volume.activate();
        } else {
            self.connect_widgets_enable(true, "Connect");
            self.button_prev.deactivate();
            self.button_play_pause.deactivate();
            self.button_next.deactivate();
            self.button_shuffle.deactivate();
            self.button_repeat.deactivate();
            self.button_power.deactivate();
            self.spinner_volume.deactivate();
        }
    }
}

#[tokio::main]
async fn main() {
    let mut my_app = MyApp::new();
    my_app.run();
}
