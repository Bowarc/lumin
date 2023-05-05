use std::io::Write as _;

mod models;
const REPO_URL: &str = "github.com/Bowarc/Lumin";
const GH_API_RELEASE_URL: &str = "https://api.github.com/repos/bowarc/lumin/releases";
/*          NOTES
    While testing i found that for the url GH_API_RELEASE_URL, releases where by chronological order
    (indx 0 is the latest)

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
        .user_agent("Lumin installer")
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

    let latest_release = releases.get(1).unwrap();
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
}

fn main() {
    download_latest_release(std::path::PathBuf::from("downloads/"))
}
