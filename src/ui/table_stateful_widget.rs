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
  headers: Vec<&str>,
  stateful_table: &mut StatefulTable<String>,
) {
  let selected_style = Style::default().add_modifier(Modifier::REVERSED);
  let normal_style = Style::default().bg(Color::LightBlue);
  let header_cells = headers
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::White)));

  let header = Row::new(header_cells)
    .style(normal_style)
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
    .widths(&[
      Constraint::Percentage(14),
      Constraint::Percentage(6),
      Constraint::Percentage(16),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
      Constraint::Percentage(6),
    ]);
  f.render_stateful_widget(t, bbox, &mut stateful_table.state);
}

// tests
#[cfg(test)]
mod tests {
  // use super::*;
  // use tui::{backend::TestBackend, buffer::Buffer, Terminal};

  // #[test]
  // fn test_stateful_list() {
  //   let mut stateful_list = StatefulTable::new();
  //   stateful_list.push("Hello");
  //   stateful_list.push("World");

  //   assert_eq!(stateful_list.items.len(), 2);
  //   assert_eq!(stateful_list.state.selected(), None);

  //   stateful_list.next();
  //   assert_eq!(stateful_list.state.selected(), Some(0));

  //   stateful_list.next();
  //   assert_eq!(stateful_list.state.selected(), Some(1));

  //   stateful_list.next();
  //   assert_eq!(stateful_list.state.selected(), Some(0));

  //   stateful_list.next();
  //   assert_eq!(stateful_list.state.selected(), Some(1));

  //   stateful_list.previous();
  //   assert_eq!(stateful_list.state.selected(), Some(0));

  //   stateful_list.previous();
  //   assert_eq!(stateful_list.state.selected(), Some(1));

  //   stateful_list.previous();
  //   assert_eq!(stateful_list.state.selected(), Some(0));

  //   stateful_list.unselect();
  //   assert_eq!(stateful_list.state.selected(), None);
  // }

  // #[test]
  // fn test_stateful_list_reverse() {
  //   let mut stateful_list = StatefulTable::new();
  //   stateful_list.push("Hello");
  //   stateful_list.push("World");

  //   assert_eq!(stateful_list.items.len(), 2);
  //   assert_eq!(stateful_list.state.selected(), None);

  //   stateful_list.previous();
  //   assert_eq!(stateful_list.state.selected(), Some(0));

  //   stateful_list.previous();
  //   assert_eq!(stateful_list.state.selected(), Some(1));

  //   stateful_list.previous();
  //   assert_eq!(stateful_list.state.selected(), Some(0));

  //   stateful_list.unselect();
  //   assert_eq!(stateful_list.state.selected(), None);
  // }

  // #[test]
  // fn test_stateful_list_debug() {
  //   let mut stateful_list = StatefulTable::new();
  //   stateful_list.push("Hello");
  //   stateful_list.push("World");

  //   assert_eq!(format!("{stateful_list:?}"), "StatefulList { state: ListState { offset: 0, selected: None }, items: [\"Hello\", \"World\"] }");
  // }

  // #[test]
  // fn test_stateful_list_with_items() {
  //   let mut stateful_list = StatefulTable::with_items(vec!["Hello", "World"]);

  //   assert_eq!(stateful_list.items.len(), 2);
  //   assert_eq!(stateful_list.state.selected(), None);

  //   stateful_list.next();
  //   assert_eq!(stateful_list.state.selected(), Some(0));

  //   stateful_list.next();
  //   assert_eq!(stateful_list.state.selected(), Some(1));

  //   stateful_list.previous();
  //   assert_eq!(stateful_list.state.selected(), Some(0));

  //   stateful_list.previous();
  //   assert_eq!(stateful_list.state.selected(), Some(1));

  //   stateful_list.unselect();
  //   assert_eq!(stateful_list.state.selected(), None);
  // }

  // #[test]
  // fn test_stateful_list_with_title() {
  //   let backend = TestBackend::new(7, 4);
  //   let mut terminal = Terminal::new(backend).unwrap();

  //   let mut stateful_list =
  //     StatefulTable::with_items(vec![String::from("Hello"), String::from("World")]);

  //   terminal
  //     .draw(|f| {
  //       let size = f.size();

  //       draw_stateful_table(f, size, " x ", &mut stateful_list, false);
  //     })
  //     .unwrap();

  //   #[rustfmt::skip]
  //   let expected = Buffer::with_lines(vec![
  //     "┌ x ──┐",
  //     "│Hello│",
  //     "│World│",
  //     "└─────┘"
  //     ]);
  //   terminal.backend().assert_buffer(&expected);

  //   terminal
  //     .draw(|f| {
  //       let size = f.size();

  //       draw_stateful_table(f, size, " x ", &mut stateful_list, true);
  //     })
  //     .unwrap();

  //   #[rustfmt::skip]
  //   let expected = Buffer::with_lines(vec![
  //     "┌ x ──┐",
  //     "│World│",
  //     "│Hello│",
  //     "└─────┘"
  //     ]);
  //   terminal.backend().assert_buffer(&expected);
  // }
}
