use iced::{
    executor,
    widget::{button, column, text, Container},
    Application, Command, Element, Length, Settings, Theme,
};

pub struct EpubEditor {
    counter: i32,
}

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
}

impl Application for EpubEditor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self { counter: 0 }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Minimal EPUB Editor")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Increment => {
                self.counter += 1;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let content = column![
            text("Minimal EPUB Editor").size(30),
            text(format!("Counter: {}", self.counter)).size(20),
            button("Increment").on_press(Message::Increment),
        ]
        .spacing(20)
        .padding(20);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

pub fn main() -> iced::Result {
    EpubEditor::run(Settings::default())
}
