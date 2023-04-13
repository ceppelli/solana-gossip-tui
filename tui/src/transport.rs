pub(crate) mod receiver;
pub(crate) mod sender;

use std::time::Duration;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CtrlCmd {
    Stop,
    Counter,
}

#[derive(Debug)]
pub enum StatsId {
    Receiver,
    Sender,
    Logic,
}

pub struct Stats {
    pub id: StatsId,
    pub counter: u32,
}

const RECV_TIMEOUT: Duration = Duration::from_millis(1000);
