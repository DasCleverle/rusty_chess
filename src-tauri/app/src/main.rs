// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;

use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use chess::{Board, Color, Coord, Move, Piece};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Debug, thiserror::Error)]
enum CommandError {
    #[error(transparent)]
    Error(#[from] anyhow::Error),
}

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

type CommandResult<T = ()> = anyhow::Result<T, CommandError>;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BoardPayload {
    pieces: Vec<Piece>,
    turn: Color,
    white_checked: bool,
    black_checked: bool,
    winner: Option<Color>,
}

impl BoardPayload {
    pub fn new(board: &Board) -> Self {
        return BoardPayload {
            pieces: board.pieces(),
            turn: board.turn(),
            white_checked: board.white_checked(),
            black_checked: board.black_checked(),
            winner: board.winner()
        };
    }
}

struct BoardState {
    board: Arc<Mutex<Board>>,
}

fn mutate_board<T, E>(app: AppHandle, state: State<BoardState>, mutation: T) -> Result<()>
where
    T: FnOnce(&mut Board) -> Result<(), E>,
    E: Error + Send + Sync + 'static,
{
    let mut board = get_board(state);

    mutation(&mut *board)?;
    app.emit_all("update", BoardPayload::new(&*board))?;

    return Ok(());
}

fn get_board(state: State<BoardState>) -> std::sync::MutexGuard<'_, Board> {
    return state.inner().board.lock().unwrap();
}

#[tauri::command]
fn get_board_cmd(state: State<BoardState>) -> BoardPayload {
    return BoardPayload::new(&*get_board(state));
}

#[tauri::command]
fn get_available_moves(coord: Coord, state: State<BoardState>) -> CommandResult<Vec<Move>> {
    let board = get_board(state);
    let all_moves = chess::get_moves(board.turn(), &*board);
    let moves_from = all_moves.into_iter().filter(|mv| mv.from == coord).collect::<Vec<Move>>();

    return Ok(moves_from);
}

#[tauri::command]
fn exec_move(mv: Move, app: AppHandle, state: State<BoardState>) -> CommandResult {
    mutate_board(app, state, |board| board.exec_move(&mv))?;
    return Ok(());
}

#[tauri::command]
fn undo(app: AppHandle, state: State<BoardState>) -> CommandResult {
    mutate_board(app, state, |board| board.undo_move())?;
    return Ok(());
}

#[tauri::command]
fn apply_fen(fen: &str, app: AppHandle, state: State<BoardState>) -> CommandResult {
    mutate_board(app, state, |board| board.apply_fen(fen))?;
    return Ok(());
}

fn main() {
    let board = Board::new_game();
    let state = BoardState { board: Arc::new(Mutex::new(board)) };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_board_cmd, get_available_moves, exec_move, undo, apply_fen])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
