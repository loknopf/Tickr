use anyhow::Result;
use self_update::cargo_crate_version;

const REPO_OWNER: &str = "loknopf";
const REPO_NAME: &str = "Tickr";

/// Check if a newer version is available on GitHub releases
pub fn check_for_updates() -> Result<Option<String>> {
    let current_version = cargo_crate_version!();
    
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .build()?
        .fetch()?;
    
    if let Some(latest_release) = releases.first() {
        let latest_version = latest_release.version.trim_start_matches('v');
        
        if latest_version != current_version {
            return Ok(Some(latest_version.to_string()));
        }
    }
    
    Ok(None)
}

/// Perform the self-update by downloading and replacing the current binary
pub fn perform_update() -> Result<()> {
    let current_version = cargo_crate_version!();
    
    let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name("tickr")
        .show_download_progress(true)
        .current_version(current_version)
        .build()?
        .update()?;
    
    if status.updated() {
        println!("Updated to version: {}", status.version());
    } else {
        println!("Already up to date");
    }
    
    Ok(())
}
