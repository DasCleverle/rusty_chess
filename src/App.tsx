import { useEffect, useState } from "react";
import { executeMove, getAvailableMoves, getBoard } from "./commands";
import { Coord, Piece, toCoordFromXY, toOffset, toXYFromCoord } from "./chess";

interface BoardState {
    rows: Row[]
    selected?: Coord;
    targets: Coord[]
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

async function updateBoard(): Promise<Row[]> {
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

    return rows;
};

export function App() {
    const [board, setBoard] = useState<BoardState>({ rows: [], targets: [] });

    const update = () => updateBoard().then(rows => setBoard(s => ({ ...s, rows })));

    useEffect(() => { update(); }, []);

    const handleSquareClick = async (square: Square) => {
        if (board.selected && board.targets.includes(square.coord)) {
            await executeMove({
                from: board.selected,
                to: square.coord,
            });
            await update();

            setBoard(s => ({
                ...s,
                selected: undefined,
                targets: []
            }));

            return;
        }

        const moves = await getAvailableMoves(square.coord);

        if (board.selected === square.coord || moves.length === 0) {
            setBoard(s => ({
                ...s,
                selected: undefined,
                targets: []
            }));
            return;
        }

        setBoard(s => ({
            ...s,
            selected: square.coord,
            targets: moves.map(m => m.to)
        }));
    }

    return (
        <div className="container">
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
                    <div className="rank-label">{row.row}</div>

                    {row.squares.map(square => (
                        <div className={`square ${board.selected === square.coord ? 'from' : ''} ${board.targets.includes(square.coord) ? 'to' : ''}`} key={square.coord} onClick={() => handleSquareClick(square)}>
                            {square.piece ? <img src={`/pieces/${getImageName(square.piece)}.png`} /> : null}
                        </div>
                    ))}
                </div>
            ))}
        </div>
    );
}
