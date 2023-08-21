const A = 97;

export type Coord = string;

export type Piece =
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

export interface Move {
    from: Coord;
    to: Coord;
    takes?: Piece;
}

export function toCoordFromOffset(offset: number): Coord {
    const column = String.fromCharCode(A + Math.floor(offset / 8));
    const row = offset % 8 + 1;

    return `${column}${row}`;
}

export function toCoordFromXY(x: number, y: number): Coord {
    const column = String.fromCharCode(A + x);
    const row = y + 1;

    return `${column}${row}`;
}

export function toXYFromCoord(coord: Coord): [number, number] {
    const column = coord[0].charCodeAt(0) - A;
    const row = parseInt(coord[1], 10) - 1;

    return [column, row];
}

export function toOffset(coord: Coord): number {
    const column = coord[0].charCodeAt(0) - A;
    const row = parseInt(coord[1], 10) - 1;

    return column * 8 + row;
}
