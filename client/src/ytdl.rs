#[derive(Default)]
pub enum DownloadState {
    #[default]
    Off,
    Running {
        thread_com: crate::threading::Channel<Message>,
        prcentage: f32,
    },
    Done,
}

#[derive(Default, Debug, Clone)]
pub struct DownloadConfig {
    pub url: String,
    pub file_name: String, // dl_opts: String,
}

#[derive(PartialEq, Debug)]
pub enum Message {
    PrcentageUpdate(f32),
    // Error(String)
}

fn start_download(cfg: &DownloadConfig) -> Result<DownloadState, String> {
    let url = cfg.url.to_string();
    let file_name = cfg.file_name.to_string();

    let (channel1, channel2) = crate::threading::Channel::new_pair();

    let video_options = rusty_ytdl::VideoOptions {
        quality: rusty_ytdl::VideoQuality::Highest,
        filter: rusty_ytdl::VideoSearchOptions::Video,
        download_options: rusty_ytdl::DownloadOptions {
            dl_chunk_size: Some(1024 * 1024), // 1MB / packet
        },
        ..Default::default()
    };

    let video = if let Ok(video) =
        rusty_ytdl::Video::new_with_options(url.clone(), video_options.clone())
    {
        video
    } else {
        error!("Could not create video with given url: {url}");
        // panic for now, fix later
        // panic!("")
        return Err(format!("Could not create video with given url: {url}"));
    };

    let video_id = video.get_video_id();

    std::thread::Builder::new()
        .name(format!("Downloader thread {}", video_id))
        .spawn(move || {
            // Spawn the root task
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                use std::io::Write as _;
                debug!("Fetching stream");
                let download_stream = video.stream().await.unwrap();

                // Make sure that the target path exists
                let mut target_directory = { std::path::PathBuf::from("downloads/") };

                if !target_directory.exists() {
                    warn!("Given dir does not exists.. creating it.. {target_directory:?}");
                    std::fs::create_dir_all(target_directory.clone())
                        .map_err(|reason| {
                            format!(
                                "Could not create directory '{target_directory:?}', reason: {reason}"
                            )
                        })
                        .unwrap();
                }
                if !target_directory.is_absolute() {
                    let unabsolute_path = target_directory.clone();
                    target_directory = target_directory
                        .canonicalize()
                        .map_err(|reason| {
                            format!("Could not canonicalize '{target_directory:?}', reason: {reason}")
                        })
                        .unwrap();
                    
                    if target_directory
                        .display()
                        .to_string()
                        .starts_with("\\\\?\\")
                    {
                        target_directory = std::path::PathBuf::from(
                            target_directory
                                .display()
                                .to_string()
                                .replace("\\\\?\\", ""),
                        )
                    }

                    warn!("Path is not absolute.. {unabsolute_path:?} -> {target_directory:?}");
                }

                // Create target file
                let mut file = std::fs::File::create(target_directory.join(file_name.clone()))
                    .map_err(|reason| {
                        format!(
                            "Could not create file '{path:?}', reason: {reason}",
                            path = target_directory.join(file_name.clone())
                        )
                    })
                    .unwrap();

                // This is used to calculate the % of the download
                let content_length = {
                    let info = video.get_info().await.unwrap();
                    let format = rusty_ytdl::choose_format(&info.formats, &video_options)
                        .map_err(|_op| rusty_ytdl::VideoError::VideoSourceNotFound)
                        .unwrap();

                    format
                        .content_length
                        .unwrap_or("0".to_string())
                        .parse::<u64>()
                        .unwrap_or(0)
                };

                let mut downloaded = 0;

                debug!("Got stream, Starting loop..");
                while let Ok(chunk) = download_stream.chunk().await {
                    if let Some(chunk_bytes) = chunk {
                        debug!("received {} bytes", chunk_bytes.len());
                        // write to file
                        debug!("Writing to file..");
                        file.write_all(&chunk_bytes)
                            .map_err(|reason| {
                                format!(
                                    "Could not write downloaded chunk to '{path:?}', reason: {reason}",
                                    path = target_directory.join(file_name.clone())
                                )
                            })
                            .unwrap();

                        downloaded += chunk_bytes.len();
                        // send update to ui
                        let prcentage = (downloaded as f32 / content_length as f32)*100.;
                        debug!("Sending {prcentage}%");
                        if let Err(e) = channel2.send(Message::PrcentageUpdate(prcentage)){
                            error!("{:?}", e);
                            panic!("");
                        };
                    }else{
                        warn!("Received empty chunk, probably the end of the download, quitting");
                        break;
                    }
                }
            });
            debug!("The download finished");
        })
        .unwrap();

    Ok(DownloadState::Running {
        thread_com: channel1,
        prcentage: 0.,
    })
}

impl DownloadState {
    pub fn update(&mut self) {
        if let DownloadState::Running {
            thread_com,
            prcentage,
        } = self
        {
            while let Ok(msg) = thread_com.try_recv() {
                match msg {
                    Message::PrcentageUpdate(value) => *prcentage = value,
                }
            }
        }
    }
    pub fn start_download(&mut self, cfg: &DownloadConfig) -> Result<(), String> {
        let new_state = start_download(cfg)?;
        *self = new_state;

        Ok(())
    }

    pub fn get_value(&self) -> Option<f32> {
        if let DownloadState::Running {
            thread_com: _,
            prcentage,
        } = self
        {
            Some(*prcentage)
        } else {
            None
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, DownloadState::Running { .. })
    }
}
