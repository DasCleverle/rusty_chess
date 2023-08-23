import { invoke } from "@tauri-apps/api/tauri";
import { Color, Coord, Move, Piece } from "./chess";

export interface BoardPayload {
    turn: Color;
    pieces: Piece[];
    whiteChecked: boolean;
    blackChecked: boolean;
}

export async function getBoard(): Promise<BoardPayload> {
    return await invoke<BoardPayload>('get_board_cmd');
}

export async function getAvailableMoves(coord: Coord) {
    return await invoke<Move[]>('get_available_moves', { coord });
}

export async function executeMove(move: Move) {
    return await invoke<Move[]>('exec_move', { mv: move });
}

export async function applyFen(fen: string) {
    return await invoke<Move[]>('apply_fen', { fen });
}
