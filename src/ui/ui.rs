use eframe::{egui::{self, Margin, Frame, Label, ScrollArea, Button, TextEdit, CentralPanel, Key}, epaint::Vec2};

pub struct UI
{
    input_buffer: String
}

impl Default for UI
{
    fn default() -> Self {
        Self { input_buffer: String::new() }
    }
}

impl eframe::App for UI
{
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let margin = 10.0;
        let confirm_button_width = 80.0;
        CentralPanel::default()
        .frame(Frame{
            inner_margin: Margin::same(margin),
            outer_margin: Margin::same(0.0),
            ..Default::default()
        }).show(ctx, |ui| {
            let size = frame.info().window_info.size;
            //add a vertical space that spans the entire height of the window
            ui.horizontal(|ui|{
                let group_margin = margin + 3.0;
                let left_group_width = size.x * 0.2 - 2.0*group_margin;
                let input_height = 35.0;
                ui.group(|ui|{
                    ui.set_height(size.y - 2.0*group_margin);
                    ui.set_min_width(200.0 - 2.0*group_margin);
                    ui.set_width(left_group_width);
                    ui.vertical(|ui|{
                        for i in 0..10
                        {
                            if ui.add_sized(
                                Vec2::new(ui.available_width(),20.0), 
                                Button::new(format!("Contact_{i}"))).clicked()
                            {
                                println!("Contact_{i} clicked");
                            }
                        }
                    });
                    //contacts
                });
                ui.vertical(|ui|{
                    ui.group(|ui|{
                        ui.set_height(size.y - 2.0*group_margin - input_height);
                        ui.set_width(ui.available_width());
                        ScrollArea::vertical()
                        .auto_shrink([false;2])
                        .show(ui, |ui|{
                            //chat
                            for i in 0..10
                            {
                                ui.horizontal(|ui|{
                                    if ui.add_sized(
                                        Vec2::new(60.0,20.0),
                                        Button::new(format!("Contact_{i}")).small()).clicked()
                                    {
                                        println!("Contact_{i} clicked");
                                    }
                                    ui.label(":");
                                    ui.vertical(|ui|{
                                        ui.add_space(4.0);
                                        ui.add(Label::new("Message bla bla".repeat(i+1)).wrap(true));
                                    });
                                });
                            }
                        });
                    });
                    ui.group(|ui|{
                        ui.set_width(ui.available_width());
                        ui.set_height(ui.available_height());
                        ui.horizontal(|ui|{
                            //text input
                            //press button enter if enter key is pressed with text input focused
                            let text_edit = ui.add_sized(
                                Vec2::new(
                                    ui.available_width() - confirm_button_width,
                                    ui.available_height()),
                                TextEdit::singleline(&mut self.input_buffer)
                                .hint_text("Input..."));
                            if (ui.add_sized(
                                Vec2::new(ui.available_width(),ui.available_height()), 
                                Button::new("Enter")).clicked() 
                                || ui.input(|i|i.key_pressed(Key::Enter)))
                                && self.input_buffer.len()>0
                            {
                                println!("Enter pressed, input: {}", self.input_buffer);
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