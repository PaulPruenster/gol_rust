use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::Rect,
    prelude::{CrosstermBackend, Stylize, Terminal},
};
use std::{
    io::{stdout, Result, Stdout},
    thread,
    time::Duration,
};

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

    for i in 0..gd.board.len() {
        for j in 0..gd.board[i].len() {
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

                let mut text = " ".into();
                if gd.board[col][row * 2] != 0 && gd.board[col][row * 2 + 1] != 0 {
                    //text = "█".green();
                    text = "█".white();
                } else if gd.board[col][row * 2] != 0 {
                    text = "▀".white();
                } else if gd.board[col][row * 2 + 1] != 0 {
                    text = "▄".white()
                }

                frame.render_widget(text, area);
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
        }
    }
    Ok(())
}

fn init_random(terminal: &Terminal<CrosstermBackend<Stdout>>) -> Result<GameData> {
    let terminal_size = terminal.size()?;
    let h: usize = terminal_size.height.into();
    let w: usize = terminal_size.width.into();

    let mut gd = GameData {
        width: w as i32,
        height: h as i32,
        board: vec![vec![0; h]; w],
        game_running: true,
        reload: false,
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

struct GameData {
    width: i32,
    height: i32,
    board: Vec<Vec<i32>>,
    game_running: bool,
    reload: bool,
}

fn main() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut gd: GameData = init_random(&terminal)?;

    while gd.game_running {
        // re init game on resize of terminal
        let term_size = terminal.size()?;
        if gd.reload || term_size.width as i32 != gd.width || term_size.height as i32 != gd.height {
            gd = init_random(&terminal)?;
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
