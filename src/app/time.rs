use std::{sync::{LazyLock, RwLock}, time::{Duration, Instant}};

pub struct Time {
    pub startup: Instant,
    pub last_update: Option<Instant>,
    pub delta: Duration,
}

pub static TIME: LazyLock<RwLock<Time>> = LazyLock::new(|| {
    RwLock::new(Time {
        startup: Instant::now(),
        delta: Duration::default(),
        last_update: None,
    })
});

pub fn update_time() {
    let mut time = TIME.write().unwrap();
    let now = Instant::now();
    if let Some(last_update) = time.last_update {
        time.delta = now - last_update;
    }
    time.last_update = Some(now);
}
