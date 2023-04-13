use solana_gossip_tui::app::Context;
use solana_gossip_tui::common::init_threads;

const APP_ID: &str = "solana_gossip_cli";
const APP_VERSION: &str = "0.0.1+";

fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("[main] starting");
  let mut ctx = Context::new(String::from(APP_ID), String::from(APP_VERSION));

  ctx.model.entrypoint = Some(ctx.model.entrypoints[1].clone());

  ctx.trace = true;

  let (_, _, t_hdls) = init_threads(&mut ctx)?;

  for t_hdl in t_hdls {
    t_hdl.join().unwrap();
  }

  Ok(())
}
