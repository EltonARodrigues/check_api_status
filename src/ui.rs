use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, BorderType, Borders, Cell, Padding, Paragraph, Row, Table,
    },
    Frame,
};

use crate::app::App;

#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: $target, $level, ?value, stringify!($ex));
                value
            }
        }
    }};
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };
    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: tracing::Level::DEBUG, $ex)
    };
}

pub fn ui(f: &mut Frame, app: &App) {
    let title: Title<'static> = Title::from(" Health Crab TUI ".bold());

    let instructions = Title::from(Line::from(vec![
        // " Add API ".into(),
        // "<a>".blue().bold(),
        // " Delete API ".into(),
        // "<d>".blue().bold(),
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]));

    let block = Block::default()
        .title(title.alignment(Alignment::Center))
        .title(
            instructions
                .alignment(Alignment::Center)
                .position(Position::Bottom),
        )
        .borders(Borders::ALL)
        .border_set(border::THICK);

    // Create the layout sections.
    let main_content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(f.size());

    f.render_widget(block, main_content_layout[0]);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(5),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(main_content_layout[0]);

    let content_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - 90) / 2),
            Constraint::Percentage(100),
            Constraint::Percentage((100 - 90) / 2),
        ])
        .split(chunks[2])[1];

    let header_info = Paragraph::new("API Monitoring")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .padding(Padding::new(1, 1, 1, 2))
                .title_alignment(Alignment::Center)
                .borders(Borders::BOTTOM)
                .border_type(BorderType::Plain),
        );

    f.render_widget(header_info, chunks[1]);

    let mut rows: Vec<Row> = Vec::new();

    for api_info in &app.apis_infos {
        let data = &api_info.data;

        let colour = match data.status.as_str() {
            "OK" => Color::Green,
            "ERROR" => Color::Red,
            _ => Color::Yellow,
        };

        rows.push(Row::new(vec![
            Cell::from(Text::from(String::from(&data.name)).alignment(Alignment::Center)),
            Cell::from(Text::from(String::from(&data.method)).alignment(Alignment::Center)),
            Cell::from(Text::from(String::from(&data.url)).alignment(Alignment::Center)),
            Cell::from(Text::from(String::from(&data.status)).alignment(Alignment::Center))
                .style(Style::default().fg(colour)),
            Cell::from(
                Text::from(String::from(&api_info.interval.to_string()))
                    .alignment(Alignment::Center),
            ),
        ]))
    }

    let widths = [
        Constraint::Length(30),
        Constraint::Length(10),
        Constraint::Length(100),
        Constraint::Length(20),
        Constraint::Length(20),
    ];
    let table: Table<'static> = Table::new(rows, widths)
        .column_spacing(1)
        .header(
            Row::new(vec![
                Cell::from(Text::from("Name").alignment(Alignment::Center)),
                Cell::from(Text::from("Method").alignment(Alignment::Center)),
                Cell::from(Text::from("Url").alignment(Alignment::Center)),
                Cell::from(Text::from("Status").alignment(Alignment::Center)),
                Cell::from(Text::from("Next Request").alignment(Alignment::Center)),
            ])
            .style(Style::new().bold())
            .bottom_margin(1),
        )
        // .bg(Color::Red) // DEBUG SIZE
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    f.render_widget(table, content_center);
}
