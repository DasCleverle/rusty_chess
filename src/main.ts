import type { Event } from '@tauri-apps/api/event';
import { listen } from "@tauri-apps/api/event";
import { invoke } from '@tauri-apps/api/tauri';

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

function updateBoard(board: BoardPayload) {
    const pieces = board.pieces;

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


// await listen('board-update', (event: Event<BoardPayload>) => {
//     updateBoard(event.payload);
//});

document.addEventListener('DOMContentLoaded', async () => {
    console.log('getting board');
    const board: BoardPayload = await invoke('get_board');
    updateBoard(board);
});
