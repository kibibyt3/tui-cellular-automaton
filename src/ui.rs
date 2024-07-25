use colors_transform::Hsl;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, WidgetRef},
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

    let title_block = Paragraph::new(Line::from(model.rulestring()))
        .block(Block::default().borders(Borders::ALL).title("Rulestring"))
        .centered();

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
                let mut hue = self.cells()[relative_y][relative_x].age as f32;
                hue *= 2.0;
                hue %= 360.0;

                let mut saturation =
                    100.0 - ((self.cells()[relative_y][relative_x].age as f32 / 360.0) * 25.0);
                if saturation < 0.0 {
                    saturation = 0.0;
                }

                let light =
                    50.0 - ((self.cells()[relative_y][relative_x].age as f32 / 360.0) * 17.0);

                let hsl = Hsl::from(hue, saturation, light);
                let rgb = colors_transform::Color::to_rgb(&hsl);
                if self.cells()[relative_y][relative_x].is_alive {
                    buf.get_mut(x, y).set_char('█').set_fg(Color::Rgb(
                        colors_transform::Color::get_red(&rgb) as u8,
                        colors_transform::Color::get_green(&rgb) as u8,
                        colors_transform::Color::get_blue(&rgb) as u8,
                    ));
                } else {
                    buf.get_mut(x, y).set_char(' ');
                }
                relative_y += 1;
            }
            relative_x += 1;
        }
        if *self.state() == State::Editing {
            let Coords {
                x: mut current_x,
                y: mut current_y,
            } = *self.current_coords();
            current_x += area.left() as i16;
            current_y += area.top() as i16;
            buf.get_mut(current_x as u16, current_y as u16)
                .set_bg(Color::Blue);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::{Message, Preset};

    use super::*;

    #[test]
    fn render_blinker() {
        let mut model = Model::new(5, 5, vec![3], vec![2, 3], 50);
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
