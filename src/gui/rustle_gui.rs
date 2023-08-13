use std::collections::HashMap;
use std::sync::Arc;

use crate::download_utils::downloader::{RustleDownloader, ResponseHeaderInfo, PartDownloadInfo, DownloadStatus};

use iced::widget::{ProgressBar, Text, Button, Container, Column, Row, TextInput, Scrollable};
use iced::{theme, 
        Alignment,
        Element,
        Application,
        Length, 
        Command, 
        Theme, 
        alignment::Horizontal
        };

use iced_aw::floating_element::Anchor;
use iced_aw::{Badge, Icon, ICON_FONT, FloatingElement, Modal, Card, Spinner};
use iced_aw::style::BadgeStyles;
use super::utils::format_file_size;
use super::styles::*;

#[derive(Debug, Clone, Default)]
struct DownloadRowInfo {
    file_url  : Option<String>,
    file_name : Option<String>,
    file_size : Option<u64>,
    file_type : Option<String>,
    download_progress : Vec<PartDownloadInfo>,
    error : Option<String>,
    engine : Arc<RustleDownloader>,
    download_status : DownloadStatus
}

#[derive(Debug)]
pub struct RustleGUI {
    downloads : HashMap<usize, DownloadRowInfo>,
    show_modal : bool,
    modal_url : String,
    modal_is_loading : bool,
    downloads_counter : usize
}


type DownloadInitHeadType = Result<(Option<ResponseHeaderInfo>, RustleDownloader), String>;
type UpdateDownloadType = (Vec<PartDownloadInfo>, DownloadStatus, usize, Arc<RustleDownloader>);

#[derive(Debug, Clone)]
pub enum Message {
    ActionButtonPressed,
    ModalSubmitButtonPressed,
    ModalCancelButtonPressed,
    StartDownloadButtonPressed(usize),
    ResumeDownloadButtonPressed(usize),
    PauseDownloadButtonPressed(usize),
    CancelDownloadButtonPressed(usize),
    ModalTextInputOnInput(String),

    UpdateDownloadCallback(UpdateDownloadType),
    DownloadInitCallback(DownloadInitHeadType),
    StartDownloadCallback(Result<bool, String>),
    PauseDownloadCallback(usize),
    ResumeDownloadCallback(usize)
}

impl RustleGUI {

    pub async fn update_download(engine : Arc<RustleDownloader>, row_id : usize) -> UpdateDownloadType {

        ( 
        engine.get_progress_vec().await, 
        engine.get_status().await, 
        row_id,
        engine.clone()
        )
    }
    pub async fn start_download(engine : Arc<RustleDownloader>) -> Result<bool, String>{
        engine.download(false).await
    }


    pub async fn pause_download(engine : Arc<RustleDownloader>, row_id : usize) -> usize {
        engine.pause().await;
        row_id
    }

    pub async fn resume_download(engine : Arc<RustleDownloader>, row_id : usize) -> usize {
        engine.resume().await;
        row_id
    }

    pub async fn init_download(url : String, dir: String) -> DownloadInitHeadType {
        let download_engine = RustleDownloader::new(4);
        match download_engine {
            Ok(mut engine) => {
                engine.set_url(&url).await?;
                engine.set_out_dir(&dir).await?;

                engine.init().await?;

                let h = engine.get_file_info().await;

                Ok((h, engine))

            },
            Err(e) => {
                Err(e)
            },
        }

    }
}



impl Application for RustleGUI {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();


    fn new(_flags: ()) -> (RustleGUI, Command<Message>) {
        (
            Self { 
                downloads: HashMap::new(),
                show_modal: false,
                modal_url : String::from(""),
                modal_is_loading: false,
                downloads_counter: 0
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("Rustle Downloader")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ActionButtonPressed => {
                self.show_modal = true;
                Command::none()
            },
            Message::ModalCancelButtonPressed => {
                self.show_modal = false;
                Command::none()
            },
            Message::ModalSubmitButtonPressed => {
                self.modal_is_loading = true;
                Command::perform (RustleGUI::init_download(self.modal_url.clone(), String::from("./")), Message::DownloadInitCallback)
            },
            Message::DownloadInitCallback (res) => {
                match res {
                    Ok(pair) => {
                        self.show_modal = false;
                        self.modal_is_loading = false;
                        self.modal_url = String::from("");
                        if let Some(headers) = pair.0 {
                            let e = pair.1;
                            self.downloads.insert(self.downloads_counter,
                                DownloadRowInfo { 
                                    file_url: Some(self.modal_url.clone()), 
                                    file_name: headers.file_name, 
                                    file_size: Some(headers.content_length.unwrap_or(0)), 
                                    file_type: headers.content_type, 
                                    download_progress: Vec::new(), 
                                    error: None,
                                    engine: Arc::new(e),
                                    download_status: DownloadStatus::Idle
                                }
                            );
                            self.downloads_counter+=1;
                        }
                        Command::none()
                    },
                    Err(e) => {
                        self.modal_is_loading = false;
                        println!("{}", e);
                        Command::none()
                    },
                }
            },
            Message::ModalTextInputOnInput(t_str) => {
                self.modal_url = t_str;
                Command::none()
            },
            Message::StartDownloadCallback(_res) => {
                // Download is done here
                Command::none()
            },
            Message::StartDownloadButtonPressed(row_i) => {
                let engine_arc = self.downloads[&row_i].engine.clone();

                // Fire up two commands to start the download / Update the gui progress
                let mut commands : Vec<Command<Message>> = Vec::new();
                commands.push(Command::perform(RustleGUI::start_download(engine_arc.clone())
                                                , Message::StartDownloadCallback));

                commands.push(Command::perform(RustleGUI::update_download(engine_arc, row_i)
                                                , Message::UpdateDownloadCallback));

                Command::batch(commands)

            },
            Message::UpdateDownloadCallback(update_pairs) => {
                let update_progress = update_pairs.0;
                let download_status = update_pairs.1;
                let row_id = update_pairs.2;
                let engine = update_pairs.3;

                match self.downloads.get_mut(&row_id) {
                    Some(row) => {
                        // update gui progress bar
                        row.download_progress = update_progress;
                        // update row download status
                        row.download_status = download_status;

                        match download_status {
                            DownloadStatus::Done => {Command::none()},
                            DownloadStatus::Error=> {/* To Do */ Command::none()},
                            DownloadStatus::Idle => {Command::none()}
                            DownloadStatus::Paused => {Command::none()}
                            DownloadStatus::Downloading => {
                                Command::perform(RustleGUI::update_download(engine,  row_id), Message::UpdateDownloadCallback)
                            }
                        }

                    },
                    None => {Command::none()},
                }
                
            }
            Message::PauseDownloadButtonPressed(row_i) => {
                let engine = self.downloads[&row_i].engine.clone();

                Command::perform(RustleGUI::pause_download(engine, row_i), Message::PauseDownloadCallback)

            },
            Message::ResumeDownloadButtonPressed(row_i) => {
                println!("Resume download pressed");
                let engine = self.downloads[&row_i].engine.clone();

                // Fire up two commands to resume the download / Update the gui progress
                let mut commands : Vec<Command<Message>> = Vec::new();

                commands.push(Command::perform(RustleGUI::resume_download(engine.clone(), row_i), Message::ResumeDownloadCallback));

                commands.push(Command::perform(RustleGUI::update_download(engine, row_i)
                                                , Message::UpdateDownloadCallback));

                Command::batch(commands)
            },
            Message::CancelDownloadButtonPressed(row_i) => {
                self.downloads.remove(&row_i);

                Command::none()
            },
            Message::PauseDownloadCallback(row_i) => {
                match self.downloads.get_mut(&row_i) {
                    Some(row) => {
                        row.download_status = DownloadStatus::Paused;
                        Command::none()
                    },
                    None => {Command::none()},
                }
            },
            Message::ResumeDownloadCallback(row_i) => {
                match self.downloads.get_mut(&row_i) {
                    Some(row) => {
                        row.download_status = DownloadStatus::Downloading;
                        Command::none()
                    },
                    None => {Command::none()},
                }
            }
        }
    }


    fn view(&self) -> Element<Message> {
        /*
            GUI Elements
         */

        let scrollable_content = self.downloads.iter().fold(
            Column::new()
                .width(Length::Fill)
                .height(Length::Shrink)
                .padding(10),
            |scroll, (key, row)| scroll.push( 
                Column::new().push(
                    Row::new()
                    .push(Badge::new(Text::new(row.file_name.clone().unwrap_or(String::from("Unknown")))).style(BadgeStyles::Primary))    
                    .push(Badge::new(Text::new(format_file_size(row.file_size.clone().unwrap_or(0)))).style(BadgeStyles::Secondary))
                    .push(Badge::new(Text::new(row.file_type.clone().unwrap_or(String::from("Unknown")))).style(BadgeStyles::Info))
                    .spacing(10)
                    .padding(10)
                ).push(
                    Row::new()
                    .push(
                        match row.download_status {
                            DownloadStatus::Paused => {
                                ProgressBar::new(0.0..=100.0, 
                                ((row.download_progress.iter().map(|e| e.downloaded_bytes as f32).sum::<f32>()) 
                                / (row.file_size.unwrap_or(1) as f32)) * 100.0
                                ).style(paused_pb_style())
                            },
                            DownloadStatus::Downloading => {
                                ProgressBar::new(0.0..=100.0, 
                                ((row.download_progress.iter().map(|e| e.downloaded_bytes as f32).sum::<f32>()) 
                                / (row.file_size.unwrap_or(1) as f32)) * 100.0
                                ).style(downloading_pb_style())
                            },
                            DownloadStatus::Done => {
                                ProgressBar::new(0.0..=100.0, 
                                ((row.download_progress.iter().map(|e| e.downloaded_bytes as f32).sum::<f32>()) 
                                / (row.file_size.unwrap_or(1) as f32)) * 100.0
                                ).style(done_pb_style())
                            }
                            _ => {
                                ProgressBar::new(0.0..=100.0, 
                                ((row.download_progress.iter().map(|e| e.downloaded_bytes as f32).sum::<f32>()) 
                                / (row.file_size.unwrap_or(1) as f32)) * 100.0
                                )
                            }
                        }

                    
                    )
                    .push(
                        match row.download_status {
                            DownloadStatus::Done => {
                                Badge::new(Text::new("Done")).style(BadgeStyles::Success)
                            },
                            DownloadStatus::Paused => {
                                Badge::new(Text::new("Paused")).style(BadgeStyles::Dark)
                            },
                            DownloadStatus::Error => {
                                Badge::new(Text::new("Error")).style(BadgeStyles::Danger)
                            },
                            _ => {
                                Badge::new(Text::new(
                                format!(
                                    "{:.2} MB/s | {:.2} %",
                                    (row.download_progress.iter().map(|e| e.download_speed as f32).sum::<f32>()) / 1_000_000.0,
                                    ((row.download_progress.iter().map(|e| e.downloaded_bytes as f32).sum::<f32>()) 
                                    / (row.file_size.unwrap_or(1) as f32)) * 100.0
                                ))).style(BadgeStyles::Light) 
                            }
                        }
                ) 
                    .push(
                        match row.download_status {
                            DownloadStatus::Paused => {
                                Button::new(Text::new(Icon::Play.to_string()).font(ICON_FONT)).on_press(Message::ResumeDownloadButtonPressed(*key)).style(play_button_style())
                            },
                            DownloadStatus::Idle => {
                                Button::new(Text::new(Icon::Play.to_string()).font(ICON_FONT)).on_press(Message::StartDownloadButtonPressed(*key)).style(play_button_style())
                            },
                            _ => {
                                Button::new(Text::new(Icon::Play.to_string()).font(ICON_FONT)).style(play_button_style())
                            }
                        }
                    
                    )
                    .push(
                        match row.download_status {
                            DownloadStatus::Downloading => {
                                Button::new(Text::new(Icon::Pause.to_string()).font(ICON_FONT)).on_press(Message::PauseDownloadButtonPressed(*key)).style(pause_button_style())
                            },
                            _ => {
                                Button::new(Text::new(Icon::Pause.to_string()).font(ICON_FONT)).style(pause_button_style())
                            }
                        }
                    
                    )
                    .push(Button::new(Text::new(Icon::X.to_string()).font(ICON_FONT)).on_press(Message::CancelDownloadButtonPressed(*key)).style(cancel_button_style()))
                    .spacing(10)
                    .padding(10)
                )
                .spacing(10)
                .padding(10)
            ));

        let scrollable_content = Scrollable::new(scrollable_content)
                                                                        .height(Length::Fill)
                                                                        .width(Length::Fill);

        let main_column = Column::new()
                            .push(Text::new("Downloads").size(50))
                            .push(
                            Text::new(".......................")
                                .width(Length::Fill)
                            )
                            .push(scrollable_content)
                            .spacing(10)
                            .padding(10);

        let content = FloatingElement::new(
        Container::new(main_column)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(theme::Container::Box),
                    || {
                        Button::new(
                            Text::new(Icon::Plus.to_string())
                            .font(ICON_FONT)
                            .size(45)
                    )
                    .on_press(Message::ActionButtonPressed)
                    .padding(5)
                    .style(circular_floating_button_style()).into()
                    }
                )
                .anchor(Anchor::SouthEast)
                .offset(20.0)
                .hide(false);

        let main_screen_container = Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y();
            
            Modal::new (
                    self.show_modal,
                    main_screen_container,
                    || {
                    match self.modal_is_loading {
                        true => {
                            Card::new(
                                Text::new("Add Url"),
                                Column::new().push(
                                    Spinner::new()
                                    .circle_radius(2.0)
                                    .width(Length::Fill)
                                ).push(
                                    Text::new("Loading ..")
                                )
                                .align_items(Alignment::Center)
                                .spacing(10)
                                .padding(10)
                                .width(450)
                            )
                            .max_width(450.0)
                            .into()
                        },
                        false => {

                            Card::new(
                                Text::new("Add Url"),
                                Column::new()
                                .push(Text::new("Enter the file url to be downloaded"))
                                .push(TextInput::new("Url to be downloaded", &self.modal_url).on_input(Message::ModalTextInputOnInput))
                                .spacing(10)
                                .padding(10)
                            
                            )
                            .foot(
                                Row::new()
                                    .spacing(10)
                                    .padding(5)
                                    .width(Length::Fill)
                                    .push(
                                        Button::new(
                                            Text::new("Cancel").horizontal_alignment(Horizontal::Center),
                                        )
                                        .width(Length::Fill)
                                        .on_press(Message::ModalCancelButtonPressed),
                                    )
                                    .push(
                                        Button::new(
                                            Text::new("Submit").horizontal_alignment(Horizontal::Center),
                                        )
                                        .width(Length::Fill)
                                        .on_press(Message::ModalSubmitButtonPressed),
                                    ),
                            ).max_width(450.0)
                            .into()
                        }
                    }
                }
            ).into()
    }

}