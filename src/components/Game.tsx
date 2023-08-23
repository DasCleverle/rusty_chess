import { useEffect, useState } from "react";
import { Color, Coord, Move, Piece, toCoordFromXY, toOffset } from "../chess";
import { BoardPayload, executeMove, getAvailableMoves, getBoard } from "../commands";
import { Square } from "./Square";
import { listen } from "@tauri-apps/api/event";

interface Row {
    row: number,
    squares: Square[];
}

interface Square {
    coord: Coord,
    piece?: Piece,
}

function transformRows(pieces: Piece[]) {
    const rows = [];

    for (let r = 0; r < 8; r++) {
        let squares = [];

        for (let c = 0; c < 8; c++) {
            const coord = toCoordFromXY(c, r);
            const offset = toOffset(coord);

            squares.push({
                coord,
                piece: pieces[offset],
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

export function Game() {
    const [rows, setRows] = useState<Row[]>([]);
    const [selected, setSelected] = useState<Coord | null>(null);
    const [moves, setMoves] = useState<Move[]>([]);
    const [turn, setTurn] = useState<Color>('White');
    const [whiteChecked, setWhiteChecked] = useState<boolean>(false);
    const [blackChecked, setBlackChecked] = useState<boolean>(false);

    useEffect(() => {
        function setState(payload: BoardPayload) {
            setRows(transformRows(payload.pieces));
            setTurn(payload.turn);
            setWhiteChecked(payload.whiteChecked);
            setBlackChecked(payload.blackChecked);
        }

        async function init() {
            setState(await getBoard());

            await listen<BoardPayload>('update', function({ payload }) {
                setState(payload);
            })
        }

        init();
    }, []);

    const handleSquareClick = async (square: Square) => {
        if (selected) {
            if (selected == square.coord) {
                setSelected(null);
                setMoves([]);
                return;
            }

            const move = moves.find(m => m.to == square.coord);

            if (move) {
                await executeMove(move);

                setSelected(null);
                setMoves([]);
                return;
            }
        }

        const availableMoves = await getAvailableMoves(square.coord);

        if (availableMoves.length === 0) {
            setSelected(null);
            setMoves([]);
            return;
        }

        setSelected(square.coord);
        setMoves(availableMoves);
    }

    return (
        <div className="game">
            <div className="game-info">
                <div className="turn-indicator">
                    {turn == 'Black' ? <div><strong>Black's turn</strong></div> : null}
                    {blackChecked ? <div><strong>Check!</strong></div> : null}
                </div>
                <div className="turn-indicator">
                    {turn == 'White' ? <div><strong>White's turn</strong></div> : null}
                    {whiteChecked ? <div><strong>Check!</strong></div> : null}
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

                {rows.map(row => (
                    <div className="row" key={row.row}>
                        <div className="rank-label">{row.row + 1}</div>

                        {row.squares.map(square =>
                            <Square
                                key={square.coord}
                                coord={square.coord}
                                piece={square.piece}
                                isSelected={selected === square.coord}
                                isTarget={moves.some(m => m.to === square.coord)}
                                onClick={() => handleSquareClick(square)}
                            />
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
}
