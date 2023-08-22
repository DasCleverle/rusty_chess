import { useEffect, useState } from "react";
import { executeMove, getAvailableMoves, getBoard } from "./commands";
import { Color, Coord, Move, Piece, toCoordFromXY, toOffset } from "./chess";

interface BoardState {
    rows: Row[];
    selected?: Coord;
    moves: Move[];
    turn: Color;
}

interface Row {
    row: number,
    squares: Square[];
}

interface Square {
    coord: Coord,
    piece?: Piece,
}

function getImageName(piece: Piece) {
    let str = '';

    for (let i = 0; i < piece.length; i++) {
        const c = piece.charAt(i);

        if (i !== 0 && c === c.toUpperCase()) {
            str += '_' + c.toLowerCase();
        }
        else {
            str += c;
        }
    }

    return str;
}

async function updateBoard(): Promise<{ rows: Row[], turn: Color }> {
    const payload = await getBoard();
    const rows = [];

    for (let r = 0; r < 8; r++) {
        let squares = [];

        for (let c = 0; c < 8; c++) {
            const coord = toCoordFromXY(c, r);
            const offset = toOffset(coord);

            squares.push({
                coord,
                piece: payload.pieces[offset],
            } satisfies Square);
        }

        rows.push({
            row: r,
            squares
        });
    }

    rows.sort((a, b) => b.row - a.row);

    return { rows, turn: payload.turn };
};

interface SquareProps {
    isTarget: boolean;
    isSelected: boolean;
    coord: Coord;
    piece?: Piece;
    onClick: () => void;
}

function Square({ isTarget, isSelected, piece, onClick }: SquareProps) {
    const classes = ['square'];

    if (isSelected) {
        classes.push('selected');
    }

    if (isTarget) {
        classes.push('target');
    }

    if (piece) {
        classes.push('occupied');
    }

    return (
        <div className={classes.join(' ')} onClick={onClick}>
            <div>
                {piece ? <img src={`/pieces/${getImageName(piece)}.png`} /> : null}
            </div>
        </div>
    );
}

export function App() {
    const [board, setBoard] = useState<BoardState>({ rows: [], moves: [], turn: 'White' });

    const update = () => updateBoard().then(update => setBoard(s => ({ ...s, ...update })));

    useEffect(() => { update(); }, []);

    const handleSquareClick = async (square: Square) => {
        if (board.selected) {
            const move = board.moves.find(m => m.to == square.coord);

            if (move) {
                await executeMove(move);
                await update();

                setBoard(s => ({
                    ...s,
                    selected: undefined,
                    moves: []
                }));
                return;
            }
        }

        const moves = await getAvailableMoves(square.coord);

        if (board.selected === square.coord || moves.length === 0) {
            setBoard(s => ({
                ...s,
                selected: undefined,
                moves: []
            }));
            return;
        }

        setBoard(s => ({
            ...s,
            selected: square.coord,
            moves
        }));
    }

    return (
        <div className="container">
            <div className="game-info">
                <div className="turn-indicator">
                    {board.turn == 'Black' ? <strong>Black's turn</strong> : null}
                </div>
                <div className="spacer"></div>
                <div className="turn-indicator">
                    {board.turn == 'White' ? <strong>White's turn</strong> : null}
                </div>
            </div>
            <div className="board">
                <div className="row labels">
                    <div className="rank-label"></div>
                    <div className="file-label">A</div>
                    <div className="file-label">B</div>
                    <div className="file-label">C</div>
                    <div className="file-label">D</div>
                    <div className="file-label">E</div>
                    <div className="file-label">F</div>
                    <div className="file-label">G</div>
                    <div className="file-label">H</div>
                </div>

                {board.rows.map(row => (
                    <div className="row" key={row.row}>
                        <div className="rank-label">{row.row + 1}</div>

                        {row.squares.map(square =>
                            <Square
                                key={square.coord}
                                coord={square.coord}
                                piece={square.piece}
                                isSelected={board.selected === square.coord}
                                isTarget={board.moves.some(m => m.to === square.coord)}
                                onClick={() => handleSquareClick(square)}
                            />
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
}
