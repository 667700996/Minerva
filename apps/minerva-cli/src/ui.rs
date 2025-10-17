use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, TryRecvError},
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use minerva_types::events::{EventPayload, SystemEvent};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

const MAX_LOG_ENTRIES: usize = 120;

pub enum UiMessage {
    Event(SystemEvent),
    Shutdown,
}

pub fn run(receiver: Receiver<UiMessage>, summary: String) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let res = run_loop(&mut terminal, receiver, summary.as_str());

    terminal.show_cursor()?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    res
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    receiver: Receiver<UiMessage>,
    summary: &str,
) -> Result<()> {
    let mut logs: VecDeque<String> = VecDeque::with_capacity(MAX_LOG_ENTRIES);
    let mut last_status = String::from("대기 중");
    let mut should_close = false;

    loop {
        let mut receiver_closed = false;
        loop {
            match receiver.try_recv() {
                Ok(UiMessage::Event(event)) => {
                    last_status = summarize_status(&event);
                    let formatted = format_event(&event);
                    if logs.len() == MAX_LOG_ENTRIES {
                        logs.pop_front();
                    }
                    logs.push_back(formatted);
                }
                Ok(UiMessage::Shutdown) => {
                    should_close = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    receiver_closed = true;
                    should_close = true;
                    break;
                }
            }
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let header = Paragraph::new(Line::from(vec![
                Span::styled(
                    "Minerva 상태",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::raw(last_status.clone()),
                Span::raw("  "),
                Span::styled("설정:", Style::default().fg(Color::Magenta)),
                Span::raw(" "),
                Span::raw(summary),
                Span::raw("  "),
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw(" 를 눌러 종료"),
            ]))
            .block(Block::default().borders(Borders::ALL).title("요약"));
            f.render_widget(header, chunks[0]);

            let items: Vec<ListItem> = logs
                .iter()
                .rev()
                .map(|entry| ListItem::new(entry.clone()))
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("최근 이벤트"))
                .highlight_style(Style::default().fg(Color::Yellow));

            f.render_widget(list, chunks[1]);
        })?;

        if should_close && receiver_closed {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }

        if should_close && receiver_closed {
            break;
        }
    }

    Ok(())
}

fn summarize_status(event: &SystemEvent) -> String {
    match &event.payload {
        EventPayload::Lifecycle(lifecycle) => {
            format!("라이프사이클: {:?}", lifecycle.phase)
        }
        EventPayload::Engine(engine) => {
            format!(
                "엔진 깊이 {} / 후보 {}개",
                engine.metrics.depth,
                engine.best_line.len()
            )
        }
        EventPayload::Board(_) => "보드 상태 갱신".to_string(),
        EventPayload::Telemetry(_) => "지연/텔레메트리 수집".to_string(),
        EventPayload::Network(_) => "네트워크 이벤트".to_string(),
        EventPayload::Ops(_) => "운영 알림".to_string(),
        EventPayload::Unknown(_) => "알 수 없는 이벤트".to_string(),
    }
}

fn format_event(event: &SystemEvent) -> String {
    let timestamp = event.timestamp.format("%H:%M:%S");
    match &event.payload {
        EventPayload::Lifecycle(lifecycle) => format!(
            "[{}] Lifecycle::{:?} {}",
            timestamp,
            lifecycle.phase,
            lifecycle.details.clone().unwrap_or_default()
        ),
        EventPayload::Engine(engine) => format!(
            "[{}] Engine depth={} nodes={} best_line={}",
            timestamp,
            engine.metrics.depth,
            engine.metrics.nodes,
            engine.best_line.len()
        ),
        EventPayload::Board(_) => format!("[{}] Board snapshot 수신", timestamp),
        EventPayload::Telemetry(_) => format!("[{}] Telemetry 업데이트", timestamp),
        EventPayload::Network(net) => format!(
            "[{}] Network topic={} payload={}",
            timestamp, net.topic, net.payload
        ),
        EventPayload::Ops(ops) => format!(
            "[{}] Ops {} [{}]",
            timestamp,
            ops.message,
            ops.tags.join(", ")
        ),
        EventPayload::Unknown(value) => format!("[{}] Unknown payload {}", timestamp, value),
    }
}
