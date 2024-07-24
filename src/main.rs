use std::{error::Error, io, time::Duration};

use app::{Cli, Config, Direction, Message, Model, Preset, State};
use clap::Parser;
use errors::install_hooks;
use ratatui::{
    crossterm::{
        event::{self, poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{
            self, disable_raw_mode, enable_raw_mode, size, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
    },
    prelude::{Backend, CrosstermBackend},
    Terminal,
};
use tui::init;
use ui::view;

mod app;
mod errors;
mod tui;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    
    let cli = Cli::parse();

    let rulestring = {
        if let Some(item) = cli.rulestring.as_deref() {
            String::from(item)
        } else {
            String::from("B3/S23")
        }
    };

    let preset_string = {
        if let Some(item) = cli.preset_string.as_deref() {
            String::from(item)
        } else {
            String::from("None")
        }
    };

    let tickrate = {
        if let Some(item) = cli.tickrate {
            item
        } else {
            100
        }
    };

    let config = Config::build(&preset_string, &rulestring, tickrate);

    install_hooks()?;
    let mut terminal = init()?;

    let (columns, rows) = size()?;

    let mut model = Model::new(
        (rows as i16) - 3 - 1,
        (columns as i16) - 1,
        config.rule.birth_list,
        config.rule.survival_list,
        config.tickrate,
    );

    model.load_preset(config.preset);
    run_model(&mut terminal, &mut model)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    Ok(())
}

fn run_model<B: Backend>(terminal: &mut Terminal<B>, model: &mut Model) -> io::Result<()> {
    loop {
        terminal.draw(|f| view(f, model))?;
        match model.state() {
            State::Running => {
                terminal.draw(|f| view(f, model))?;
                if poll(Duration::from_millis(model.tickrate() as u64))? {
                    if let Event::Key(key) = read()? {
                        if key.kind == event::KeyEventKind::Release {
                            continue;
                        }

                        if let KeyCode::Char(ch) = key.code {
                            match ch {
                                'e' => {
                                    model.update(Message::ToggleEditing);
                                }
                                'q' => {
                                    model.update(Message::Quit);
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    model.update(Message::Idle);
                }
            }

            State::Editing => {
                if let Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Release {
                        continue;
                    }

                    if let KeyCode::Char(ch) = key.code {
                        match ch {
                            'w' => {
                                model.update(Message::Move(Direction::Up));
                            }
                            'a' => {
                                model.update(Message::Move(Direction::Left));
                            }
                            's' => {
                                model.update(Message::Move(Direction::Down));
                            }
                            'd' => {
                                model.update(Message::Move(Direction::Right));
                            }
                            'e' => {
                                model.update(Message::ToggleEditing);
                            }
                            'q' => {
                                model.update(Message::Quit);
                            }
                            ' ' => {
                                model.update(Message::ToggleCellState);
                            }
                            _ => {}
                        }
                    }
                }
            }

            State::Done => {
                break;
            }
        }
    }

    Ok(())
}
