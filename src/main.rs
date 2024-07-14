use std::{io::stdout, mem::swap};

use anyhow::Result;

use rand::random;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    layout::Rect,
    text::Span,
    Terminal,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Board {
    values: Vec<Vec<bool>>,
}

impl Board {
    fn new() -> Self {
        Self {
            values: vec![vec![]],
        }
    }
    fn random(width: usize, height: usize) -> Self {
        let mut board = Self {
            values: vec![vec![false; width]; height],
        };
        for i in 0..height {
            for j in 0..width {
                board.values[i][j] = random();
            }
        }
        board
    }
    fn advance(&mut self, based_on: &mut Self) {
        for i in 0..self.height() {
            for j in 0..self.width() {
                let neighbours_count = based_on.neighbors_count(i, j);
                match (based_on.values[i][j], neighbours_count) {
                    (true, 0..=1) => self.values[i][j] = false,
                    (true, 4..) => self.values[i][j] = false,
                    (false, 3) => self.values[i][j] = true,
                    (_, _) => self.values[i][j] = based_on.values[i][j],
                }
            }
        }
    }
    fn width(&self) -> usize {
        self.values.first().map(|col| col.len()).unwrap_or(0)
    }
    fn height(&self) -> usize {
        self.values.len()
    }
    fn neighbors_count(&self, i: usize, j: usize) -> usize {
        [
            self.values[mod_dec(i, self.height())][mod_dec(j, self.width())],
            self.values[mod_dec(i, self.height())][j],
            self.values[mod_dec(i, self.height())][mod_inc(j, self.width())],
            self.values[i][mod_dec(j, self.width())],
            self.values[i][mod_inc(j, self.width())],
            self.values[mod_inc(i, self.height())][mod_dec(j, self.width())],
            self.values[mod_inc(i, self.height())][j],
            self.values[mod_inc(i, self.height())][mod_inc(j, self.width())],
        ]
        .into_iter()
        .filter(|&cell| cell)
        .count()
    }
}

fn mod_inc(value: usize, modulo: usize) -> usize {
    if value == modulo - 1 {
        0
    } else {
        value + 1
    }
}

fn mod_dec(value: usize, modulo: usize) -> usize {
    if value == 0 {
        modulo - 1
    } else {
        value - 1
    }
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut is_paused = false;

    let mut board1 = Board::new();
    let mut board2 = Board::new();

    let mut current = &mut board1;
    let mut previous = &mut board2;

    loop {
        terminal.draw(|frame| {
            let area = frame.size();

            if current.width() != area.width as usize / 2
                || current.height() != area.height as usize
            {
                *current = Board::random(area.width as usize / 2, area.height as usize);
                *previous = current.clone();
            }

            for i in 0..current.height() {
                for j in 0..current.width() {
                    if current.values[i][j] {
                        frame
                            .render_widget(Span::raw("██"), Rect::new(j as u16 * 2, i as u16, 2, 1))
                    }
                }
            }
        })?;

        if !is_paused {
            swap(&mut current, &mut previous);

            current.advance(previous);
        }

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('p') => is_paused = !is_paused,
                        KeyCode::Char('r') => {
                            let size = terminal.size().unwrap();
                            *current = Board::random(size.width as usize / 2, size.height as usize);
                            *previous = current.clone();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
