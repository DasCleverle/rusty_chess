import { Coord, Piece } from "../chess";

interface SquareProps {
    isTarget: boolean;
    isSelected: boolean;
    coord: Coord;
    piece?: Piece;
    onClick: () => void;
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

export function Square({ isTarget, isSelected, piece, onClick }: SquareProps) {
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
