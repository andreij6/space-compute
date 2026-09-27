use std::time::Duration;

use crate::rate;

const RESUME_INTERVAL: Duration = Duration::from_secs(300);
const RATE_REFRESH_INTERVAL: Duration = Duration::from_secs(3_600);

pub fn start() {
    ic_cdk_timers::set_timer_interval(RESUME_INTERVAL, resume_sweep);
    ic_cdk_timers::set_timer_interval(RATE_REFRESH_INTERVAL, rate_refresh_sweep);
    ic_cdk_timers::set_timer(Duration::ZERO, rate_refresh_sweep());
}

async fn resume_sweep() {}

async fn rate_refresh_sweep() {
    let _ = rate::refresh().await;
}
