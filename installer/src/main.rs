
use std::io::Write as _;

mod models;
const REPO_URL: &str = "github.com/Bowarc/Lumin";
const GH_API_RELEASE_URL: &str = "https://api.github.com/repos/bowarc/lumin/releases";
/*          NOTES
    While testing i found that for the url GH_API_RELEASE_URL, releases where by chronological order
    (indx 0 is the latest)


    The following is the console output of when you run the program with no internet access:

    Given dir does not exists.. creating it.. "downloads/"
    Path is not absolute.. "downloads/" -> "D:\\Dev\\Rust\\projects\\lumin\\installer\\downloads"
    Installing components to "D:\\Dev\\Rust\\projects\\lumin\\installer\\downloads"
    thread 'main' panicked at 'Could not get the release data: reqwest::Error { kind: Request, url: Url { scheme: "https", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("api.github.com")), port: None, path: "/repos/bowarc/lumin/releases", query: None, fragment: None }, source: hyper::Error(Connect, ConnectError("dns error", Os { code: 11001, kind: Uncategorized, message: "No such host is known." })) }', installer\src\main.rs:55:10
    stack backtrace:
       0: std::panicking::begin_panic_handler
                 at /rustc/22f247c6f3ed388cb702d01c2ff27da658a8b353/library\std\src\panicking.rs:579
       1: core::panicking::panic_fmt
                 at /rustc/22f247c6f3ed388cb702d01c2ff27da658a8b353/library\core\src\panicking.rs:64
       2: core::result::unwrap_failed
                 at /rustc/22f247c6f3ed388cb702d01c2ff27da658a8b353/library\core\src\result.rs:1750
       3: enum2$<core::result::Result<reqwest::blocking::response::Response,reqwest::error::Error> >::expect<reqwest::blocking::response::Response,reqwest::error::Error>
                 at /rustc/22f247c6f3ed388cb702d01c2ff27da658a8b353\library\core\src\result.rs:1047
       4: installer::download_latest_release
                 at .\src\main.rs:52
       5: installer::main
                 at .\src\main.rs:107
       6: core::ops::function::FnOnce::call_once<void (*)(),tuple$<> >
                 at /rustc/22f247c6f3ed388cb702d01c2ff27da658a8b353\library\core\src\ops\function.rs:250
    note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.
    error: process didn't exit successfully: `D:\Dev\Rust\projects\lumin\target\debug\installer.exe` (exit code: 101)


*/

fn test() {
    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent("Lumin installer")
        .build()
        .unwrap();

    let resp = client
        .get(GH_API_RELEASE_URL)
        .send()
        .expect("request failed");

    if resp.status() != 200 {
        println!("Status error: {}", resp.status());
    }

    println!("{:#?}", resp.json::<Vec<models::gh_releases::Release>>());
}

fn download_latest_release(mut target_path: std::path::PathBuf) {
    if !target_path.exists() {
        println!("Given dir does not exists.. creating it.. {target_path:?}");
        std::fs::create_dir_all(target_path.clone()).unwrap();
    }
    if !target_path.is_absolute() {
        let old = target_path.clone();
        target_path = target_path.canonicalize().unwrap();
        if target_path.display().to_string().starts_with("\\\\?\\") {
            target_path =
                std::path::PathBuf::from(target_path.display().to_string().replace("\\\\?\\", ""))
        }

        println!("Path is not absolute.. {old:?} -> {target_path:?}");
    }
    println!("Installing components to {target_path:?}");

    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent("Bowarc's Lumin installer")
        .build()
        .unwrap();

    let releases_resp = client
        .get(GH_API_RELEASE_URL)
        .send()
        .expect("Could not get the release data");

    if releases_resp.status() != 200 {
        eprintln!(
            "could not get the releases data (status: {})",
            releases_resp.status()
        );
        return;
    }

    let releases = releases_resp
        .json::<Vec<models::gh_releases::Release>>()
        .unwrap();

    println!("Listing releases");
    for (idx, release) in releases.iter().enumerate() {
        println!(
            "Release {idx}\nName: {}\nTag: {}\nDescription: {}\nDate: {}\n\n",
            release.name, release.tag_name, release.body, release.published_at
        )
    }

    let latest_release = releases.get(0).unwrap();
    println!("Downloading assets ...");
    for asset in latest_release.assets.iter() {
        let start_time = std::time::Instant::now();
        print!("Downloading {} ..\r", asset.name);
        std::io::stdout().flush().unwrap();

        let resp = client
            .get(asset.browser_download_url.clone())
            .send()
            .expect("request failed");

        let body = resp.bytes().expect("body invalid");
        // THERE IS A BUG SOMEWHERE ARROUND HERE WHERE BIG FILES TIME OUT

        std::fs::write(target_path.join(asset.name.clone()), &body).unwrap();
        println!(
            "Downloading {} .. Done ({} bytes in {:.2}s)",
            asset.name,
            body.len(),
            start_time.elapsed().as_secs_f32()
        );
    }

    println!("Done");

    // ask to create a lnk on desktop
    // ask to create an entry in the start menu dir

    // https://docs.rs/dialoguer/latest/dialoguer/ ?
    // https://docs.rs/indicatif/latest/indicatif/ ?
    // https://docs.rs/console/latest/console/ ?
    // https://docs.rs/rfd/latest/rfd/ ?
}

#[tokio::main]
async fn main() {
    // https://github.com/Bowarc/Lumin/releases/download/0.1.3/lumin_mpv.exe

    // let client = reqwest::blocking::ClientBuilder::new()
    //     .user_agent("Bowarc's Lumin installer")
    //     .build()
    //     .unwrap();

    // let resp = client
    //     .get("https://github.com/Bowarc/Lumin/releases/download/0.1.3/lumin_mpv.exe")
    //     .send()
    //     .expect("Could not get the release data");

    // if resp.status() != 200 {
    //     eprintln!(
    //         "could not get the releases data (status: {})",
    //         resp.status()
    //     );
    //     return;
    // }

    // let body = resp.bytes().expect("body invalid");

    // std::fs::write(".\\mpv.exe", &body).unwrap();
    // println!("Ok");

    // download_latest_release(std::path::PathBuf::from("downloads/"))
    if let Err(e) = download_latest_release_test1(std::path::PathBuf::from("downloads/")).await{
        println!("Lumin installer ran into an error:\n{e}\nStopping installation");
    }
    
}

async fn download_latest_release_test1(mut download_path: std::path::PathBuf)-> Result<(), String> {
    use futures_util::stream::StreamExt;
    if !download_path.exists() {
        println!("Given dir does not exists.. creating it.. {download_path:?}");
        std::fs::create_dir_all(download_path.clone()).map_err(|reason| format!("Could not create directory '{download_path:?}', reason: {reason}"))?;
    }
    if !download_path.is_absolute() {
        let unabsolute_path = download_path.clone();
        download_path = download_path.canonicalize().map_err(|reason| format!("Could not canonicalize '{download_path:?}', reason: {reason}"))?;
        if download_path.display().to_string().starts_with("\\\\?\\") {
            download_path =
                std::path::PathBuf::from(download_path.display().to_string().replace("\\\\?\\", ""))
        }

        println!("Path is not absolute.. {unabsolute_path:?} -> {download_path:?}\n");
    }

    let reqwest_client = reqwest::ClientBuilder::new()
        .user_agent("Bowarc's Lumin installer")
        .build().map_err(|reason| format!("Could not create reqwest client, reason: {reason}"))?;


    print!("Fetching releases list..\r");
    std::io::stdout().flush().map_err(|reason| format!("Could not flush stdout, reason: {reason}"))?;
    let releases_list_resp = reqwest_client
        .get(GH_API_RELEASE_URL)
        .send()
        .await.map_err(|reason| format!("Could not fetch releases data, reason: {reason}"))?;

    if releases_list_resp.status() != 200 {
        // eprintln!(
        //     "Release list request returned a non-ok status (status: {})",
        //     releases_list_resp.status()
        // );
        // return Err(format!("Release list request returned a non-ok status (status: {})", releases_list_resp.status()));
        Err(format!("Release list request returned a non-ok status (status: {})", releases_list_resp.status()))?
    }
    std::thread::sleep(std::time::Duration::from_secs_f32(0.1));

    println!("Fetching releases list.. Ok");

    println!("Converting release list response into usable data..");
    let release_list = releases_list_resp
        .json::<Vec<models::gh_releases::Release>>()
        .await.map_err(|reason| format!("Could not convert release list response into usable data, reason: {reason}"))?;

    // println!("Releases: ");
    // for (idx, release) in release_list.iter().enumerate() {
    //     println!(
    //         "Release {idx}\nName: {}\nTag: {}\nDate: {}\n",
    //         release.name, release.tag_name, release.published_at
    //     )
    // }

    let selected_release = release_list
        .get(0)
        .ok_or(
            if release_list.is_empty(){
                "Could not get the latest release data, reason: The list is empty".to_string()
            }else{
                "Could not get the latest release data, reason: Unknown".to_string()
            }
        )?;
    println!(
        "Selected latest release with tag: <{}> and title: <{}>\n",
        selected_release.tag_name, selected_release.name
    );

    println!("Installing components to {download_path:?}");
    for asset in selected_release.assets.iter() {
        let asset_resp = reqwest_client
            .get(asset.browser_download_url.clone())
            .send()
            .await
            .map_err(|reason| 
                format!("Could not fetch url '{url}', reason: {reason}", url = asset.browser_download_url)
            )?;

        let total_size = asset_resp
            .content_length()
            .ok_or(format!(
                "Could not get content length from '{}'",
                asset.browser_download_url
            ))?;

        // Indicatif setup
        let pb = indicatif::ProgressBar::new(total_size);
        pb.set_style(indicatif::ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").expect("Could not create the progress bar")
            .progress_chars("#>-"));
        pb.set_message(format!("Downloading {}", asset.name));

        // download chunks
        let mut file = std::fs::File::create(download_path.join(asset.name.clone()))
            .map_err(|reason |format!(
                "Could not create file '{path:?}', reason: {reason}",
                path = download_path.join(asset.name.clone())
            ))?;
        let mut downloaded: u64 = 0;
        let mut stream = asset_resp.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(
                |reason |
                format!("Could not get the next chunk while downloading '{url}', reason {reason}", 
                    url = asset.browser_download_url
                )
            )?;
            file.write_all(&chunk)
                .map_err(|reason| format!("Could not write downloaded chunk to '{path:?}', reason: {reason}", path=download_path.join(asset.name.clone()) ))?;
            let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message(format!(
            "Downloaded {}\nTo {:?}",
            asset.browser_download_url,
            download_path.join(asset.name.clone())
        ));
    }

    println!("\n\nSuccessfully installed Lumin to {:?}", download_path);
    Ok(())
}
