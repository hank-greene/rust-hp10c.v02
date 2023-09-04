use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::thread;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs
    },
    Terminal,
};

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    println!("4 windows 4 rpn");

    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx ) = mpsc::channel();
    let tick_rate  = Duration::from_millis(200);
    thread::spawn(move ||{
        let mut last_tick = Instant::now();
        loop {

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }

        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;


    loop {

        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let copyright = Paragraph::new("crtp.io all rights reserved")
                .style(Style::default().fg(Color::Black))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Black))
                        .title(" Copyright ")
                        .border_type(BorderType::Rounded)
                );

            let input = Paragraph::new("enter data here")
                .style(Style::default().fg(Color::Black))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Black))
                        .title(" User Input ")
                        .border_type(BorderType::Rounded)
                );

            let stack = Paragraph::new("")
                .style(Style::default().fg(Color::Black))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Black))
                        .title(" RPN Stack ")
                        .border_type(BorderType::Rounded)
                );

            let docs = Paragraph::new(" RPN - Reverse Polish Notation - Notes ")
                .style(Style::default().fg(Color::Black))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Black))
                        .title(" RPN Notes ")
                        .border_type(BorderType::Rounded)
                );



            rect.render_widget(input, chunks[0]);


            let middle_windows = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [Constraint::Percentage(30), Constraint::Percentage(70)]
                )
                .split(chunks[1]);

            rect.render_widget(stack, middle_windows[0]);
            rect.render_widget(docs, middle_windows[1]);


            rect.render_widget(copyright, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Esc => {
                    break;
                }
                _ => {}

            },
            Event::Tick => {}
        }
    }

    disable_raw_mode()?;
    let _ = terminal.clear();
    terminal.show_cursor()?;
    Ok(())
}


