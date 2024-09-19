use epub::doc::EpubDoc;
use std::path::Path;

pub struct EpubHandler;

impl EpubHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn open_epub<P: AsRef<Path>>(path: P) -> Result<String, String> {
        println!("[INFO] 打开 epub");
        match EpubDoc::new(path) {
            Ok(doc) => {
                let content = doc
                    .metadata
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.join(", ")))
                    .collect::<Vec<String>>()
                    .join("\n");
                Ok(content)
            }
            Err(e) => Err(format!("Error opening EPUB: {}", e)),
        }
    }
}
