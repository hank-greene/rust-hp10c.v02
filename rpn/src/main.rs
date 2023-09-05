use crossterm::event::{Event,EventStream,KeyCode,KeyEvent};
use crossterm::terminal::{ disable_raw_mode, enable_raw_mode };
use futures::StreamExt;
use std::io::{self, Write};
use std::time::Duration;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{
        Block, BorderType, Borders, Paragraph
    },
};
use tui::{backend::CrosstermBackend, Terminal};
use tui_textarea::TextArea;

// libhunt.com/compare-tokio-vs-async-std
// https://users.rust-lang.org/t/text-mode-terminal-application-with-asynchronous-input-output/74760

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    enable_raw_mode().expect("can run in raw mode");

    let ( input_s, mut input_r ) = tokio::sync::mpsc::channel::<Option<String>>(100);
    let ( output_s, mut output_r) = tokio::sync::mpsc::channel::<String>(100);
    tokio::spawn(async move {

        let mut user_entry = Some(String::from("Startup string; Welcome to RPN"));
        let mut send_user_entry: bool = true;

        loop {
            tokio::time::sleep(Duration::from_millis(200)).await;
            if let Ok(new_state) = input_r.try_recv() {
                user_entry = new_state;
                send_user_entry = true;
            }

            if send_user_entry {
                match user_entry.clone() {
                    Some(line) => {
                        match line.as_str() {
                            _ => {
                                output_s
                                    .send(format!("{}",line))
                                    .await
                                    .unwrap();
                                send_user_entry = false;
                            }
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    });

    let mut msg: Vec<String> = Vec::new();
    let mut log: Vec<String> = Vec::new();
    let mut stack: Vec<String> = Vec::new();

    let mut input_buffer: String = String::new();
    let mut textarea: TextArea<'_> = TextArea::default();

    textarea.set_block(
        Block::default()
            .borders(Borders::ALL)
            .title("Input"),   
    );
    textarea.set_cursor_style(Style::default());

    let mut reader = EventStream::new();

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

            let input = Paragraph::new(&*input_buffer)
                .style(Style::default().fg(Color::Black))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::Black))
                        .title(" Input ")
                        .border_type(BorderType::Rounded)
                );

            rect.render_widget(input, chunks[0]);

            let stack = Paragraph::new(stack.join("\n"))
                .style(Style::default().fg(Color::Black))
                .alignment(Alignment::Left)
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

            let middle_windows = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [Constraint::Percentage(30), Constraint::Percentage(70)]
                )
                .split(chunks[1]);

            rect.render_widget(stack, middle_windows[0]);
            rect.render_widget(docs, middle_windows[1]);

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

            rect.render_widget(copyright, chunks[2]);

        })?;


        tokio::select! {

            user_entry = output_r.recv() => {

                let entry: String = user_entry.unwrap();
                if entry.contains("help") {

                } else if entry.contains("p") {

                } else {
                    stack.push(entry);
                }
            }

            user_event = reader.next() => {
                let event = match user_event {
                    None => break,
                    Some(Err(_)) => break,
                    Some(Ok(event)) => event,
                };

                match event {
                    //Quit
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc, ..
                    }) => {
                        break;
                    }
                    // Send link
                    Event::Key(KeyEvent {
                        code: KeyCode::Enter, ..
                    }) => {
                        input_s.send(Some(input_buffer.clone())).await.unwrap();
                        input_buffer.clear();
                    }
                    //Type character
                    Event::Key(KeyEvent{
                        code: KeyCode::Char(c), ..
                    }) => {
                        input_buffer.push(c);
                    }
                    _ => {
                        write!(terminal.backend_mut().by_ref(), "\x07")?;
                        terminal.backend_mut().flush()?;
                    }
                }
            }

        }
    }

    disable_raw_mode()?;
    let _ = terminal.clear();
    terminal.show_cursor()?;
    Ok(())
}


