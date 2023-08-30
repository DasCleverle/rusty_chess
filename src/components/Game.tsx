import { useEffect, useState } from "react";
import { Color, Coord, Move, Piece, toCoordFromXY } from "../chess";
import { BoardPayload, executeMove, getAvailableMoves, getBoard } from "../commands";
import { Square } from "./Square";
import { listen } from "@tauri-apps/api/event";
import swal from 'sweetalert2';

interface Row {
    row: number,
    pieces: Piece[];
}

function transformRows(allPieces: Piece[]): Row[] {
    const rows = [];

    for (let r = 0; r < 8; r++) {
        let pieces = new Array(8);

        for (let c = 0; c < 8; c++) {
            const coord = toCoordFromXY(c, r);
            const piece = allPieces.find(p => p.coord == coord);

            if (piece) {
                pieces[c] = piece;
            }
            else {
                pieces[c] = { coord };
            }
        }

        rows.push({
            row: r,
            pieces
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
    const [winner, setWinner] = useState<string | undefined>();

    useEffect(() => {
        function setState(payload: BoardPayload) {
            setRows(transformRows(payload.pieces));
            setTurn(payload.turn);
            setWhiteChecked(payload.whiteChecked);
            setBlackChecked(payload.blackChecked);
            setWinner(payload.winner);
        }

        async function init() {
            setState(await getBoard());

            await listen<BoardPayload>('update', function({ payload }) {
                setState(payload);
            })
        }

        init();
    }, []);

    useEffect(() => {
        if (!winner) {
            return;
        }

        swal.fire(`${winner} has won!`);
    }, [winner]);

    const handleSquareClick = async (piece: Piece) => {
        if (selected) {
            if (selected == piece.coord) {
                setSelected(null);
                setMoves([]);
                return;
            }

            const move = moves.find(m => m.to == piece.coord);

            if (move) {
                await executeMove(move);

                setSelected(null);
                setMoves([]);
                return;
            }
        }

        const availableMoves = await getAvailableMoves(piece.coord);

        if (availableMoves.length === 0) {
            setSelected(null);
            setMoves([]);
            return;
        }

        setSelected(piece.coord);
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

                        {row.pieces.map(piece =>
                            <Square
                                key={piece.coord}
                                coord={piece.coord}
                                pieceType={piece.pieceType}
                                color={piece.color}
                                isSelected={selected === piece.coord}
                                isTarget={moves.some(m => m.to === piece.coord)}
                                onClick={() => handleSquareClick(piece)}
                            />
                        )}
                    </div>
                ))}
            </div>
        </div>
    );
}
