use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::io::{self, Stdout};

use tui::{backend::CrosstermBackend, Terminal};

pub struct XTerminal {
  pub terminal: tui::Terminal<CrosstermBackend<Stdout>>,
}

impl XTerminal {
  pub fn new() -> Result<XTerminal, Box<dyn std::error::Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(XTerminal { terminal })
  }

  pub fn restore(&mut self) -> io::Result<()> {
    // restore terminal
    disable_raw_mode()?;
    execute!(
      self.terminal.backend_mut(),
      LeaveAlternateScreen,
      DisableMouseCapture
    )?;

    self.terminal.show_cursor()?;

    Ok(())
  }
}
