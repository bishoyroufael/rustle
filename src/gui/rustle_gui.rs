/*
    Imports
*/
use std::collections::HashMap;
use std::sync::Arc;
use crate::download_utils::downloader::{RustleDownloader, ResponseHeaderInfo, PartDownloadInfo, DownloadStatus};
use iced::Color;
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
use super::components::*;


/*
    Struct defining a row content in the GUI downloads list 
*/
#[derive(Debug, Clone, Default)]
struct DownloadRowInfo {
    /// file url to be downloaded
    file_url  : Option<String>,
    /// detected file name
    file_name : Option<String>,
    /// detected file size in bytes 
    file_size : Option<u64>,
    /// detected file type
    file_type : Option<String>,
    /// vector storing the downloading progress
    download_progress : Vec<PartDownloadInfo>,
    /// error message if present
    error : Option<String>,
    /// engine for downloading the file
    engine : Arc<RustleDownloader>,
    /// downloading status
    download_status : DownloadStatus
}

impl DownloadRowInfo {
    pub fn get_total_download_progress(self: &DownloadRowInfo) -> f32 {
        (self.download_progress.iter().map(|e| e.downloaded_bytes as f32).sum::<f32>()) 
                                / (self.file_size.unwrap_or(1) as f32) * 100.0
    }
    pub fn get_download_speed_mbs(self: &DownloadRowInfo) -> f32 {
        (self.download_progress.iter().map(|e| e.download_speed as f32).sum::<f32>()) / 1_000_000.0
    }
}

/*
    Struct defining the GUI
*/
#[derive(Debug)]
pub struct RustleGUI {
    /// hasmap storing the downloads in the scrollable list
    downloads : HashMap<usize, DownloadRowInfo>,
    /// flag to show modal
    show_modal : bool,
    /// modal url string field
    modal_url : String,
    /// modal url string field
    modal_is_loading : bool,
    /// counter that acts as the key for the hashmap 
    downloads_counter : usize
}


// Callback types
type DownloadInitHeadType = Result<(Option<ResponseHeaderInfo>, RustleDownloader), String>;
type UpdateDownloadType = (Vec<PartDownloadInfo>, DownloadStatus, usize, Arc<RustleDownloader>);


/*
    GUI messages
*/
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

    /// Updates the download progress and status for a specific row.
    ///
    /// # Arguments
    ///
    /// * `engine` - A shared Arc reference to the `RustleDownloader` instance.
    /// * `row_id` - The identifier of the row for which to update the download information.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// * A vector of download progress.
    /// * The current download status.
    /// * The provided `row_id`.
    /// * A cloned `RustleDownloader` instance.
    pub async fn update_download(engine : Arc<RustleDownloader>, row_id : usize) -> UpdateDownloadType {

        ( 
        engine.get_progress_vec().await, 
        engine.get_status().await, 
        row_id,
        engine.clone()
        )
    }

    /// Starts the download using the provided `RustleDownloader` instance.
    ///
    /// # Arguments
    ///
    /// * `engine` - A shared Arc reference to the `RustleDownloader` instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` indicating whether the download was successfully started (`Ok(true)`)
    /// or an error message (`Err(String)`).
    pub async fn start_download(engine : Arc<RustleDownloader>) -> Result<bool, String>{
        engine.download(false).await
    }

    /// Pauses the download using the provided `RustleDownloader` instance and returns the row ID.
    ///
    /// # Arguments
    ///
    /// * `engine` - A shared Arc reference to the `RustleDownloader` instance.
    /// * `row_id` - The identifier of the row to pause.
    ///
    /// # Returns
    ///
    /// Returns the provided `row_id`.
    pub async fn pause_download(engine : Arc<RustleDownloader>, row_id : usize) -> usize {
        engine.pause().await;
        row_id
    }

    /// Resumes the download using the provided `RustleDownloader` instance and returns the row ID.
    ///
    /// # Arguments
    ///
    /// * `engine` - A shared Arc reference to the `RustleDownloader` instance.
    /// * `row_id` - The identifier of the row to resume.
    ///
    /// # Returns
    ///
    /// Returns the provided `row_id`.
    pub async fn resume_download(engine : Arc<RustleDownloader>, row_id : usize) -> usize {
        engine.resume().await;
        row_id
    }

    /// Initializes a download using the provided URL and directory, returning initialization info.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download the file from.
    /// * `dir` - The directory to save the downloaded file.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the initialization info as a tuple:
    /// * A `DownloadInitHeadType` containing file information.
    /// * A newly created `RustleDownloader` instance.
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

    /// Creates a new `RustleGUI` instance along with an initial `Command`.
    ///
    /// # Arguments
    ///
    /// * `_flags` - A placeholder argument that is not used in this method.
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// * A newly constructed `RustleGUI` instance.
    /// * An initial `Command` representing no action.
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

    /// Returns the title of the GUI.
    ///
    /// # Returns
    ///
    /// Returns the title as a `String`.
    fn title(&self) -> String {
        String::from("Rustle Downloader")
    }

    /// Updates the GUI state based on the received message and returns a `Command`.
    ///
    /// # Arguments
    ///
    /// * `message` - A message representing a user action or event.
    ///
    /// # Returns
    ///
    /// Returns a `Command` representing an action to be executed.
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
                // Download callback after it's done
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
                // println!("Resume download pressed");
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

    /// Generates the GUI view based on the current state of `RustleGUI`.
    ///
    /// # Returns
    ///
    /// Returns an `Element` representing the GUI's user interface.
    fn view(&self) -> Element<Message> {
        /*
            GUI Elements
         */

        // Scrollable content list
        let scrollable_content = self.downloads.iter().fold(
            Column::new()
                .width(Length::Fill)
                .height(Length::Shrink)
                .padding(10),
            |scroll, (key, row)| scroll.push( 
                // Column containing 2 rows
                // 1st row contains badges for file info
                // 2nd row contains progress bar and respective action buttons
                Column::new().push(
                    // 1st row
                    Row::new()
                    .push(badge(row.file_name.clone().unwrap_or(String::from("Unknown")), BadgeStyles::Primary))    
                    .push(badge(format_file_size(row.file_size.clone().unwrap_or(0)), BadgeStyles::Secondary))
                    .push(badge(row.file_type.clone().unwrap_or(String::from("Unknown")), BadgeStyles::Info))
                    .spacing(10)
                    .padding(10)
                ).push(
                    // 2nd row
                    Row::new()
                    .push( // progress bar
                        match row.download_status {
                            DownloadStatus::Paused => {
                                progress_bar(row.get_total_download_progress(), paused_pb_style())
                            },
                            DownloadStatus::Downloading => {
                                progress_bar(row.get_total_download_progress(), downloading_pb_style())
                            },
                            DownloadStatus::Done => {
                                progress_bar(row.get_total_download_progress(),done_pb_style())
                            }
                            _ => {
                                progress_bar(row.get_total_download_progress(), theme::ProgressBar::Danger)
                            }
                        }
                    )
                    .push( // badge progress status
                        match row.download_status {
                            DownloadStatus::Done => {
                                badge(String::from("Done"), BadgeStyles::Success)
                            },
                            DownloadStatus::Paused => {
                                badge(String::from("Paused"), BadgeStyles::Dark)
                            },
                            DownloadStatus::Error => {
                                badge(String::from("Error"), BadgeStyles::Danger)
                            },
                            // Downloading Badge 
                            _ => {
                                badge (
                                format!("{:.2} MB/s | {:.2} %",
                                    row.get_download_speed_mbs(),
                                    row.get_total_download_progress()
                                ), BadgeStyles::Light)
                            }
                        }
                ) 
                    .push( // play button
                        match row.download_status {
                            DownloadStatus::Paused => {
                                button(play_icon(), Some(Message::ResumeDownloadButtonPressed(*key)), play_submit_button_style())
                            },
                            DownloadStatus::Idle => {
                                button(play_icon(), Some(Message::StartDownloadButtonPressed(*key)), play_submit_button_style())
                            },
                            _ => {
                                button(play_icon(), None, play_submit_button_style())
                            }
                        }
                    
                    )
                    .push( // pause button
                        match row.download_status {
                            DownloadStatus::Downloading => {
                                button( pause_icon(), Some(Message::PauseDownloadButtonPressed(*key)), pause_button_style())
                            },
                            _ => {
                                button(pause_icon(), None, pause_button_style())
                            }
                        }
                    
                    )
                    .push( // cancel button
                        button(cancel_icon(), Some(Message::CancelDownloadButtonPressed(*key)), cancel_button_style())
                    )
                    .spacing(10)
                    .padding(10)
                )
                .spacing(10)
                .padding(10)
            ));

        let scrollable_content = Scrollable::new(scrollable_content)
                                                                        .height(Length::Fill)
                                                                        .width(Length::Fill);

        let initial_info_container = Container::new(
            Row::new()
            .push(
                info_icon().size(22).style(grey_color_text_style())
            ).push(
                Text::new("Add downloads using the floating button").size(22).style(grey_color_text_style())
            ).spacing(3)
        ).width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y();

        
        // Main column content
        // If no downloads are added, show info container
        // otherwise scrollable content
        let main_column = Column::new()
                            .push(
                            Row::new().push(
                                Text::new("Downloads").size(50).style(theme::Text::Color(GREEN_COLOR_MAIN)) 
                            ).push(
                                Text::new(Icon::FileEarmarkArrowDown.to_string()).font(ICON_FONT).size(50).style(theme::Text::Color(GREEN_COLOR_MAIN))
                            ).spacing(15)
                            
                            )
                            .push(
                            Text::new("----------------------------------------------------------------").style(theme::Text::Color(GREEN_COLOR_MAIN))
                                .width(Length::Fill)
                            )
                            .push(
                                match self.downloads.is_empty() {
                                    true => {
                                        initial_info_container
                                    },
                                    false=> {
                                        Container::new(scrollable_content) 
                                    }
                                }
                            
                            )
                            .spacing(10)
                            .padding(10);

        // Floating button for adding a download url
        let content = FloatingElement::new(
                // Main contain under the floating button
        Container::new(main_column)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(white_container_style()),
                    || {
                        // Floating Button
                        button(plus_icon().size(45),
                            Some(Message::ActionButtonPressed),
                                 circular_floating_button_style()
                            ).padding(5).into()
                    }
                )
                .anchor(Anchor::SouthEast)
                .offset(20.0)
                .hide(false);

        // Full container with all elements
        let main_screen_container = Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y();
        
        // Modal that is set to show dynamically
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
                                        .style(cancel_button_style())
                                        .width(Length::Fill)
                                        .on_press(Message::ModalCancelButtonPressed),
                                    )
                                    .push(
                                        Button::new(
                                            Text::new("Submit").horizontal_alignment(Horizontal::Center),
                                        )
                                        .style(play_submit_button_style())
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