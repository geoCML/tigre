import "./App.css";
import { useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Output } from "./types/Output.type";
import { addVectorLayer } from "./map.slice";
import { VectorLayer } from "./types/Layer.type";
import { useDispatch } from "react-redux";
import Map from "./components/Map";
import ControlBar from "./components/ControlBar";
import REPLHistoryItem from "./components/REPLHistoryItem";
import LoadingBar from "./components/LoadingBar";

function App() {
    const dispatch = useDispatch();
    const [history, setHistory] = useState([
        <REPLHistoryItem cmd={"Welcome to TIGRE!"} 
            output={{
                errors: [],
                results: ["Learn more about TIGRE here: https://geocml.github.io/docs/"]
            }} forceMessage={true} />
    ])
    let replInput = useRef<HTMLTextAreaElement>(null);
    let replForm = useRef<HTMLFormElement>(null);

    return (
      <main>
        <div className="w-full h-[90vh] grid grid-cols-[20%_80%] grid-rows-1">
            <div className="bg-stone-950 text-white overflow-y-auto">
                { history.map((elem) => elem) }
            </div>
            <div className="grid grid-cols-1 grid-rows-[3vh_86vh]">
                <ControlBar />
                <Map />
            </div>
        </div>
        <form id="repl-form" ref={replForm} onSubmit={async (event) => {
            event.preventDefault();
            const cmd = replInput.current!.value.replaceAll("\n", "");
            if (cmd === "clear") {
                setHistory([]);
            } else {
                const readResult = await invoke<Record<string, string[]>>("read", { cmd });
                invoke<string>("eval", { ast: readResult }).then((result) => {
                    listen<string>('add-vector-layer', (event) => {
                        const layer: VectorLayer = {
                            name: event.payload,
                            visible: true
                        }
                        dispatch(addVectorLayer({
                            layer
                        }));
                    });

                    const evalResult = JSON.parse(result) as Output;
                    setHistory([...history, <REPLHistoryItem cmd={cmd} output={evalResult} forceMessage={false}/>]);
                });
            }

            replInput.current!.value = "";
            replInput.current!.innerText = "";
        }}>
        <textarea id="repl-input" autoComplete="off" placeholder="Press Enter/Return to Execute REPL Command" autoFocus onKeyDown={(event) => {
            if (event && event.key === "Enter")
                replForm.current!.requestSubmit();
        }} ref={replInput} className="border-solid border-2 border-stone-800 w-full h-[5vh] bg-stone-300 text-black resize-none"></textarea>
            <input type="submit" style={{ display: "none" }} />
        </form>
        <LoadingBar />
      </main>
    )
}

export default App;
