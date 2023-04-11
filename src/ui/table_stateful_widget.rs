#[allow(unused_imports)]
use tui::{
  backend::Backend,
  layout::{Corner, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders},
  Frame,
};
use tui::{
  layout::Constraint,
  widgets::{Cell, Row, Table, TableState},
};

#[derive(Debug)]
pub struct StatefulTable<T> {
  pub state: TableState,
  pub items: Vec<Vec<T>>,
}

impl<T> Default for StatefulTable<T> {
  fn default() -> Self {
    StatefulTable { state: TableState::default(), items: Vec::new() }
  }
}

#[allow(unused)]
impl<T> StatefulTable<T> {
  pub fn with_items(items: Vec<Vec<T>>) -> StatefulTable<T> {
    StatefulTable { state: TableState::default(), items }
  }

  pub fn push_row(&mut self, item: Vec<T>) {
    self.items.push(item);
  }

  pub fn next_row(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i >= self.items.len() - 1 {
          0
        } else {
          i + 1
        }
      },
      None => 0,
    };
    if i < self.items.len() {
      self.state.select(Some(i));
    }
  }

  pub fn previous_row(&mut self) {
    let i = match self.state.selected() {
      Some(i) => {
        if i == 0 && !self.items.is_empty() {
          self.items.len() - 1
        } else if i > 0 {
          i - 1
        } else {
          0
        }
      },
      None => 0,
    };
    if i < self.items.len() {
      self.state.select(Some(i));
    }
  }

  pub fn unselect(&mut self) {
    self.state.select(None);
  }

  pub fn clear(&mut self) {
    self.items.clear();
    self.state.select(None);
  }
}

pub fn draw_stateful_table<B: Backend>(
  f: &mut Frame<B>,
  bbox: Rect,
  title: &str,
  headers: &[&str],
  widths: &[Constraint],
  stateful_table: &mut StatefulTable<String>,
) {
  let selected_style = Style::default().add_modifier(Modifier::REVERSED);
  let header_style = Style::default().bg(Color::LightBlue);
  let header_cells = headers
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));

  let header = Row::new(header_cells)
    .style(header_style)
    .height(1)
    .bottom_margin(1);

  let rows = stateful_table.items.iter().map(|row| {
    let cells = row.iter().map(|c| Cell::from(c.to_string()));
    Row::new(cells).height(1_u16).bottom_margin(0)
  });

  let t = Table::new(rows)
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .highlight_style(selected_style)
    .widths(widths);
  f.render_stateful_widget(t, bbox, &mut stateful_table.state);
}

// tests
#[cfg(test)]
mod tests {
  use std::vec;

  use super::*;
  use tui::{backend::TestBackend, buffer::Buffer, Terminal};

  #[test]
  fn test_stateful_table() {
    let mut stateful_table = StatefulTable::default();
    stateful_table.push_row(vec!["Hello"]);
    stateful_table.push_row(vec!["World"]);

    assert_eq!(stateful_table.items.len(), 2);
    assert_eq!(stateful_table.state.selected(), None);

    stateful_table.next_row();
    assert_eq!(stateful_table.state.selected(), Some(0));

    stateful_table.next_row();
    assert_eq!(stateful_table.state.selected(), Some(1));

    stateful_table.next_row();
    assert_eq!(stateful_table.state.selected(), Some(0));

    stateful_table.next_row();
    assert_eq!(stateful_table.state.selected(), Some(1));

    stateful_table.previous_row();
    assert_eq!(stateful_table.state.selected(), Some(0));

    stateful_table.previous_row();
    assert_eq!(stateful_table.state.selected(), Some(1));

    stateful_table.previous_row();
    assert_eq!(stateful_table.state.selected(), Some(0));

    stateful_table.unselect();
    assert_eq!(stateful_table.state.selected(), None);
  }

  #[test]
  fn test_stateful_table_reverse() {
    let mut stateful_table = StatefulTable::default();
    stateful_table.push_row(vec!["Hello"]);
    stateful_table.push_row(vec!["World"]);

    assert_eq!(stateful_table.items.len(), 2);
    assert_eq!(stateful_table.state.selected(), None);

    stateful_table.previous_row();
    assert_eq!(stateful_table.state.selected(), Some(0));

    stateful_table.previous_row();
    assert_eq!(stateful_table.state.selected(), Some(1));

    stateful_table.previous_row();
    assert_eq!(stateful_table.state.selected(), Some(0));

    stateful_table.unselect();
    assert_eq!(stateful_table.state.selected(), None);
  }

  #[test]
  fn test_stateful_table_debug() {
    let mut stateful_table = StatefulTable::default();
    stateful_table.push_row(vec!["Hello"]);
    stateful_table.push_row(vec!["World"]);

    assert_eq!(format!("{stateful_table:?}"), "StatefulTable { state: TableState { offset: 0, selected: None }, items: [[\"Hello\"], [\"World\"]] }");
  }

  #[test]
  fn test_stateful_table_with_items() {
    let mut stateful_table = StatefulTable::with_items(vec![vec!["Hello"], vec!["World"]]);

    assert_eq!(stateful_table.items.len(), 2);
    assert_eq!(stateful_table.state.selected(), None);

    stateful_table.next_row();
    assert_eq!(stateful_table.state.selected(), Some(0));

    stateful_table.next_row();
    assert_eq!(stateful_table.state.selected(), Some(1));

    stateful_table.previous_row();
    assert_eq!(stateful_table.state.selected(), Some(0));

    stateful_table.previous_row();
    assert_eq!(stateful_table.state.selected(), Some(1));

    stateful_table.unselect();
    assert_eq!(stateful_table.state.selected(), None);
  }

  #[test]
  fn test_stateful_table_with_title() {
    let backend = TestBackend::new(22, 6);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut stateful_table = StatefulTable::with_items(vec![
      vec![String::from("Hello"), String::from("World")],
      vec![String::from("World"), String::from("Hello")],
    ]);

    terminal
      .draw(|f| {
        let size = f.size();

        let withs = [Constraint::Percentage(50), Constraint::Percentage(50)];

        draw_stateful_table(
          f,
          size,
          " title ",
          &["Header1", "Header2"],
          &withs,
          &mut stateful_table,
        );
      })
      .unwrap();

    #[rustfmt::skip]
    let mut expected = Buffer::with_lines(vec![
      "┌ title ─────────────┐",
      "│Header1    Header2  │",
      "│                    │",
      "│Hello      World    │",
      "│World      Hello    │",
      "└────────────────────┘"
      ]);
    expected.set_string(
      1,
      1,
      "Header1    Header2  ",
      Style::default().fg(Color::White).bg(Color::LightBlue),
    );
    expected.set_string(
      11,
      1,
      " ",
      Style::default().fg(Color::Reset).bg(Color::LightBlue),
    );
    expected.set_string(
      1,
      2,
      "                    ",
      Style::default().fg(Color::White).bg(Color::Reset),
    );
    expected.set_string(
      11,
      2,
      " ",
      Style::default().fg(Color::Reset).bg(Color::Reset),
    );
    terminal.backend().assert_buffer(&expected);
  }
}
