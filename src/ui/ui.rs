use std::{sync::{Arc, RwLock, mpsc::{Sender, Receiver}, Mutex}, time::Duration, net::{SocketAddr, IpAddr}};

use chrono::{Local, DateTime};
use cpal::traits::{HostTrait, DeviceTrait};
use eframe::{egui::{self, Margin, Frame, Label, ScrollArea, Button, TextEdit, CentralPanel, Key, Ui, Slider, Style, Visuals, style::Selection, ComboBox, TextureOptions, ImageButton, Layout, load::SizedTexture}, epaint::{Vec2, Rounding, Stroke, TextureHandle}, NativeOptions, emath::{Align2, Align}, CreationContext};

use crate::{network::{ConnectionList, ConnectionRequest}, text::{TextList, TextRequest, TextDirection}, thread::context::UnmovableContext, log::{Logger, MessageKind}, config::defines, voice::VoiceRequest};

use crate::load_image;

use super::UiNotification;

pub fn run(
    connection_list: Arc<RwLock<ConnectionList>>,
    text_list: Arc<RwLock<TextList>>,
    log: Logger,
    connection_requests: Sender<ConnectionRequest>,
    text_requests: Sender<TextRequest>,
    voice_requests: Sender<VoiceRequest>,
    voice_interlocutor: Arc<Mutex<Option<SocketAddr>>>,
    ui_notifications: Receiver<UiNotification>,

    unmovable_context: UnmovableContext,
)
{
    let options = NativeOptions{
        initial_window_size: Some(Vec2::new(800.0, 500.0)),
        min_window_size: Some(Vec2::new(400.0, 300.0)),
        icon_data: Some(super::load_icon::load_icon()),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "Mokaccino",
        options, 
        Box::new(|cc| Box::new(UI::new(
            connection_list,
            text_list,
            log,
            connection_requests,
            text_requests,
            voice_requests,
            voice_interlocutor,
            ui_notifications,

            unmovable_context,
            cc
        ))))
    {
        panic!("Error starting GUI: {}", e)
    }
}

pub struct UI
{
    input_buffer: String,
    new_connection_address_buffer: String,
    new_connection_port_buffer: String,
    settings_port_buffer: String,

    active_contact: Option<String>,

    connection_list: Arc<RwLock<ConnectionList>>,
    text_list: Arc<RwLock<TextList>>,
    log: Logger,
    connection_requests: Sender<ConnectionRequest>,
    text_requests: Sender<TextRequest>,
    voice_requests: Sender<VoiceRequest>,
    voice_interlocutor: Arc<Mutex<Option<SocketAddr>>>,

    ui_notifications: Receiver<UiNotification>,

    unmovable_context: UnmovableContext,

    show_new_connection_dialog: bool,
    show_settings_dialog: bool,
    show_incoming_call_dialog: Option<String>,

    input_devices: Vec<String>,
    output_devices: Vec<String>,

    settings_image_dark: TextureHandle,
    settings_image_light: TextureHandle,
    voice_image_dark: TextureHandle,
    voice_image_light: TextureHandle,
    send_image_dark: TextureHandle,
    send_image_light: TextureHandle,
}

impl UI
{
    pub fn new(
        connection_list: Arc<RwLock<ConnectionList>>,
        text_list: Arc<RwLock<TextList>>,
        log: Logger,
        connection_requests: Sender<ConnectionRequest>,
        text_requests: Sender<TextRequest>,
        voice_requests: Sender<VoiceRequest>,
        voice_interlocutor: Arc<Mutex<Option<SocketAddr>>>,
        ui_notifications: Receiver<UiNotification>,
        unmovable_context: UnmovableContext,
        cc: &CreationContext
    ) -> Self
    {   
        let host = cpal::default_host();
        let input_devices = host.input_devices().unwrap();
        let output_devices = host.output_devices().unwrap();
        let mut input_devices_names = input_devices.map(|d| d.name().unwrap_or("Unknown device".to_string())).collect::<Vec<_>>();
        let mut output_devices_names = output_devices.map(|d| d.name().unwrap_or("Unknown device".to_string())).collect::<Vec<_>>();
        input_devices_names.insert(0, "Default".to_string());
        output_devices_names.insert(0, "Default".to_string());
        let settings_port_buffer = unmovable_context.config.read().unwrap().network.port.to_string();
        let settings_image_dark = cc.egui_ctx.load_texture("SettingsDark", 
            load_image!("../../assets/settings_dark.png"),
            TextureOptions::default());
        let settings_image_light = cc.egui_ctx.load_texture("SettingsLight", 
            load_image!("../../assets/settings_light.png"),
            TextureOptions::default());
        let voice_image_dark = cc.egui_ctx.load_texture("VoiceDark", 
            load_image!("../../assets/voice_dark.png"),
            TextureOptions::default());
        let voice_image_light = cc.egui_ctx.load_texture("VoiceLight", 
            load_image!("../../assets/voice_light.png"),
            TextureOptions::default());
        let send_image_dark = cc.egui_ctx.load_texture("SendDark", 
            load_image!("../../assets/send_dark.png"),
            TextureOptions::default());
        let send_image_light = cc.egui_ctx.load_texture("SendLight",
            load_image!("../../assets/send_light.png"),
            TextureOptions::default());
        Self { 
            input_buffer: String::new(), 
            new_connection_address_buffer: String::new(),
            new_connection_port_buffer: String::new(),
            settings_port_buffer,
            active_contact: None, 
            connection_list, 
            text_list, 
            log,
            connection_requests, 
            text_requests,
            voice_requests,
            voice_interlocutor,
            ui_notifications,
            unmovable_context,
            show_new_connection_dialog: false,
            show_settings_dialog: false,
            show_incoming_call_dialog: None,
            input_devices: input_devices_names,
            output_devices: output_devices_names,
            settings_image_dark,
            settings_image_light,
            voice_image_dark,
            voice_image_light,
            send_image_dark,
            send_image_light,
        }
    }

    fn validate_new_connection_address(&self) -> bool
    {
        self.new_connection_address_buffer.parse::<IpAddr>().is_ok()
    }

    fn validate_new_connection_port(&self) -> bool
    {
        self.new_connection_port_buffer.parse::<u16>().is_ok()
    }

    fn save_config(&self)
    {
        // save config to file
        match self.unmovable_context.config.write()
        {
            Ok(config) => 
            {
                match config.to_file(defines::CONFIG_PATH)
                {
                    Ok(_) => (),
                    Err(e) => 
                    {
                        println!("Error saving config: {}",e);
                    },
                }
            },
            Err(e) => 
            {
                println!("Error saving config: {}",e);
            },
        }
    }

    fn add_contacts(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        group_margin: f32,
        input_height: f32,
        left_group_width: f32,
        left_group_min_width: f32,
        accent_color: egui::Color32,
    )
    {
        ui.set_height(size.y - 2.0*group_margin - input_height);
        ui.set_min_width(left_group_min_width);
        ui.set_width(left_group_width);
        //contacts
        ScrollArea::vertical()
        .id_source("ContactsScrollArea")
        .auto_shrink([false;2])
        .show(ui, |ui|{
            ui.vertical(|ui|{
                let mut contacts = 
                {
                    let connection_list = self.connection_list.read().unwrap();
                    if let Some(name) = &self.active_contact
                    {
                        if connection_list.get_address(name).is_none()
                        {
                            self.active_contact = None;
                        }
                    }
                    connection_list.get_names()
                };
                contacts.sort_by(|c1,c2|{
                    c1.cmp(c2)    
                });
                { // add system button
                    let has_new_messages = self.log.has_new_messages().unwrap();
                    let button_text = if has_new_messages {"System*"} else {"System"};
                    let mut button = Button::new(button_text);
                    if self.active_contact == None
                    {
                        button = button.fill(accent_color);
                    }
                    if ui.add_sized(
                        Vec2::new(ui.available_width(),20.0), 
                        button).clicked()
                    {
                        // system selected
                        self.active_contact = None;
                    }
                }
                for c in contacts
                {
                    let has_new_messages = 
                    {
                        let text_list = self.text_list.read().unwrap();
                        text_list.has_new_messages(&c)
                    };
                    let button_text = 
                    {
                        if has_new_messages
                        {
                            format!("{}*",c)
                        }
                        else 
                        {
                            c.clone()    
                        }
                    };
                    ui.horizontal(|ui|{
                        ui.style_mut().spacing.item_spacing.x = 4.0;
                        let mut button = Button::new(&button_text)
                        .rounding(Rounding{nw: 3.0, ne: 0.0, sw: 3.0, se: 0.0});
                        if self.active_contact == Some(c.clone())
                        {
                            button = button.fill(accent_color);
                        }
                        if ui.add_sized(
                            Vec2::new(ui.available_width() - 28.0,20.0), 
                            button).clicked()
                        {
                            // user selected
                            self.active_contact = Some(c.clone());
                        }
                        let mut button = Button::new("x")
                        .rounding(Rounding{nw: 0.0, ne: 3.0, sw: 0.0, se: 3.0});
                        if self.active_contact == Some(c.clone())
                        {
                            button = button.fill(accent_color);
                        }
                        if ui.add_sized(
                            Vec2::new(ui.available_width(),20.0), 
                            button).clicked()
                        {
                            // disconnect user
                            self.connection_requests.send(ConnectionRequest::Disconnect(c.clone())).unwrap();
                            if self.active_contact == Some(c.clone())
                            {
                                self.active_contact = None;
                            }
                        }
                    });
                }
                { // add new contact button
                    let button = Button::new("+");
                    if ui.add_sized(
                        Vec2::new(ui.available_width(),20.0), 
                        button).clicked()
                    {
                        // system selected
                        self.show_new_connection_dialog = true;
                    }
                }
            });
        });   
    }

    fn add_chat(
        &mut self,
        ui: &mut Ui,
        size: Vec2,
        group_margin: f32,
        input_height: f32,
        text_color: egui::Color32,
    )
    {
        ui.set_height(size.y - 2.0*group_margin - input_height);
        ui.set_width(ui.available_width());
        ScrollArea::vertical()
        .id_source("ChatScrollArea")
        .auto_shrink([false;2])
        .stick_to_bottom(true)
        .show(ui, |ui|{
            //chat
            if let Some(c) = &self.active_contact
            {
                let mut text_list = self.text_list.write().unwrap();
                if let Some(messages) = text_list.get(c)
                {
                    for m in messages
                    {    
                        ui.horizontal(|ui|{
                            
                            ui.label(format!("{}:",
                            if m.direction == TextDirection::Incoming {c} else {"You"}));
                            ui.add(Label::new(m.text.clone())
                                .wrap(true)
                            );
                        });
                    }
                }
            }
            else
            {
                let messages = self.log.get().unwrap();
                for m in messages
                {
                    ui.horizontal(|ui|{
                        let time_string = DateTime::<Local>::from(m.time).format("%H:%M:%S").to_string();
                        let color = match m.kind
                        {
                            MessageKind::Event => text_color,
                            MessageKind::Command => defines::LOG_COMMAND_COLOR,
                            MessageKind::Error => defines::LOG_ERROR_COLOR, 
                        };
                        let text = match m.kind {
                            MessageKind::Command =>  format!("{} ({}) Command:",time_string,m.src),
                            MessageKind::Event =>  format!("{} ({}):",time_string,m.src),
                            MessageKind::Error =>  format!("{} ({}) Error:",time_string,m.src),
                        };
                        ui.colored_label(color,text);
                        ui.add(Label::new(m.text.clone())
                            .wrap(true)
                        );
                    });
                }
            }
        });
    }

    fn add_input(
        &mut self,
        ui: &mut Ui,
        send_image: SizedTexture,
    )
    {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());
        ui.horizontal(|ui|{
            //text input
            ui.with_layout(Layout::right_to_left(Align::Center), |ui|{
                let mut just_sent = false;
                // send button
                if (ui.add(ImageButton::new(send_image)).clicked() 
                    || ui.input(|i|i.key_pressed(Key::Enter)))
                    && self.input_buffer.len()>0
                {
                    if let Some(c) = &self.active_contact
                    {
                        // sent a message
                        let is_connected = {
                            let connection_list = self.connection_list.read().unwrap();
                            connection_list.get_address(c).is_some()
                        };
                        if is_connected
                        {
                            self.text_requests.send(TextRequest { text: self.input_buffer.clone(), dst: c.clone() }).unwrap();
                        }
                    }
                    else
                    {
                        // sent command
                        self.log.log(MessageKind::Command,&self.input_buffer).unwrap();
                        todo!("Parse command");
                    }
                    self.input_buffer.clear();
                    //set focus to text input
                    just_sent = true;
                }
                //press button enter if enter key is pressed with text input focused
                let text_hint = 
                    if let Some(_c) = &self.active_contact
                    {"Type a message"}
                    else 
                    {"Type a command"};

                let text_edit = ui.add_sized(
                    Vec2::new(
                        ui.available_width(),
                        ui.available_height()),
                    TextEdit::singleline(&mut self.input_buffer)
                    //.vertical_align(Align::Max) // it's a bit buggy as it does not align the hint text
                    .hint_text(text_hint)
                );
                if just_sent
                {
                    text_edit.request_focus();
                }
            });
        });
    }

    fn add_actions(
        &mut self,
        ui: &mut Ui,
        left_group_width: f32,
        left_group_min_width: f32,
        accent_color: egui::Color32,
        settings_image: SizedTexture,
        voice_image: SizedTexture,
    )
    {
        ui.horizontal(|ui|{
            ui.set_height(ui.available_height());
            ui.set_min_width(left_group_min_width);
            ui.set_width(left_group_width);
            //align buttons to the right
            ui.with_layout(Layout::right_to_left(Align::Center), |ui|{
                if ui.add(ImageButton::new(settings_image))
                .clicked()
                {
                    self.show_settings_dialog = true;
                }
                if let Some(contact) = &self.active_contact
                {
                    let connection_list = self.connection_list.read().unwrap();
                    if let Some(address) = connection_list.get_address(contact)
                    {
                        let button_color = {
                            if let Some(voice_interlocutor) = *self.voice_interlocutor.lock().unwrap()
                            {
                                if voice_interlocutor == *address
                                {Some(accent_color)} else {None}
                            } else {None}
                        };
                        let button = ImageButton::new(voice_image);
                        if ui.add(
                            |ui: &mut Ui|{
                                if let Some(color) = button_color
                                {
                                    ui.visuals_mut().selection.bg_fill = color;
                                    ui.style_mut().visuals.widgets.active.weak_bg_fill = color;
                                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill = color;
                                    ui.style_mut().visuals.widgets.open.weak_bg_fill = color;
                                    ui.style_mut().visuals.widgets.inactive.weak_bg_fill = color;
                                }
                                ui.add(button)
                            })
                        .clicked()
                        {
                            if button_color.is_none()
                            { // not yet in voice chat
                                self.voice_requests.send(VoiceRequest::StartTransmission(*address)).unwrap();
                            }
                            else {
                                self.voice_requests.send(VoiceRequest::StopTransmission(*address)).unwrap();
                            }
                        }
                    }
                }
            });
        });
    }

    fn show_settings(
        &mut self, 
        window_frame: Frame,
        ctx: &egui::Context)
    {
        let mut save_config = false;
        egui::Window::new("Settings")
        .frame(window_frame)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::new(0.0,0.0))
        .show(ctx,|ui|{
            let mut config = self.unmovable_context.config.write().unwrap();
            {//Network
                ui.label("Network");
                ui.group(|ui|{
                    ui.set_width(ui.available_width());
                    ui.label("Name");
                    ui.add_sized(
                        Vec2::new(ui.available_width(),20.0),
                        TextEdit::singleline(&mut config.network.name));
                    ui.label("Port");
                    ui.add_sized(
                        Vec2::new(ui.available_width(),20.0),
                        TextEdit::singleline(&mut self.settings_port_buffer));
                    if let Ok(port) = self.settings_port_buffer.parse::<u16>()
                    {
                        config.network.port = port;
                    }
                });
            }
            {//Voice
                ui.label("Voice");
                ui.group(|ui|{

                    ui.style_mut().spacing.slider_width = ui.available_width() - 10.0;
                    ui.style_mut().spacing.combo_width = ui.available_width() - 10.0;

                    ui.label("Input device");
                    let input_selected_text = 
                        if let Some(name) = config.voice.input_device.clone()
                        {name} else {"Default".to_string()};
                    ComboBox::new(
                        "InputDeviceComboBox",
                        "",    
                    )
                    .selected_text(input_selected_text)
                    .show_ui(ui, |ui|{
                        for name in self.input_devices.iter()
                        {
                            if ui.selectable_value(&mut config.voice.input_device, Some(name.clone()), name.clone()).clicked()
                            {
                                if name == "Default"
                                {
                                    config.voice.input_device = None;
                                }
                                else
                                {
                                    config.voice.input_device = Some(name.clone());
                                }
                                self.voice_requests.send(VoiceRequest::ReloadConfiguration).unwrap();
                            }
                        }
                    });
                    ui.label("Output device");
                    let output_selected_text = 
                        if let Some(name) = config.voice.output_device.clone()
                        {name} else {"Default".to_string()};
                    ComboBox::new(
                        "OutputDeviceComboBox",
                        "",
                    ).selected_text(output_selected_text)
                    .show_ui(ui, |ui|{
                        for name in self.output_devices.iter()
                        {
                            if ui.selectable_value(&mut config.voice.output_device, Some(name.clone()), name.clone()).clicked()
                            {
                                if name == "Default"
                                {
                                    config.voice.output_device = None;
                                }
                                else
                                {
                                    config.voice.output_device = Some(name.clone());
                                }
                                self.voice_requests.send(VoiceRequest::ReloadConfiguration).unwrap();
                            }
                        }
                    });
                    ui.label("Gain");
                    ui.add(
                        Slider::new(
                        &mut config.voice.gain,
                        defines::MIN_GAIN..=defines::MAX_GAIN)
                        .show_value(false)
                        .clamp_to_range(true));
                });
            }
            if ui.add_sized(
                Vec2::new(ui.available_width(),20.0),
                Button::new("Close")).clicked()
            {
                self.show_settings_dialog = false;
                save_config = true;
            }
        });
        if save_config
        {
            self.save_config();
        }
    }

    fn show_new_connection(
        &mut self, 
        window_frame: Frame,
        ctx: &egui::Context, 
        accent_color: egui::Color32)
    {
        let text_color_addr = 
            if self.validate_new_connection_address() {None} else {Some(egui::Color32::RED)};
        let text_color_port = 
            if self.validate_new_connection_port() {None} else {Some(egui::Color32::RED)};
        egui::Window::new("Connect")
        .frame(window_frame)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::new(0.0,0.0))
        .show(ctx,|ui|{
            ui.horizontal(|ui|{
                ui.add_sized(Vec2::new(250.0,20.0),TextEdit::singleline(&mut self.new_connection_address_buffer)
                .hint_text("Address")
                .text_color_opt(text_color_addr));
                ui.add_sized(Vec2::new(ui.available_width(),20.0),TextEdit::singleline(&mut self.new_connection_port_buffer)
                .hint_text("Port")
                .text_color_opt(text_color_port));
            });
            ui.horizontal(|ui|{
                let mut close_window = false;
                
                if ui.add_sized(
                    Vec2::new(ui.available_width()/2.0,20.0),
                    Button::new("Connect").fill(accent_color))
                    .clicked() ||
                    ui.input(|i| i.key_pressed(Key::Enter))
                {
                    let address_string = format!("{}:{}",self.new_connection_address_buffer,self.new_connection_port_buffer);
                    if self.validate_new_connection_address() && self.validate_new_connection_port()
                    {
                        match address_string.parse::<SocketAddr>() {
                            Ok(address) => 
                            {
                                self.connection_requests.send(ConnectionRequest::Connect(address)).unwrap();
                                close_window = true;
                            },
                            Err(_) => {
                                //this should not be rachable but just in case, ingnore the input
                            },
                        }
                    }   
                }
                if ui.add_sized(
                    Vec2::new(ui.available_width(),20.0),
                    Button::new("Cancel")).clicked() ||
                    ui.input(|i| i.key_pressed(Key::Escape))
                {
                    close_window = true;
                }
                if close_window
                {
                    self.new_connection_address_buffer.clear();
                    self.new_connection_port_buffer.clear();
                    self.show_new_connection_dialog = false;
                }
            });
        });
    }

    fn show_incoming_call(
        &mut self, 
        from: String,
        window_frame: Frame,
        ctx: &egui::Context, 
        accent_color: egui::Color32)
    {
        
        egui::Window::new(format!("{} is calling!",from))
        .frame(window_frame)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::new(0.0,0.0))
        .show(ctx, |ui|{
            ui.horizontal(|ui|{
                if ui.add(Button::new("Accept")
                .fill(accent_color))
                .clicked()
                {
                    let connection_list = self.connection_list.read().unwrap();
                    if let Some(address) = connection_list.get_address(&from)
                    {
                        self.voice_requests.send(VoiceRequest::StartTransmission(address.to_owned())).unwrap()
                    }
                    self.show_incoming_call_dialog = None;
                }
                if ui.button("Decline").clicked()
                {
                    let connection_list = self.connection_list.read().unwrap();
                    if let Some(address) = connection_list.get_address(&from)
                    {
                        self.voice_requests.send(VoiceRequest::StopTransmission(address.to_owned())).unwrap()
                    }
                    self.show_incoming_call_dialog = None;
                }
            });
        });
    }

    fn handle_notifications(&mut self)
    {
        while let Ok(notification) = self.ui_notifications.try_recv()
        {
            match notification 
            {
                UiNotification::IncomingConnection(_) => todo!(),
                UiNotification::IncomingCall(from) => 
                {
                    if self.voice_interlocutor.lock().unwrap().is_none()
                    {
                        self.show_incoming_call_dialog = Some(from);
                    }
                },
            }
        }
    }
}

impl eframe::App for UI
{
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.handle_notifications();
        let margin = 10.0;
        let size = frame.info().window_info.size;
        let image_size = 14.0;
        let (
            background_color,
            accent_color,
            text_color,
            settings_image,
            voice_image,
            send_image,
        ) = 
            if ctx.style().visuals.dark_mode
            {(
                defines::FRAME_COLOR_DARK,
                defines::ACCENT_COLOR_DARK,
                defines::TEXT_COLOR_DARK,
                &self.settings_image_dark,
                &self.voice_image_dark,
                &self.send_image_dark,
            )} 
            else {(
                defines::FRAME_COLOR_LIGHT,
                defines::ACCENT_COLOR_LIGHT,
                defines::TEXT_COLOR_LIGHT,
                &self.settings_image_light,
                &self.voice_image_light,
                &self.send_image_light,
            )};

        let settings_image = SizedTexture::new(settings_image, Vec2::new(image_size,image_size));
        let voice_image = SizedTexture::new(voice_image, Vec2::new(image_size,image_size));
        let send_image = SizedTexture::new(send_image, Vec2::new(image_size,image_size));
        ctx.set_style(Style
        {
            override_font_id: Some(egui::FontId::monospace(12.0)),
            visuals: Visuals
            {
                override_text_color: Some(text_color),
                selection: Selection{
                    bg_fill: accent_color,
                    ..ctx.style().visuals.selection.clone()
                },
                ..ctx.style().visuals.clone()
            },
            ..(*ctx.style()).clone()
        });

        let window_frame = Frame{
            inner_margin: Margin::same(margin),
            outer_margin: Margin::same(0.0),
            rounding: Rounding::same(5.0),
            fill: background_color,
            stroke: Stroke::new(1.0, accent_color),
            ..Default::default()
        };

        CentralPanel::default()
        .frame(Frame{
            inner_margin: Margin::same(margin),
            outer_margin: Margin::same(0.0),
            fill: background_color,
            ..Default::default()
        }).show(ctx, |ui| {
            ui.horizontal(|ui|{
                let group_margin = margin + 5.0;
                let left_group_width = size.x * 0.2 - 2.0*group_margin;
                let input_height = 35.0;
                let left_group_min_width = 150.0 - 2.0*group_margin;
                ui.vertical(|ui|{
                    ui.group(|ui|{
                        self.add_contacts(ui,size, group_margin, input_height, left_group_width, left_group_min_width, accent_color)
                    });
                    ui.group(|ui|{
                        self.add_actions(ui, left_group_width, left_group_min_width, accent_color, settings_image, voice_image)
                    });
                });
                ui.vertical(|ui|{
                    ui.group(|ui|{
                        self.add_chat(ui, size, group_margin, input_height, text_color)
                    });
                    ui.group(|ui|{
                        self.add_input(ui, send_image)
                    });
                });
            });
        });

        if self.show_new_connection_dialog
        {
            self.show_new_connection(window_frame, ctx, accent_color);
        }

        if self.show_settings_dialog
        {
            self.show_settings(window_frame, ctx);
        }

        if let Some(from) = &self.show_incoming_call_dialog
        {
            self.show_incoming_call(from.clone(), window_frame, ctx, accent_color);
        }

        ctx.request_repaint_after(Duration::from_millis(defines::UPDATE_UI_INTERVAL_MS));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_config();
        // stop the other threads
        self.unmovable_context.stop();
    }
}