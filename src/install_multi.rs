use std::fmt::Write;
use std::sync::{Arc, Mutex};

use crate::install;
use crate::types::{Installation, Package};
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use crate::config::Config;

pub async fn install_multi(
    pending: &Vec<Package>,
    config: &Config,
    silent: bool,
) -> Result<Vec<Installation>, Box<dyn std::error::Error>> {
    struct SharedState {
        pb: Option<ProgressBar>,
        total_size: u64,
        counter: usize,
        downloaded_bytes: u64,
    }

    let shared_state = Arc::new(Mutex::new(SharedState {
        pb: None,
        total_size: 0,
        counter: 0,
        downloaded_bytes: 0,
    }));

    let n = pending.len();

    let mut promises = Vec::new();
    for pkg in pending {
        let shared_state = Arc::clone(&shared_state);
        let promise = install::install(pkg, config, move |event| match event {
            install::InstallEvent::DownloadSize(size) => {
                let mut state = shared_state.lock().unwrap();
                state.total_size += size;
                state.counter += 1;
                if state.counter == n && !silent {
                    let bar = ProgressBar::new(state.total_size);
                    bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                        .expect("failed to construct progress bar")
                        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                        .progress_chars("#>-"));
                    bar.tick();
                    state.pb = Some(bar);
                }
            }
            install::InstallEvent::Progress(chunk) => {
                let mut state = shared_state.lock().unwrap();
                state.downloaded_bytes += chunk;
                if let Some(pb) = &state.pb {
                    pb.set_position(state.downloaded_bytes);
                }
            }
        });
        promises.push(promise);
    }

    let results = join_all(promises).await;
    if let Some(bar) = &shared_state.lock().unwrap().pb {
        bar.finish();
    }

    let mut installations = vec![];

    // Handle results
    for result in results {
        installations.push(result?);
    }

    Ok(installations)
}
