import { invoke } from "@tauri-apps/api/tauri";
import { Color, Coord, Move, Piece } from "./chess";

interface BoardPayload {
    turn: Color;
    pieces: Piece[];
}

export async function getBoard(): Promise<BoardPayload> {
    return await invoke<BoardPayload>('get_board');
}

export async function getAvailableMoves(coord: Coord) {
    return await invoke<Move[]>('get_available_moves', { coord });
}

export async function executeMove(move: Move) {
    return await invoke<Move[]>('exec_move', { mv: move });
}
