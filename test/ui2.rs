// 引入 iced 库中的各种组件和功能。
use iced::{
    executor, theme,
    widget::{button, checkbox, column, row, scrollable, text, text_input, Container},
    Application, Command, Element, Length, Settings, Theme,
};
// 引入 Markdown 解析库。
use pulldown_cmark::{html, Options, Parser};
// 引入异步文件对话框库。
use rfd::AsyncFileDialog;
// 引入标准库中的集合和路径处理。
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// EPUB 数据结构定义。
#[derive(Debug, Clone)]
struct Epub {
    metadata: EpubMetadata,
    spine: Vec<String>,
    manifest: HashMap<String, EpubItem>,
}
// EPUB 元数据结构定义
#[derive(Debug, Clone)]
struct EpubMetadata {
    title: String,
    author: String,
    language: String,
}

// EPUB 项目项结构定义
#[derive(Debug, Clone)]
struct EpubItem {
    id: String,
    href: String,
    media_type: String,
    content: String,
}

// Epub 结构的实现，提供一个新方法来创建新的 EPUB 实例。
impl Epub {
    fn new() -> Self {
        Self {
            metadata: EpubMetadata {
                title: String::new(),
                author: String::new(),
                language: String::new(),
            },
            spine: Vec::new(),
            manifest: HashMap::new(),
        }
    }
}

// 编辑器模式枚举，用于切换富文本和 Markdown 编辑。
#[derive(Debug, Clone)]
enum EditorMode {
    RichText,
    Markdown,
}

// EPUB 编辑器应用程序的状态结构定义。
pub struct EpubEditor {
    epub: Epub,
    current_item_id: Option<String>,
    edit_content: String,
    editor_mode: EditorMode,
    markdown_preview: String,
}

// EpubEditor 结构的实现，提供更新 Markdown 预览的方法。
#[derive(Debug, Clone)]
pub enum Message {
    OpenEpub,
    EpubLoaded(Result<Epub, String>),
    SelectItem(String),
    EditContent(String),
    SaveContent,
    UpdateMetadata(MetadataField, String),
    ToggleEditorMode, // Remove the boolean parameter
    UpdateMarkdownPreview,
}

// 元数据字段枚举，用于更新 EPUB 元数据。
#[derive(Debug, Clone)]
pub enum MetadataField {
    Title,
    Author,
    Language,
}

// 为 EpubEditor 结构体实现 iced 的 Application trait。
impl Application for EpubEditor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                epub: Epub::new(),
                current_item_id: None,
                edit_content: String::new(),
                editor_mode: EditorMode::RichText,
                markdown_preview: String::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("EPUB Editor - {}", self.epub.metadata.title)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::OpenEpub => {
                println!("[INFO] 打开 EPUB");
                Command::perform(
                    async {
                        if let Some(handle) = AsyncFileDialog::new()
                            .add_filter("EPUB", &["epub"])
                            .pick_file()
                            .await
                        {
                            let path = handle.path().to_owned();

                            // 这里应该实现实际的 EPUB 解析逻辑
                            println!("[INFO] 解析 EPUB");
                            Ok(Epub::new()) // 暂时返回一个空的 Epub 结构
                        } else {
                            Err("No file selected".to_string())
                        }
                    },
                    Message::EpubLoaded,
                )
            }
            Message::EpubLoaded(result) => {
                match result {
                    Ok(epub) => {
                        self.epub = epub;
                        if let Some(first_item) = self.epub.spine.first() {
                            self.current_item_id = Some(first_item.clone());
                            if let Some(item) = self.epub.manifest.get(first_item) {
                                self.edit_content = item.content.clone();
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error loading EPUB: {}", e);
                    }
                }
                Command::none()
            }
            Message::SelectItem(id) => {
                self.current_item_id = Some(id.clone());
                if let Some(item) = self.epub.manifest.get(&id) {
                    self.edit_content = item.content.clone();
                }
                self.update_markdown_preview();
                Command::none()
            }
            Message::EditContent(content) => {
                self.edit_content = content;
                if let EditorMode::Markdown = self.editor_mode {
                    self.update_markdown_preview();
                }
                Command::none()
            }
            Message::SaveContent => {
                if let Some(id) = &self.current_item_id {
                    if let Some(item) = self.epub.manifest.get_mut(id) {
                        item.content = self.edit_content.clone();
                    }
                }
                Command::none()
            }
            Message::UpdateMetadata(field, value) => {
                match field {
                    MetadataField::Title => self.epub.metadata.title = value,
                    MetadataField::Author => self.epub.metadata.author = value,
                    MetadataField::Language => self.epub.metadata.language = value,
                }
                Command::none()
            }
            Message::ToggleEditorMode => {
                self.editor_mode = match self.editor_mode {
                    EditorMode::RichText => EditorMode::Markdown,
                    EditorMode::Markdown => EditorMode::RichText,
                };
                if let EditorMode::Markdown = self.editor_mode {
                    self.update_markdown_preview();
                }
                Command::none()
            }
            Message::UpdateMarkdownPreview => {
                self.update_markdown_preview();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let metadata_input = column![
            row![
                text("Title: "),
                text_input("", &self.epub.metadata.title)
                    .on_input(|s| Message::UpdateMetadata(MetadataField::Title, s))
            ],
            row![
                text("Author: "),
                text_input("", &self.epub.metadata.author)
                    .on_input(|s| Message::UpdateMetadata(MetadataField::Author, s))
            ],
            row![
                text("Language: "),
                text_input("", &self.epub.metadata.language)
                    .on_input(|s| Message::UpdateMetadata(MetadataField::Language, s))
            ],
        ]
        .spacing(10);

        let file_list = self
            .epub
            .spine
            .iter()
            .fold(column![].spacing(5), |column, id| {
                column.push(
                    button(text(id).size(14))
                        .on_press(Message::SelectItem(id.to_string()))
                        .style(if Some(id.to_string()) == self.current_item_id {
                            theme::Button::Primary
                        } else {
                            theme::Button::Secondary
                        }),
                )
            });

        let editor: Element<_> = match self.editor_mode {
            EditorMode::RichText => text_input("Edit content", &self.edit_content)
                .on_input(Message::EditContent)
                .padding(10)
                .size(16)
                .into(),
            EditorMode::Markdown => column![
                text_input("Edit Markdown", &self.edit_content)
                    .on_input(Message::EditContent)
                    .padding(10)
                    .size(16),
                scrollable(text(&self.markdown_preview)).height(Length::FillPortion(1))
            ]
            .spacing(10)
            .into(),
        };

        let editor_toggle = checkbox(
            "Use Markdown Editor",
            matches!(self.editor_mode, EditorMode::Markdown),
            |_| Message::ToggleEditorMode,
        );

        let content = column![
            text("EPUB Editor").size(30),
            button("Open EPUB").on_press(Message::OpenEpub),
            metadata_input,
            editor_toggle,
            row![
                scrollable(column![text("File Structure:"), file_list])
                    .width(Length::FillPortion(1))
                    .height(Length::Fill),
                column![text("Content Editor:"), editor].width(Length::FillPortion(3)),
            ]
            .spacing(20)
            .height(Length::Fill),
            button("Save Changes").on_press(Message::SaveContent),
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

// EpubEditor 结构的实现，提供更新 Markdown 预览的方法。
impl EpubEditor {
    fn update_markdown_preview(&mut self) {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&self.edit_content, options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        self.markdown_preview = html_output;
    }
}
