#[allow(dead_code)]
mod trello;
use crate::trello::TrelloRestLocal as TrelloRest;

#[allow(dead_code)]
mod util;
use crate::util::{
    event::{Event, Events},
    TabsState,
};
use chrono;

#[macro_use]
extern crate log;

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Tabs, List, ListItem},
    Terminal,
};
use std::fs::File;
use std::io::Read;

struct App {
    tabs: TabsState,
    trello: TrelloRest,
}

fn render_board(f: &mut tui::Frame<'_, tui::backend::TermionBackend<termion::screen::AlternateScreen<termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>>, app: &mut App, chunk: tui::layout::Rect) {
    let current_board = &app.tabs.boards[app.tabs.index];
    let inner = Block::default().title(format!("{}", current_board["name"])).borders(Borders::ALL).border_type(BorderType::Rounded);
    f.render_widget(inner, chunk);
    let columns = app.trello.get_board(current_board["id"].to_string());
    let width = Constraint::Percentage(100/columns.len() as u16);
    let inner_chunks = Layout::default().direction(Direction::Horizontal).margin(1).constraints(vec![width; columns.len()].as_ref()).split(chunk);
    columns.members().zip(inner_chunks.iter()).for_each( |(col, chunk)| {
      let cards: Vec<ListItem> = app.trello.get_cards(col["id"].to_string()).members().map(|i| {
          let lines = vec![Spans::from(i["name"].to_string())];
          ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
      }).collect();
      let column = List::new(cards).block(Block::default().title(col["name"].to_string()).borders(Borders::ALL).border_type(BorderType::Rounded)).highlight_style(
          Style::default().bg(Color::LightGreen).add_modifier(Modifier::BOLD),
          ).highlight_symbol(">> ");
      f.render_widget(column, *chunk);
    });
}

fn render_tabs(f: &mut tui::Frame<'_, tui::backend::TermionBackend<termion::screen::AlternateScreen<termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>>, app: &mut App, chunk: tui::layout::Rect) {
        let titles = app
            .tabs
            .boards
            .members()
            .map(|t| {
                let color = if t == &app.tabs.boards[app.tabs.index] { Color::Yellow } else { Color::Green };
                Spans::from(vec![Span::styled(format!("{}", t["name"]), Style::default().fg(color))])
            })
            .collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Boards"))
            .select(app.tabs.index)
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );
        f.render_widget(tabs, chunk);
}

fn setup_terminal() -> Result<tui::Terminal<tui::backend::TermionBackend<termion::screen::AlternateScreen<termion::input::MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>>>>, Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    Ok(Terminal::new(backend).unwrap())
}

fn render_gui(mut app: &mut App) -> Result<(), Box<dyn Error>>{
    let events = Events::new();
    let mut terminal = setup_terminal().unwrap();

    // Main loop
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            let block = Block::default().style(Style::default().bg(Color::White).fg(Color::Black));
            f.render_widget(block, size);
            render_tabs(f, &mut app, chunks[0]);
            render_board(f, &mut app, chunks[1]);
        })?;

        if let Event::Input(input) = events.next()? {
            match input {
                Key::Char('q') => {
                    break;
                }
                Key::Right => app.tabs.next(),
                Key::Left => app.tabs.previous(),
                _ => {}
            }
        }
    }
    Ok(())
}

fn load_config() -> json::JsonValue {
    let mut input = File::open("config.json").unwrap();
    let mut buffer = String::new();
    input.read_to_string(&mut buffer).unwrap();
    return json::parse(&buffer).unwrap();
}


fn main() -> Result<(), Box<dyn Error>> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("output.log")?)
        .apply()?;
    let config = load_config();
    let trello = TrelloRest::new(
        config["base_url"].to_string(),
        config["key"].to_string(),
        config["token"].to_string(),
    );
    let boards = trello.get_boards();

    let mut app = App {
        tabs: TabsState::new(boards),
        trello: trello,
    };
    render_gui(&mut app).unwrap();
    Ok(())
 }
