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
                    #[allow(clippy::literal_string_with_formatting_args)]
                    bar.set_style(ProgressStyle::with_template("{elapsed:.dim} ❲{wide_bar:.red}❳ {percent}% {bytes_per_sec:.dim} {bytes:.dim}")
                        .expect("failed to construct progress bar")
                        .with_key("elapsed", |state: &ProgressState, w: &mut dyn Write| {
                            let s = state.elapsed().as_secs_f64();
                            let precision = precision(s);
                            write!(w, "{:.precision$}s", s, precision = precision).unwrap()
                        })
                        .with_key("bytes", |state: &ProgressState, w: &mut dyn Write| {
                            let (right, divisor) = pretty_size(state.len().unwrap(), None);
                            let left = state.pos() as f64 / divisor as f64;
                            let leftprecision = precision(left);
                            write!(w, "{:.precision$}/{}", left, right, precision = leftprecision).unwrap()
                        })
                        .progress_chars("⌬ "));
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

    let mut installations = vec![];
    for result in join_all(promises).await {
        installations.push(result?);
    }

    if let Some(bar) = &shared_state.lock().unwrap().pb {
        bar.finish_and_clear();
    }

    Ok(installations)
}

fn pretty_size(n: u64, fixed: Option<usize>) -> (String, u64) {
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
    let mut size = n as f64;
    let mut i = 0;
    let mut divisor = 1;

    while size > 1024.0 && i < units.len() - 1 {
        size /= 1024.0;
        i += 1;
        divisor *= 1024;
    }

    let precision = fixed.unwrap_or_else(|| precision(size));

    let formatted = format!("{:.precision$} {}", size, units[i], precision = precision);
    (formatted, divisor)
}

fn precision(n: f64) -> usize {
    if n < 10.0 {
        2
    } else if n < 100.0 {
        1
    } else {
        0
    }
}
