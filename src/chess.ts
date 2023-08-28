export type Color = 'Black' | 'White';

export type Coord = string;

export type PieceType =
    | 'Pawn'
    | 'Rook'
    | 'Knight'
    | 'Bishop'
    | 'Queen'
    | 'King'

export interface Piece {
    coord: Coord,
    pieceType?: PieceType,
    color?: Color
}

export interface Move {
    from: Coord;
    to: Coord;
}

const A = 97;

export function toCoordFromXY(x: number, y: number): Coord {
    const column = String.fromCharCode(A + x);
    const row = y + 1;

    return `${column}${row}`;
}
