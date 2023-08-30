import { useState } from "react"
import { applyFen, undo } from "../commands";

export function FenInput() {
    const [fen, setFen] = useState('');

    async function handleApplyClick() {
        await applyFen(fen);
    }

    async function handleUndoClick() {
        await undo();
    }

    return (
        <div className="fen">
            <input type="text" value={fen} onChange={(e) => setFen(e.target.value)} />
            <button onClick={() => handleApplyClick()}>Apply</button>
            <button onClick={() => handleUndoClick()}>Undo</button>
        </div>
    )
}
