use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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

use tracing::{error, trace};
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};


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
        " Add API ".into(),
        "<a>".blue().bold(),
        " Delete API ".into(),
        "<d>".blue().bold(),
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
            Constraint::Percentage((100 - 50) / 2),
            Constraint::Percentage(50),
            Constraint::Percentage((100 - 50) / 2),
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

    // for api_info in app.apis_infos {
    //     rows.push([
    //         Cell::from(Text::from(String::from(api_info.get("name").unwrap())).alignment(Alignment::Center)),
    //         Cell::from(Text::from(String::from(api_info.get("url").unwrap())).alignment(Alignment::Center)),
    //         Cell::from(Text::from(String::from(api_info.get("url").unwrap())).alignment(Alignment::Center)),
    //         Cell::from(Text::from(String::from(api_info.get("status").unwrap())).alignment(Alignment::Center))
    //             .style(Style::default().fg(Color::Green)),
    //     ]);
    // }

    let mut rows: Vec<Row> = Vec::new();

    for api_info in &app.apis_infos {
        rows.push(Row::new(vec![
            Cell::from(
                Text::from(String::from(api_info.get("name").unwrap()))
                    .alignment(Alignment::Center),
            ),
            Cell::from(
                Text::from(String::from(api_info.get("method").unwrap())).alignment(Alignment::Center),
            ),
            Cell::from(
                Text::from(String::from(api_info.get("url").unwrap())).alignment(Alignment::Center),
            ),
            Cell::from(
                Text::from(String::from(api_info.get("status").unwrap()))
                    .alignment(Alignment::Center),
            )
            .style(Style::default().fg(Color::Green)),

        ]))
        // rows.push([
        // ]);
    }

    // let rows = [
    //     Row::new(vec![
    //         Cell::from(Text::from("Buscar usuários").alignment(Alignment::Center)),
    //         Cell::from(Text::from("GET").alignment(Alignment::Center)),
    //         Cell::from(Text::from("http://test.com.test").alignment(Alignment::Center)),
    //         Cell::from(Text::from("OK").alignment(Alignment::Center))
    //             .style(Style::default().fg(Color::Green)),
    //     ]),
    //     Row::new(vec![
    //         Cell::from(Text::from("Buscar lances").alignment(Alignment::Center)),
    //         Cell::from(Text::from("POST").alignment(Alignment::Center)),
    //         Cell::from(Text::from("http://test.com.test22").alignment(Alignment::Center)),
    //         Cell::from(Text::from("FAILED").alignment(Alignment::Center))
    //             .style(Style::default().fg(Color::Red)),
    //     ]),
    // ];
    // Columns widths are constrained in the same way as Layout...
    let widths = [
        Constraint::Length(20),
        Constraint::Length(20),
        Constraint::Length(20),
        Constraint::Length(10),
    ];
    let table: Table<'static> = Table::new(rows, widths)
        .column_spacing(1)
        .header(
            Row::new(vec![
                Cell::from(Text::from("Name").alignment(Alignment::Center)),
                Cell::from(Text::from("Method").alignment(Alignment::Center)),
                Cell::from(Text::from("Url").alignment(Alignment::Center)),
                Cell::from(Text::from("Status").alignment(Alignment::Center)),
            ])
            .style(Style::new().bold())
            .bottom_margin(1),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    f.render_widget(table, content_center);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
