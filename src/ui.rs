use iced::{
    executor,
    widget::{button, column, row, scrollable, text, text_input, Container},
    Application, Command, Element, Length, Settings, Theme,
};
use rfd::AsyncFileDialog;

use crate::epub_handler::EpubHandler;

pub struct EpubEditor {
    epub_handler: EpubHandler,
    current_content: String,
    edit_content: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenEpub,
    EpubLoaded(Result<String, String>),
    EditContent(String),
    SaveContent,
    Test,
}

impl Application for EpubEditor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                epub_handler: EpubHandler::new(),
                current_content: String::new(),
                edit_content: String::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("EPUB Editor")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::OpenEpub => Command::perform(
                async {
                    if let Some(handle) = AsyncFileDialog::new()
                        .add_filter("EPUB", &["epub"])
                        .pick_file()
                        .await
                    {
                        let path = handle.path().to_owned();
                        EpubHandler::open_epub(&path)
                    } else {
                        Err("No file selected".to_string())
                    }
                },
                Message::EpubLoaded,
            ),
            Message::EpubLoaded(result) => {
                match result {
                    Ok(content) => {
                        self.current_content = content.clone();
                        self.edit_content = content;
                    }
                    Err(e) => {
                        println!("Error loading EPUB: {}", e);
                    }
                }
                Command::none()
            }
            Message::EditContent(content) => {
                self.edit_content = content;
                Command::none()
            }
            Message::SaveContent => {
                self.current_content = self.edit_content.clone();
                // TODO: Implement actual saving logic
                Command::none()
            }
            Message::Test => {
                println!("[INFO] test");
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let open_button = button("Open EPUB").on_press(Message::OpenEpub);
        let test = button("[TEST]").on_press(Message::Test);
        let save_button = button("Save Changes").on_press(Message::SaveContent);

        let editor = text_input("Edit EPUB content", &self.edit_content)
            .on_input(Message::EditContent)
            .padding(10)
            .size(20);

        let content = column![
            text("EPUB Editor").size(30),
            row![open_button, save_button, test].spacing(10),
            scrollable(text(&self.current_content).size(16).width(Length::Fill))
                .height(Length::FillPortion(1)),
            editor
        ]
        .spacing(20)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[tokio::main]
async fn main() -> iced::Result {
    EpubEditor::run(Settings::default())
}
