`ui.rs`
```rust
use epub::doc::EpubDoc;
use iced::{
    executor, theme,
    widget::{button, checkbox, column, row, scrollable, text, text_input, Container},
    Application, Command, Element, Length, Settings, Theme,
};
use pulldown_cmark::{html, Options, Parser};
use rfd::AsyncFileDialog;
use std::collections::HashMap;
use std::path::PathBuf;
use zip::result::ZipError;

// EPUB 数据结构 (保持不变)
#[derive(Debug, Clone)]
struct Epub {
    metadata: EpubMetadata,
    spine: Vec<String>,
    manifest: HashMap<String, EpubItem>,
}

#[derive(Debug, Clone)]
struct EpubMetadata {
    title: String,
    author: String,
    language: String,
}

#[derive(Debug, Clone)]
struct EpubItem {
    id: String,
    href: String,
    media_type: String,
    content: String,
}

impl Epub {
    fn from_epub_doc<R: std::io::Read + std::io::Seek>(
        doc: &mut EpubDoc<R>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut epub = Epub::new();

        // Parse metadata
        epub.metadata.title = doc
            .mdata("title")
            .unwrap_or_else(|| String::from("Unknown Title"));
        epub.metadata.author = doc
            .mdata("creator")
            .unwrap_or_else(|| String::from("Unknown Author"));
        epub.metadata.language = doc.mdata("language").unwrap_or_else(|| String::from("en"));

        // Parse spine and manifest
        for i in 0..doc.spine.len() {
            let (href, id) = doc
                .get_resource(&doc.spine[i])
                .ok_or("Failed to get resource")?;
            let (content, _) = doc.get_resource_str(&doc.spine[i]).unwrap_or_default();
            let media_type = doc.get_resource_mime(&doc.spine[i]).unwrap_or_default();

            epub.spine.push(id.clone());
            epub.manifest.insert(
                id.clone(),
                EpubItem {
                    id,
                    href: String::from_utf8(href).unwrap_or_default(),
                    media_type,
                    content,
                },
            );
        }

        Ok(epub)
    }
}

#[derive(Debug, Clone)]
enum EditorMode {
    RichText,
    Markdown,
}

pub struct EpubEditor {
    epub: Epub,
    current_item_id: Option<String>,
    edit_content: String,
    editor_mode: EditorMode,
    markdown_preview: String,
}

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

#[derive(Debug, Clone)]
pub enum MetadataField {
    Title,
    Author,
    Language,
}

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
                Command::perform(
                    async {
                        if let Some(handle) = AsyncFileDialog::new()
                            .add_filter("EPUB", &["epub"])
                            .pick_file()
                            .await
                        {
                            let path = handle.path().to_owned();
                            println!("[INFO] {:?}", path);
                            //  EPUB 解析逻辑
                            parse_epub(path)
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
                        self.update_markdown_preview();
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
            |_| Message::ToggleEditorMode, // Remove the boolean parameter
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
                column![
                    text("Content Editor:"),
                    editor,
                    scrollable(text(&self.markdown_preview).size(14))
                        .height(Length::FillPortion(1))
                ]
                .width(Length::FillPortion(3)),
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

fn parse_epub(path: PathBuf) -> Result<Epub, String> {
    let mut doc = EpubDoc::new(path).map_err(|e| format!("Failed to open EPUB: {}", e))?;

    Epub::from_epub_doc(&mut doc).map_err(|e| format!("Failed to parse EPUB: {}", e))
}
```



第一版可以正确显示的

```rust
```
