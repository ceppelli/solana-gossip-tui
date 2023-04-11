use solana_gossip_tui::app::AppContext;
use solana_gossip_tui::common::init_threads;

const APP_ID: &str = "solana_gossip_cli";
const APP_VERSION: &str = "0.0.1+";

fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("[main] starting");
  let mut ctx = AppContext::new(String::from(APP_ID), String::from(APP_VERSION));

  ctx.trace = true;

  let _status = init_threads(&mut ctx);

  if let Ok((_data_rx, _stats_rx, t_hdls)) = _status {
    for t_hdl in t_hdls {
      t_hdl.join().unwrap();
    }
  }

  Ok(())
}
