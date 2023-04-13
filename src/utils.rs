use std::time::{SystemTime, UNIX_EPOCH};

#[allow(clippy::cast_possible_truncation)]
pub fn since_the_epoch_millis() -> u64 {
  let start = SystemTime::now();
  let since_the_epoch = start
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards");

  since_the_epoch.as_millis() as u64
}
