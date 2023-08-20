// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod chess;

use chess::{Board, Coord, Move, Piece};
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Clone)]
struct BoardPayload {
    pieces: Vec<Option<Piece>>,
}

struct BoardState {
    board: Board,
}

impl From<Board> for BoardPayload {
    fn from(value: Board) -> Self {
        let pieces = Vec::from(value.pieces());
        return BoardPayload { pieces };
    }
}

#[tauri::command]
fn get_board(state: State<BoardState>) -> BoardPayload {
    return state.inner().board.into();
}

#[tauri::command]
fn get_available_moves(coord: Coord, state: State<BoardState>) -> Vec<Move> {
    match state.inner().board.get_available_moves(coord) {
        Some(moves) => moves,
        None => Vec::new(),
    }
}

fn main() {
    let board = Board::new_game();
    let state = BoardState { board };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_board, get_available_moves])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
