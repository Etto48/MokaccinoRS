use std::collections::HashMap;

use eframe::{egui::{self, Margin, Frame, Label, ScrollArea, Button, TextEdit, CentralPanel, Key, Ui}, epaint::{Vec2, Color32}, NativeOptions};

use super::{MessageInfo, MessageDirection, ContactInfo};

pub fn run()
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
        Box::new(|_cc| Box::<UI>::default()))
    {
        panic!("Error starting GUI: {}", e)
    }
}

pub struct UI
{
    input_buffer: String,
    messages: HashMap<ContactInfo,Vec<MessageInfo>>,
    active_contact: Option<ContactInfo>,
}

impl Default for UI
{
    fn default() -> Self {
        let system = ContactInfo::new("System");
        let messages = HashMap::from_iter(
            [
                (system.clone(),vec![MessageInfo::new("Welcome to Mokaccino!",MessageDirection::Incoming)]),
                (ContactInfo::new("Pino"),vec![])
            ],
        );
        Self { 
            input_buffer: String::new(), 
            messages, 
            active_contact: Some(system), 
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
                            let mut contacts = self.messages.keys().collect::<Vec<&ContactInfo>>();
                            contacts.sort_by(|c1,c2|{
                                if c1.name() == "System"
                                {
                                    std::cmp::Ordering::Less
                                }
                                else if c2.name() == "System" 
                                {
                                    std::cmp::Ordering::Greater
                                }
                                else {
                                    c1.name().cmp(c2.name())    
                                }
                            });
                            for c in contacts
                            {
                                let mut button = Button::new(c.name());
                                if self.active_contact == Some(c.clone())
                                {
                                    button = button.fill(selected_color);
                                }
                                if ui.add_sized(
                                    Vec2::new(ui.available_width(),20.0), 
                                    button).clicked()
                                {
                                    println!("{} selected",c.name());
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
                                if let Some(messages) = self.messages.get(c)
                                {
                                    for m in messages
                                    {    
                                        ui.horizontal(|ui|{
                                            
                                            ui.label(format!("{}:",
                                            if m.direction() == MessageDirection::Incoming
                                            {
                                                c.name()
                                            }
                                            else
                                            {
                                                "You"
                                            }));
                                            ui.add(Label::new(m.text()).wrap(true));
                                        });
                                    }
                                }
                                else {
                                    //the active contact was removed from the list of contacts
                                    self.active_contact = None;
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
                            let mut send_enabled = true;
                            let text_hint = 
                            if let Some(c) = &self.active_contact
                            {
                                if c.name() == "System"
                                {
                                    "Type a command"
                                }
                                else {
                                    "Type a message"
                                }
                            }
                            else 
                            {
                                send_enabled = false;
                                "No active contact"
                            };

                            let text_edit = ui.add_sized(
                                Vec2::new(
                                    ui.available_width() - confirm_button_width,
                                    ui.available_height()),
                                TextEdit::singleline(&mut self.input_buffer)
                                .hint_text(text_hint));
                            
                            if (ui.add_enabled(send_enabled, |ui: &mut Ui|{
                                ui.add_sized(
                                Vec2::new(ui.available_width(),ui.available_height()), 
                                Button::new("Send"))}).clicked() 
                                
                                || ui.input(|i|i.key_pressed(Key::Enter)))
                                && self.input_buffer.len()>0
                            {
                                if let Some(c) = &self.active_contact
                                {
                                    if c.name() == "System"
                                    {
                                        println!("Sent command: {}",self.input_buffer);
                                    }
                                    else 
                                    {    
                                        println!("Sent message: {}",self.input_buffer);
                                    }
                                    if let Some(messages) = self.messages.get_mut(c)
                                    {
                                        messages.push(MessageInfo::new(
                                            &self.input_buffer,
                                            MessageDirection::Outgoing));
                                    }
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
}