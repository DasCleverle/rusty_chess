// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod chess;

use std::error::Error;

use serde::Serialize;
use tauri::{App, Wry, Manager, Window, PageLoadPayload};

use crate::chess::{Board, Piece};

#[derive(Serialize, Clone)]
struct BoardPayload {
    pieces: Vec<Option<Piece>>
}

impl From<Board> for BoardPayload {
    fn from(value: Board) -> Self {
        let pieces = Vec::from(value.pieces);
        return BoardPayload { pieces };
    }
}

fn init(app: &mut App<Wry>) -> Result<(), Box<dyn Error>> {
    return Ok(());
}

fn handle_page_load(window: Window<Wry>, _payload: PageLoadPayload) {
    let board: Board = Board::new_game();
    let payload: BoardPayload = board.into();

    println!("emitting board-update");
    window.emit_all("board-update", payload).expect("emit to be successful");
}

#[tauri::command]
fn get_board() -> BoardPayload {
    println!("getting board");
    let board: Board = Board::new_game();
    let payload: BoardPayload = board.into();

    return payload;
}

fn main() {
    tauri::Builder::default()
        .setup(init)
        .invoke_handler(tauri::generate_handler![get_board])
        .on_page_load(handle_page_load)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
