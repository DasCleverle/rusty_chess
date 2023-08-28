import { Color, Coord, PieceType } from "../chess";

interface SquareProps {
    isTarget: boolean;
    isSelected: boolean;
    coord: Coord;
    pieceType?: PieceType;
    color?: Color;
    onClick: () => void;
}

function getImageName(color: Color, piece: PieceType) {
    return `${color.toLowerCase()}_${piece.toLowerCase()}`;
}

export function Square({ isTarget, isSelected, pieceType, color, onClick }: SquareProps) {
    const classes = ['square'];

    if (isSelected) {
        classes.push('selected');
    }

    if (isTarget) {
        classes.push('target');
    }

    if (pieceType || color) {
        classes.push('occupied');
    }

    return (
        <div className={classes.join(' ')} onClick={onClick}>
            <div>
                {pieceType && color ? <img src={`/pieces/${getImageName(color, pieceType)}.png`} /> : null}
            </div>
        </div>
    );
}
