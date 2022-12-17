use std::collections::HashSet;
use std::fmt::Write;
use std::io;

use arboard::Clipboard;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode,
    enable_raw_mode,
    EnterAlternateScreen,
    LeaveAlternateScreen,
};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans, Text};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::{Frame, Terminal};

use crate::error::Result;
use crate::{LetterStatus, Spot, ALPHABETS};

/// App holds the state of the application
struct App {
    input: String,
    message: Option<String>,
    guesses: Vec<[Spot; 5]>,
    alphabet_statuses: [Option<LetterStatus>; 26],
    attempts: usize,
    word: String,
    allowed_guesses: HashSet<String>,
    index: usize,
}

impl App {
    fn new(word: String, allowed_guesses: HashSet<String>, index: usize) -> Self {
        Self {
            input: String::new(),
            message: None,
            guesses: Vec::new(),
            alphabet_statuses: [None; 26],
            attempts: 0,
            word,
            allowed_guesses,
            index,
        }
    }
}

pub fn main(word: String, allowed_guesses: HashSet<String>, index: usize) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.autoresize()?;

    // create app and run it
    let app = App::new(word, allowed_guesses, index);
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    res
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let mut win = false;
    terminal.show_cursor()?;
    loop {
        terminal.draw(|f| {
            if win {
                success_ui(f, &app);
            } else if app.attempts == 6 {
                loss_ui(f, &app);
            } else {
                game_ui(f, &app);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if app.attempts == 6 || win {
                if let KeyCode::Char('c') = key.code {
                    let mut text = String::new();
                    let los = result_text_spans(&app);
                    for (i, spans) in los.iter().enumerate() {
                        for span in &spans.0 {
                            write!(&mut text, "{}", span.content)?;
                        }
                        writeln!(&mut text)?;
                        if i == 0 {
                            writeln!(&mut text)?;
                        }
                    }
                    let mut clipboard = Clipboard::new()?;
                    clipboard.set_text(text)?;
                }
                return Ok(());
            }
            match key.code {
                KeyCode::Enter => {
                    if app.input.len() != 5 || !app.allowed_guesses.contains(&app.input) {
                        app.message =
                            Some("Not a valid five letter word. Try again... ".to_string());
                        continue;
                    }

                    app.message = None;

                    let spots = get_spots(&app.input, &app.word);
                    app.guesses.push(spots);
                    app.attempts += 1;

                    if app.input == app.word {
                        win = true;
                        continue;
                    }

                    for spot in spots {
                        app.alphabet_statuses[letter_to_index(spot.letter).unwrap_or_default()] =
                            Some(spot.status);
                    }

                    app.input.clear();
                },
                KeyCode::Char(c) => {
                    app.input.push(c.to_ascii_uppercase());
                },
                KeyCode::Backspace => {
                    app.input.pop();
                },
                KeyCode::Esc => return Ok(()),
                _ => {},
            }
        }
    }
}

fn game_ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Max(2),
                Constraint::Length(8),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    let mut msg = vec![Spans::from(vec![
        Span::raw("Press "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to stop editing, "),
        Span::styled("enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to submit a word."),
    ])];

    if let Some(message) = &app.message {
        msg.push(Spans::from(Span::styled(
            message,
            Style::default().fg(Color::Red),
        )));
    }

    let mut text = Text::from(msg);
    text.patch_style(Style::default());
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let mut text = app
        .guesses
        .iter()
        .map(|g| {
            let mut spans = Vec::with_capacity(5);
            for spot in g {
                spans.push(Span::styled(
                    spot.letter.to_string(),
                    Style::default().fg(color_from_status(spot.status)),
                ));
            }
            Spans::from(spans)
        })
        .collect::<Vec<_>>();
    text.push(Spans::from(Span::raw({
        if app.input.is_empty() {
            "_____"
        } else {
            &app.input
        }
    })));
    let guesses_widget = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(format!("Guesses {}/6", app.attempts))
                .title_alignment(Alignment::Center),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(guesses_widget, chunks[1]);

    f.render_widget(alphabets_widget(&app.alphabet_statuses), chunks[2]);
}

fn color_from_status(status: LetterStatus) -> Color {
    match status {
        LetterStatus::Correct => Color::Green,
        LetterStatus::Incorrect => Color::Yellow,
        LetterStatus::NotInWord => Color::DarkGray,
    }
}

/// Returns the index of the given letter in the English alphabet.
///
/// Indexing starts at zero.
///
/// Returns [`None`] if the given letter is not present in the English alphabet.
fn letter_to_index(letter: char) -> Option<usize> {
    if letter.is_alphabetic() {
        Some((letter.to_ascii_uppercase() as u8 - b'A') as usize)
    } else {
        None
    }
}

fn alphabets_widget<'a>(alphabet_statuses: &[Option<LetterStatus>; 26]) -> Paragraph<'a> {
    let mut spans = vec![Vec::new()];
    for (index, status) in alphabet_statuses.iter().enumerate() {
        let color = status.map_or(Color::Reset, color_from_status);

        spans.last_mut().unwrap().push(Span::styled(
            ALPHABETS[index].to_string(),
            Style::default().fg(color),
        ));

        if index != 23 && (index + 1) % 8 == 0 {
            spans.push(Vec::new());
        }
    }

    let mut text = Vec::new();
    for span in spans {
        text.push(Spans::from(span));
    }

    Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title("Alphabets")
                .title_alignment(Alignment::Center),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}

fn success_ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(8)].as_ref())
        .split(f.size());

    let mut spans = vec![
        Spans::from(vec![
            Span::raw("Correct! The word was "),
            Span::styled(
                &app.word,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("."),
        ]),
        Spans::from(Span::raw("")),
        Spans::from(Span::raw("")),
    ];

    spans.extend_from_slice(&result_text_spans(app));
    add_copy_result_spans(&mut spans);

    let widget = Paragraph::new(spans)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title("RESULT")
                .title_alignment(Alignment::Center),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(widget, chunks[0]);
}

fn loss_ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(8)].as_ref())
        .split(f.size());

    let mut spans = vec![
        Spans::from(vec![
            Span::raw("The correct word was "),
            Span::styled(
                &app.word,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("."),
        ]),
        Spans::from(Span::raw("")),
        Spans::from(Span::raw("")),
    ];

    spans.extend_from_slice(&result_text_spans(app));
    add_copy_result_spans(&mut spans);

    let widget = Paragraph::new(spans)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title("Result!")
                .title_alignment(Alignment::Center),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(widget, chunks[0]);
}

fn result_text_spans(app: &App) -> Vec<Spans> {
    let mut los = vec![Spans::from(Span::raw(format!(
        "Wordle {} {}/6",
        app.index + 1,
        app.attempts
    )))];

    for guess in &app.guesses {
        let mut spans = Vec::new();
        for spot in guess {
            spans.push(Span::raw(emoji_from_status(spot.status)));
        }
        los.push(Spans::from(spans));
    }

    los
}

fn add_copy_result_spans(los: &mut Vec<Spans>) {
    los.extend_from_slice(&[
        Spans::from(Span::raw("")),
        Spans::from(Span::raw("")),
        Spans::from(vec![Span::styled(
            "Press C to copy result to clipboard",
            Style::default().add_modifier(Modifier::DIM),
        )]),
    ]);
}

fn emoji_from_status(status: LetterStatus) -> &'static str {
    match status {
        LetterStatus::Correct => "🟩",
        LetterStatus::Incorrect => "🟨",
        LetterStatus::NotInWord => "⬛",
    }
}

fn get_spots(input: &str, word: &str) -> [Spot; 5] {
    let mut spots = [Spot::default(); 5];

    for (index, letter) in input.chars().enumerate() {
        if letter == word.as_bytes()[index] as char {
            spots[index] = Spot::correct(letter);
        } else if word.contains(|c| c == letter) {
            spots[index] = Spot::incorrect(letter);
        } else {
            spots[index] = Spot::not_in_word(letter);
        }
    }

    spots
}
