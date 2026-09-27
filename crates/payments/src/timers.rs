use std::time::Duration;

const RESUME_INTERVAL: Duration = Duration::from_secs(300);

pub fn start() {
    ic_cdk_timers::set_timer_interval(RESUME_INTERVAL, resume_sweep);
}

async fn resume_sweep() {}
