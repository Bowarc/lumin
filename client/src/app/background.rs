#[derive(Default, Debug, Clone)]
pub struct Background {
    pub monitor_index: usize,
    pub content: String,
    pub state: crate::app::state::State<crate::app::state::BackgroundState>,
}

impl Background {
    pub fn build_content(&self) -> Option<shared::background::BackgroundContent> {
        let content_pathbuf = std::path::PathBuf::from(self.content.clone());
        if content_pathbuf.exists() {
            // It was in fact a path
            Some(shared::background::BackgroundContent::File(content_pathbuf))
        } else if let Ok(content_url) = url::Url::parse(&self.content) {
            // It was in fact an url
            Some(shared::background::BackgroundContent::Url(content_url))
        } else {
            // Fuck ya
            None
        }
    }
}
