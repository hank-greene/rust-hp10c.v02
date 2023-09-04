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

        //terminal.draw(|rect| {

        //})?;
        //break
        match rx.recv()? {
            Event::Input(event) => match event.code {
                //KeyCode::Char('q') => {
                //    disable_raw_mode()?;
                //    terminal.show_cursor()?;
                //    break;
                //}
                KeyCode::Esc => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                _ => {}

            },
            Event::Tick => {}
        }
    }

    Ok(())
}

