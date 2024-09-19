mod epub_handler;
mod ui;

use iced::{Application, Settings};
use ui::EpubEditor;

fn main() -> iced::Result {
    EpubEditor::run(Settings::default())
}
