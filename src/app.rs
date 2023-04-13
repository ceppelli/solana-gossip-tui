use std::sync::mpsc::Sender;

use crate::{
  transport::CtrlCmd,
  ui::{list_stateful_widget::StatefulList, table_stateful_widget::StatefulTable},
};

#[derive(Debug)]
pub struct Model {
  pub debug_messages_stateful: StatefulList<String>,
  pub home_stateful_table: StatefulTable<String>,
  pub home_stats_stateful_list: StatefulList<String>,

  pub entrypoints_stateful: StatefulList<String>,
  pub entrypoints: Vec<String>,
  pub entrypoint: Option<String>,
  pub listern_port: u16,
}

impl Default for Model {
  fn default() -> Self {
    let entrypoints = vec![
      "entrypoint.devnet.solana.com:8001",
      "141.98.219.218:8000",
      "72.20.2.47:8000",
      "3.231.25.193:8001",
      "entrypoint.testnet.solana.com:8001",
      "entrypoint.mainnet-beta.solana.com:8001",
    ]
    .into_iter()
    .map(String::from)
    .collect::<Vec<String>>();
    Model {
      debug_messages_stateful: StatefulList::default(),
      home_stateful_table: StatefulTable::default(),
      home_stats_stateful_list: StatefulList::with_items(vec![
        "[Receiver] processed msgs #:0".to_string(),
        "[Sender] processed msgs #:0".to_string(),
        "[Logic] processed msgs #:0".to_string(),
      ]),
      entrypoints_stateful: StatefulList::default(),
      entrypoints,
      entrypoint: None,
      listern_port: 8001,
    }
  }
}

pub struct Context {
  app_id: String,
  app_version: String,

  pub model: Model,
  pub trace: bool,

  pub ctrl_txs: Vec<Sender<CtrlCmd>>,
}

impl Context {
  pub fn new(app_id: String, app_version: String) -> Self {
    Self {
      app_id,
      app_version,
      model: Model::default(),
      trace: false,
      ctrl_txs: Vec::new(),
    }
  }

  pub fn info(&self) -> String {
    format!("AppId:{}, AppVersion:{}\n", self.app_id, self.app_version)
  }

  pub fn debug(&mut self, message: String) {
    self.model.debug_messages_stateful.push(message);
  }
}

#[cfg(test)]
mod mock_test {
  use super::*;

  impl Context {
    #[allow(unused)]
    pub fn new_for_testing() -> Self {
      Self {
        app_id: String::from("_app_id_"),
        app_version: String::from("_app_version_"),
        model: Model::default(),
        trace: false,
        ctrl_txs: Vec::new(),
      }
    }
  }
}
