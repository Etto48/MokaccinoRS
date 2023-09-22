use std::{sync::{Arc, RwLock, mpsc::Sender, Mutex}, time::Duration};

use chrono::{Local, DateTime};
use eframe::{egui::{self, Margin, Frame, Label, ScrollArea, Button, TextEdit, CentralPanel, Key, Ui}, epaint::{Vec2, Color32}, NativeOptions};

use crate::{network::{connection_list::ConnectionList, connection_request::ConnectionRequest}, text::{text_list::TextList, text_request::TextRequest, text_info::TextDirection}, thread::context::UnmovableContext, log::{logger::Logger, message_kind::MessageKind}, config::defines};

use super::updater;

pub fn run(
    connection_list: Arc<RwLock<ConnectionList>>,
    text_list: Arc<RwLock<TextList>>,
    log: Logger,
    connection_requests: Sender<ConnectionRequest>,
    text_requests: Sender<TextRequest>,

    unmovable_context: UnmovableContext
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

            unmovable_context,
            cc,
        ))))
    {
        panic!("Error starting GUI: {}", e)
    }
}

pub struct UI
{
    input_buffer: String,
    active_contact: Option<String>,

    connection_list: Arc<RwLock<ConnectionList>>,
    text_list: Arc<RwLock<TextList>>,
    log: Logger,
    connection_requests: Sender<ConnectionRequest>,
    text_requests: Sender<TextRequest>,

    unmovable_context: UnmovableContext,
}

impl UI
{
    pub fn new(
        connection_list: Arc<RwLock<ConnectionList>>,
        text_list: Arc<RwLock<TextList>>,
        log: Logger,
        connection_requests: Sender<ConnectionRequest>,
        text_requests: Sender<TextRequest>,
        unmovable_context: UnmovableContext,
        cc: &eframe::CreationContext<'_>
    ) -> Self
    {
        let ctx = cc.egui_ctx.clone();
        let running_clone = unmovable_context.running.clone();
        let handle = std::thread::spawn(move ||
        {
            updater::run(ctx,running_clone);
        });
        
        Self { 
            input_buffer: String::new(), 
            active_contact: None, 
            connection_list, 
            text_list, 
            log,
            connection_requests, 
            text_requests,
            unmovable_context,
        }
    }
}

impl eframe::App for UI
{
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let margin = 10.0;
        let confirm_button_width = 80.0;
        let selected_color = Color32::from_rgb(116, 77, 169);
        let size = frame.info().window_info.size;
        CentralPanel::default()
        .frame(Frame{
            inner_margin: Margin::same(margin),
            outer_margin: Margin::same(0.0),
            ..Default::default()
        }).show(ctx, |ui| {
            //add a vertical space that spans the entire height of the window
            ui.horizontal(|ui|{
                let group_margin = margin + 3.0;
                let left_group_width = size.x * 0.2 - 2.0*group_margin;
                let input_height = 35.0;
                let left_group_min_width = 150.0 - 2.0*group_margin;
                ui.group(|ui|{
                    ui.set_height(size.y - 2.0*group_margin);
                    ui.set_min_width(left_group_min_width);
                    ui.set_width(left_group_width);
                    //contacts
                    ScrollArea::vertical()
                    .auto_shrink([false;2])
                    .show(ui, |ui|{
                        ui.vertical(|ui|{
                            let mut contacts = 
                            {
                                let connection_list = self.connection_list.read().expect("I sure hope there is no poisoning here");
                                connection_list.get_names()
                            };
                            contacts.sort_by(|c1,c2|{
                                c1.cmp(c2)    
                            });
                            { // add system button
                                let mut button = Button::new("System");
                                if self.active_contact == None
                                {
                                    button = button.fill(selected_color);
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
                                let mut button = Button::new(&c);
                                if self.active_contact == Some(c.clone())
                                {
                                    button = button.fill(selected_color);
                                }
                                if ui.add_sized(
                                    Vec2::new(ui.available_width(),20.0), 
                                    button).clicked()
                                {
                                    // user selected
                                    self.active_contact = Some(c.clone());
                                }
                            }
                        });
                    });
                   
                });
                ui.vertical(|ui|{
                    ui.group(|ui|{
                        ui.set_height(size.y - 2.0*group_margin - input_height);
                        ui.set_width(ui.available_width());
                        ScrollArea::vertical()
                        .auto_shrink([false;2])
                        .stick_to_bottom(true)
                        .show(ui, |ui|{
                            //chat
                            if let Some(c) = &self.active_contact
                            {
                                let text_list = self.text_list.read().expect("I sure hope there is no poisoning here");
                                if let Some(messages) = text_list.get(c)
                                {
                                    for m in messages
                                    {    
                                        ui.horizontal(|ui|{
                                            
                                            ui.label(format!("{}:",
                                            if m.direction == TextDirection::Incoming
                                            {
                                                c
                                            }
                                            else
                                            {
                                                "You"
                                            }));
                                            ui.add(Label::new(m.text.clone()).wrap(true));
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
                                            MessageKind::Event => defines::LOG_EVENT_COLOR,
                                            MessageKind::Command => defines::LOG_COMMAND_COLOR,
                                            MessageKind::Error => defines::LOG_ERROR_COLOR, 
                                        };
                                        let text = match m.kind {
                                            MessageKind::Command =>  format!("{} ({}) Command:",time_string,m.src),
                                            MessageKind::Event =>  format!("{} ({}):",time_string,m.src),
                                            MessageKind::Error =>  format!("{} ({}) Error:",time_string,m.src),
                                        };
                                        ui.colored_label(color,text);
                                        ui.add(Label::new(m.text.clone()).wrap(true));
                                    });
                                }
                            }
                        });
                    });
                    ui.group(|ui|{
                        ui.set_width(ui.available_width());
                        ui.set_height(ui.available_height());
                        ui.horizontal(|ui|{
                            //text input
                            //press button enter if enter key is pressed with text input focused
                            let text_hint = 
                                if let Some(c) = &self.active_contact
                                {"Type a message"}
                                else 
                                {"Type a command"};

                            let text_edit = ui.add_sized(
                                Vec2::new(
                                    ui.available_width() - confirm_button_width,
                                    ui.available_height()),
                                TextEdit::singleline(&mut self.input_buffer)
                                .hint_text(text_hint));
                            
                            if (ui.add(|ui: &mut Ui|{
                                ui.add_sized(
                                Vec2::new(ui.available_width(),ui.available_height()), 
                                Button::new("Send"))}).clicked() 
                                
                                || ui.input(|i|i.key_pressed(Key::Enter)))
                                && self.input_buffer.len()>0
                            {
                                if let Some(c) = &self.active_contact
                                {
                                    // sent a message
                                    let is_connected = {
                                        let connection_list = self.connection_list.read().expect("I sure hope there is no poisoning here");
                                        connection_list.get_address(c).is_some()
                                    };
                                    if is_connected
                                    {
                                        self.text_requests.send(TextRequest { text: self.input_buffer.clone(), dst: c.clone() }).expect("Please don't crush now");
                                    }
                                }
                                else
                                {
                                    // sent command
                                    self.log.log(MessageKind::Command,&self.input_buffer).unwrap();
                                    //todo!("Parse command");
                                    //CHANGE THIS
                                    self.connection_requests.send(ConnectionRequest::Connect(self.input_buffer.parse().expect("Pls no"))).expect("Please don't crush now");
                                }
                                self.input_buffer.clear();
                                //set focus to text input
                                text_edit.request_focus();
                            }
                        });
                    });
                });
            });
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        *self.unmovable_context.running.write().unwrap() = false;
    }
}