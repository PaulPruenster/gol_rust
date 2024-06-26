use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::Rect,
    prelude::{CrosstermBackend, Span, Stylize, Terminal},
};
use std::{
    io::{stdout, Result, Stdout},
    thread,
    time::Duration,
};

struct GameData {
    width: i32,
    height: i32,
    board: Vec<Vec<i32>>,
    game_running: bool,
    reload: bool,
    color: Color,
}

enum Color {
    CYAN,
    PINK,
    GREEN,
    WHITE,
}

fn count_neighbors(gd: &mut GameData, row: i32, col: i32) -> i32 {
    let mut count = 0;
    for i in row - 1..row + 2 {
        for j in col - 1..col + 2 {
            if (i == row && j == col) || (i < 0 || j < 0) || (i >= gd.width || j >= gd.height) {
                continue;
            }
            if gd.board[i as usize][j as usize] != 0 {
                count += 1;
            }
        }
    }
    return count;
}

fn calculate_gol(gd: &mut GameData) {
    let mut tmp_board = vec![vec![0; gd.height as usize]; gd.width as usize];

    for i in 0..gd.width as usize {
        for j in 0..gd.height as usize {
            let n_alive = count_neighbors(gd, i as i32, j as i32);

            if gd.board[i][j] != 0 && (n_alive == 2 || n_alive == 3) {
                tmp_board[i][j] = n_alive;
            } else if gd.board[i][j] == 0 && n_alive == 3 {
                tmp_board[i][j] = n_alive;
            } else {
                tmp_board[i][j] = 0;
            }
        }
    }

    gd.board = tmp_board;
}

fn render_gol(gd: &GameData, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal.draw(|frame| {
        // Upper: ▀
        // Lower: ▄
        // Full:  █

        for col in 0..gd.board.len() {
            for row in 0..gd.board[col].len() / 2 {
                let area = Rect {
                    x: col as u16,
                    y: row as u16,
                    width: 1,
                    height: 1,
                };

                let mut text: Span = " ".into();
                if gd.board[col][row * 2] != 0 && gd.board[col][row * 2 + 1] != 0 {
                    text = "█".cyan();
                } else if gd.board[col][row * 2] != 0 {
                    text = "▀".cyan();
                } else if gd.board[col][row * 2 + 1] != 0 {
                    text = "▄".cyan()
                }

                let colored_text: Span = match gd.color {
                    Color::CYAN => text.cyan(),
                    Color::PINK => text.magenta(),
                    Color::GREEN => text.green(),
                    Color::WHITE => text.white(),
                };

                frame.render_widget(colored_text, area);
            }
        }
    })?;

    Ok(())
}

fn handle_keys(gd: &mut GameData) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(10))? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                gd.game_running = false;
            }

            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('r') {
                gd.reload = true;
            }

            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('c') {
                gd.color = match gd.color {
                    Color::CYAN => Color::PINK,
                    Color::PINK => Color::GREEN,
                    Color::GREEN => Color::WHITE,
                    Color::WHITE => Color::CYAN,
                }
            }
        }
    }
    Ok(())
}

fn init_random(
    terminal: &Terminal<CrosstermBackend<Stdout>>,
    old_color: Option<Color>,
) -> Result<GameData> {
    let terminal_size = terminal.size()?;
    let h: usize = terminal_size.height as usize * 2;
    let w: usize = terminal_size.width.into();

    let mut gd = GameData {
        width: w as i32,
        height: h as i32,
        board: vec![vec![0; h]; w],
        game_running: true,
        reload: false,
        color: old_color.unwrap_or(Color::CYAN),
    };

    for row in 0..gd.board.len() {
        for col in 0..gd.board[row].len() {
            if rand::random() {
                gd.board[row][col] = 1;
            }
        }
    }

    Ok(gd)
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut gd: GameData = init_random(&terminal, None)?;

    while gd.game_running {
        // re init game on resize of terminal or r button press
        let term_size = terminal.size()?;
        if gd.reload
            || term_size.width as i32 != gd.width
            || term_size.height as i32 * 2 != gd.height
        {
            gd.reload = false;
            gd = init_random(&terminal, Some(gd.color))?;
        }

        calculate_gol(&mut gd);

        render_gol(&mut gd, &mut terminal)?;

        handle_keys(&mut gd)?;

        thread::sleep(Duration::from_millis(10))
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    return Ok(());
}
