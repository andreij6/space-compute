use std::cell::Cell;
use std::time::Duration;

use crate::journal;
use crate::rate;

const RESUME_INTERVAL: Duration = Duration::from_secs(300);
const RATE_REFRESH_INTERVAL: Duration = Duration::from_secs(3_600);
const RESUME_SWEEP_LIMIT: usize = 50;

thread_local! {
    static WATERMARK: Cell<u64> = const { Cell::new(0) };
}

pub fn start() {
    ic_cdk_timers::set_timer_interval(RESUME_INTERVAL, resume_sweep);
    ic_cdk_timers::set_timer_interval(RATE_REFRESH_INTERVAL, rate_refresh_sweep);
    ic_cdk_timers::set_timer(Duration::ZERO, rate_refresh_sweep());
}

async fn resume_sweep() {
    let (ops, watermark) = journal::resumable_from(WATERMARK.get(), RESUME_SWEEP_LIMIT);
    WATERMARK.set(watermark);
    for op in ops {
        let _ = crate::api::resume(op.id).await;
    }
}

async fn rate_refresh_sweep() {
    let _ = rate::refresh().await;
}
