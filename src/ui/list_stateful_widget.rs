#[allow(unused_imports)]
use tui::{
  backend::Backend,
  layout::{Corner, Rect},
  style::{Color, Modifier, Style},
  text::{Span, Spans},
  widgets::{Block, Borders, List, ListItem, ListState},
  Frame,
};

#[derive(Debug)]
pub struct StatefulList<T> {
  pub state: ListState,
  pub items: Vec<T>,
}

impl<T> Default for StatefulList<T> {
  fn default() -> Self {
    StatefulList { state: ListState::default(), items: Vec::new() }
  }
}

#[allow(unused)]
impl<T> StatefulList<T> {
  pub fn with_items(items: Vec<T>) -> StatefulList<T> {
    StatefulList { state: ListState::default(), items }
  }

  pub fn push(&mut self, item: T) {
    self.items.push(item);
  }

  pub fn next(&mut self) {
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

  pub fn previous(&mut self) {
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

pub fn draw_stateful_list<B: Backend>(
  f: &mut Frame<B>,
  bbox: Rect,
  title: &str,
  stateful_list: &mut StatefulList<String>,
  reverse: bool,
) {
  let mut items: Vec<ListItem> = stateful_list
    .items
    .iter()
    .map(|message| ListItem::new(message.as_str()))
    .collect();

  if reverse {
    items.reverse();
  }

  let widget = List::new(items)
    .block(Block::default().borders(Borders::ALL).title(title))
    .highlight_style(
      Style::default()
        .bg(Color::White)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD),
    )
    .start_corner(Corner::TopLeft);

  f.render_stateful_widget(widget, bbox, &mut stateful_list.state);
}

// tests
#[cfg(test)]
mod tests {
  use super::*;
  use tui::{backend::TestBackend, buffer::Buffer, Terminal};

  #[test]
  fn test_stateful_list() {
    let mut stateful_list = StatefulList::default();
    stateful_list.push("Hello");
    stateful_list.push("World");

    assert_eq!(stateful_list.items.len(), 2);
    assert_eq!(stateful_list.state.selected(), None);

    stateful_list.next();
    assert_eq!(stateful_list.state.selected(), Some(0));

    stateful_list.next();
    assert_eq!(stateful_list.state.selected(), Some(1));

    stateful_list.next();
    assert_eq!(stateful_list.state.selected(), Some(0));

    stateful_list.next();
    assert_eq!(stateful_list.state.selected(), Some(1));

    stateful_list.previous();
    assert_eq!(stateful_list.state.selected(), Some(0));

    stateful_list.previous();
    assert_eq!(stateful_list.state.selected(), Some(1));

    stateful_list.previous();
    assert_eq!(stateful_list.state.selected(), Some(0));

    stateful_list.unselect();
    assert_eq!(stateful_list.state.selected(), None);
  }

  #[test]
  fn test_stateful_list_reverse() {
    let mut stateful_list = StatefulList::default();
    stateful_list.push("Hello");
    stateful_list.push("World");

    assert_eq!(stateful_list.items.len(), 2);
    assert_eq!(stateful_list.state.selected(), None);

    stateful_list.previous();
    assert_eq!(stateful_list.state.selected(), Some(0));

    stateful_list.previous();
    assert_eq!(stateful_list.state.selected(), Some(1));

    stateful_list.previous();
    assert_eq!(stateful_list.state.selected(), Some(0));

    stateful_list.unselect();
    assert_eq!(stateful_list.state.selected(), None);
  }

  #[test]
  fn test_stateful_list_debug() {
    let mut stateful_list = StatefulList::default();
    stateful_list.push("Hello");
    stateful_list.push("World");

    assert_eq!(format!("{stateful_list:?}"), "StatefulList { state: ListState { offset: 0, selected: None }, items: [\"Hello\", \"World\"] }");
  }

  #[test]
  fn test_stateful_list_with_items() {
    let mut stateful_list = StatefulList::with_items(vec!["Hello", "World"]);

    assert_eq!(stateful_list.items.len(), 2);
    assert_eq!(stateful_list.state.selected(), None);

    stateful_list.next();
    assert_eq!(stateful_list.state.selected(), Some(0));

    stateful_list.next();
    assert_eq!(stateful_list.state.selected(), Some(1));

    stateful_list.previous();
    assert_eq!(stateful_list.state.selected(), Some(0));

    stateful_list.previous();
    assert_eq!(stateful_list.state.selected(), Some(1));

    stateful_list.unselect();
    assert_eq!(stateful_list.state.selected(), None);
  }

  #[test]
  fn test_stateful_list_with_title() {
    let backend = TestBackend::new(7, 4);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut stateful_list =
      StatefulList::with_items(vec![String::from("Hello"), String::from("World")]);

    terminal
      .draw(|f| {
        let size = f.size();

        draw_stateful_list(f, size, " x ", &mut stateful_list, false);
      })
      .unwrap();

    #[rustfmt::skip]
    let expected = Buffer::with_lines(vec![
      "┌ x ──┐",
      "│Hello│",
      "│World│",
      "└─────┘"
      ]);
    terminal.backend().assert_buffer(&expected);

    terminal
      .draw(|f| {
        let size = f.size();

        draw_stateful_list(f, size, " x ", &mut stateful_list, true);
      })
      .unwrap();

    #[rustfmt::skip]
    let expected = Buffer::with_lines(vec![
      "┌ x ──┐",
      "│World│",
      "│Hello│",
      "└─────┘"
      ]);
    terminal.backend().assert_buffer(&expected);
  }
}
