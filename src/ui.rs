use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, WidgetRef},
    Frame,
};

use crate::app::{Coords, Model, State};

pub fn view(f: &mut Frame, model: &mut Model) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(2),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(model.rulestring());

    f.render_widget(title_block, chunks[0]);

    f.render_widget(&*model, chunks[1]);

    let current_keys_hint = {
        match model.state() {
            State::Editing => Span::styled(
                "(Space) to toggle cell / (WASD) to move / (e) to exit editing mode",
                Style::default().fg(Color::Yellow),
            ),
            State::Running => Span::styled(
                "(e) to enter editing mode",
                Style::default().fg(Color::Yellow),
            ),
            State::Done => Span::styled("", Style::default()),
        }
    };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    f.render_widget(key_notes_footer, chunks[2]);
}

impl WidgetRef for Model {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut relative_x = 0;
        for x in area.left()..area.right() {
            let mut relative_y = 0;
            for y in area.top()..area.bottom() {
                if self.cells()[relative_y][relative_x] {
                    buf.get_mut(x, y).set_char('█');
                } else {
                    buf.get_mut(x, y).set_char(' ');
                }
                relative_y += 1;
            }
            relative_x += 1;
        }
        if *self.state() == State::Editing {
            let Coords { x: mut current_x, y: mut current_y } = *self.current_coords();
            current_x += area.left() as i16;
            current_y += area.top() as i16;
            buf.get_mut(current_x as u16, current_y as u16).set_bg(Color::Blue);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{Message, Preset};

    use super::*;

    #[test]
    fn render_blinker() {
        let mut model = Model::new(5, 5, vec![3], vec![2, 3]);
        let mut buf = Buffer::empty(Rect::new(0, 0, 6, 6));
        model.load_preset(Preset::Blinker);
        model.render_ref(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            "      ",
            "███   ",
            "      ",
            "      ",
            "      ",
            "      ",
        ]);

        assert_eq!(buf, expected);

        model.update(Message::ToggleEditing);
        model.update(Message::Idle);
        model.render_ref(buf.area, &mut buf);

        let expected = Buffer::with_lines(vec![
            " █    ", " █    ", " █    ", "      ", "      ", "      ",
        ]);

        assert_eq!(buf, expected);
    }
}
