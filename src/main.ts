import type { Event } from '@tauri-apps/api/event';
import { listen } from "@tauri-apps/api/event";
import { invoke } from '@tauri-apps/api/tauri';

type Coord = string;

type Piece =
    | 'BlackRook'
    | 'BlackKnight'
    | 'BlackBishop'
    | 'BlackQueen'
    | 'BlackKing'
    | 'BlackPawn'
    | 'WhiteRook'
    | 'WhiteKnight'
    | 'WhiteBishop'
    | 'WhiteQueen'
    | 'WhiteKing'
    | 'WhitePawn';

interface BoardPayload {
    pieces: Piece[];
}

interface Move {
    from: Coord;
    to: Coord;
    takes?: Piece;
}

class State {
    isMoving: boolean = false;
    coord?: string;
    moves?: Move[];

    set(coord: string, moves: Move[]) {
        this.coord = coord;
        this.moves = moves;
        this.isMoving = true;
    }

    reset() {
        this.isMoving = false;
        this.coord = undefined;
        this.moves = undefined;
    }
}

function toCoord(offset: number) {
    const rank = offset % 8 + 1;
    const file = String.fromCharCode(97 + Math.floor(offset / 8));

    return `${file}${rank}`;
}

function getImageName(piece: Piece) {
    let str = '';

    for (let i = 0; i < piece.length; i++) {
        const c = piece.charAt(i);

        if (i !== 0 && c == c.toUpperCase()) {
            str += '_' + c.toLowerCase();
        }
        else {
            str += c;
        }
    }

    return str;
}

async function updateBoard() {
    const pieces = (await getBoard()).pieces;

    for (let i = 0; i < pieces.length; i++) {
        const piece = pieces[i];
        const coord = toCoord(i);
        const el = document.getElementById(coord);

        if (!el) {
            continue;
        }

        if (piece) {
            el.innerHTML = `<img src="/pieces/${getImageName(piece)}.png" />`;
        }
        else {
            el.innerHTML = '';
        }
    }
}

async function getBoard(): Promise<BoardPayload> {
    return await invoke<BoardPayload>('get_board');
}

async function getAvailableMoves(coord: Coord) {
    return await invoke<Move[]>('get_available_moves', { coord });
}

async function executeMove(move: Move) {
    return await invoke<Move[]>('exec_move', { mv: move });
}

document.addEventListener('DOMContentLoaded', async () => {
    await updateBoard();

    let state: State = new State();

    const squares = document.getElementsByClassName('square');

    function resetHighlights() {
        for (let square of squares) {
            square.classList.remove('to');
            square.classList.remove('from');
        }
    }

    for (let square of squares) {
        const coord = square.id;

        square.addEventListener('click', async function() {
            let moves: Move[] | null = null;

            if (state.isMoving) {
                if (state.coord === coord) {
                    resetHighlights();
                    state.reset();
                    return;
                }
                else {
                    const move = state.moves?.find(m => m.to == coord);

                    if (move) {
                        await executeMove(move);
                        await updateBoard();
                        state.reset();
                        resetHighlights();

                        return;
                    }
                    else {
                        moves = await getAvailableMoves(coord);

                        if (moves.length === 0) {
                            return;
                        }
                    }
                }
            }

            resetHighlights();

            if (!moves) {
                moves = await getAvailableMoves(coord);
            }

            if (moves.length === 0) {
                return;
            }

            state.set(coord, moves);

            square.classList.add('from');

            for (let move of moves) {
                const to = squares.namedItem(move.to)!;
                to.classList.add('to');
            }
        })
    }
});

