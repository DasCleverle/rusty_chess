// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use] extern crate serde;

mod chess;

use std::sync::{Arc, Mutex};

use chess::{Board, Coord, Move, Piece};
use serde::Serialize;
use tauri::State;

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Could not aquire lock")]
    LockErr,

    #[error("{0}")]
    Err(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Serialize, Clone)]
struct BoardPayload {
    pieces: Vec<Option<Piece>>,
}

struct BoardState {
    board: Arc<Mutex<Board>>,
}

impl From<&Board> for BoardPayload {
    fn from(value: &Board) -> Self {
        let pieces = Vec::from(value.pieces());
        return BoardPayload { pieces };
    }
}

#[tauri::command]
fn get_board(state: State<BoardState>) -> Result<BoardPayload, AppError> {
    return match state.inner().board.lock() {
        Ok(board) => Ok((&*board).into()),
        Err(_) => Err(AppError::LockErr),
    };
}

#[tauri::command]
fn get_available_moves(coord: Coord, state: State<BoardState>) -> Result<Vec<Move>, AppError> {
    return match state.inner().board.lock() {
        Ok(board) => match (*board).get_available_moves(coord) {
            Some(moves) => Ok(moves),
            None => Ok(Vec::new()),
        },
        Err(_) => Err(AppError::LockErr),
    };
}

#[tauri::command]
fn exec_move(mv: Move, state: State<BoardState>) -> Result<(), AppError> {
    return match state.inner().board.lock() {
        Ok(mut board) => {
            match board.exec_move(mv) {
                Ok(_) => Ok(()),
                Err(err) => Err(AppError::Err(err)),
            }
        }
        Err(_) => Err(AppError::LockErr),
    };
}

fn main() {
    let board = Board::new_game();
    let state = BoardState {
        board: Arc::new(Mutex::new(board)),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_board, get_available_moves, exec_move])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
