use std::error::Error;
use std::fmt::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::install::{install, InstallEvent};
use crate::types::{Installation, Package};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use crate::config::Config;

pub async fn install_multi(
    pending: &[Package],
    config: &Config,
    pb: Option<ProgressBar>,
) -> Result<Vec<Installation>, Box<dyn Error>> {
    let pb = pb.map(|pb| {
        configure_bar(&pb);
        Arc::new(Mutex::new(pb))
    });

    pending
        .iter()
        .map(|pkg| {
            install(
                pkg,
                config,
                pb.clone().map(|pb| {
                    move |event| match event {
                        InstallEvent::DownloadSize(size) => {
                            pb.lock().unwrap().inc_length(size);
                        }
                        InstallEvent::Progress(chunk) => {
                            pb.lock().unwrap().inc(chunk);
                        }
                    }
                }),
            )
        })
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect()
}

fn configure_bar(pb: &ProgressBar) {
    pb.set_length(1);
    pb.set_style(
        ProgressStyle::with_template(
            "{elapsed:.dim} ❲{wide_bar:.red}❳ {percent}% {bytes_per_sec:.dim} {bytes:.dim}",
        )
        .unwrap()
        .with_key("elapsed", |state: &ProgressState, w: &mut dyn Write| {
            let s = state.elapsed().as_secs_f64();
            let precision = precision(s);
            write!(w, "{:.precision$}s", s, precision = precision).unwrap()
        })
        .with_key("bytes", |state: &ProgressState, w: &mut dyn Write| {
            let (right, divisor) = pretty_size(state.len().unwrap());
            let left = state.pos() as f64 / divisor as f64;
            let leftprecision = precision(left);
            write!(
                w,
                "{:.precision$}/{}",
                left,
                right,
                precision = leftprecision
            )
            .unwrap()
        })
        .progress_chars("⌬ "),
    );
    pb.enable_steady_tick(Duration::from_millis(50));
}

fn pretty_size(n: u64) -> (String, u64) {
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
    let mut size = n as f64;
    let mut i = 0;
    let mut divisor = 1;

    while size > 1024.0 && i < units.len() - 1 {
        size /= 1024.0;
        i += 1;
        divisor *= 1024;
    }

    let formatted = format!(
        "{:.precision$} {}",
        size,
        units[i],
        precision = precision(size)
    );

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
