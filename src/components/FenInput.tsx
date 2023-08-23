import { useState } from "react"
import { applyFen } from "../commands";

export function FenInput() {
    const [fen, setFen] = useState('');

    async function handleApplyClick() {
        await applyFen(fen);
    }

    return (
        <div className="fen">
            <input type="text" value={fen} onChange={(e) => setFen(e.target.value)} />
            <button onClick={() => handleApplyClick()}>Apply</button>
        </div>
    )
}
