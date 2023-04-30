#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub enum BackgroundContent {
    Url(url::Url),
    File(std::path::PathBuf),
}

impl ToString for BackgroundContent {
    fn to_string(&self) -> String {
        match self {
            BackgroundContent::Url(url) => url.to_string(),
            BackgroundContent::File(path) => {
                path.as_path().as_os_str().to_str().unwrap().to_string()
            }
        }
    }
}

impl BackgroundContent {
    pub fn is_valid(&self) -> bool {
        match self {
            BackgroundContent::Url(url) => url.domain().is_some(),
            BackgroundContent::File(path) => path.exists(),
        }
    }
}
