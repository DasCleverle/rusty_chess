import { FenInput } from "./components/FenInput";
import { Game } from "./components/Game";

export function App() {
    return (
        <div className="container">
            <Game />
            <FenInput />
        </div>
    );
}
