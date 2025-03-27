import "./App.css";
import { useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Output } from "./types/Output.type";
import { addVectorLayer, removeAllVectorLayers } from "./map.slice";
import { VectorLayer } from "./types/Layer.type";
import { useDispatch } from "react-redux";
import Map from "./components/Map";
import ControlBar from "./components/ControlBar";
import REPLHistoryItem from "./components/REPLHistoryItem";
import LoadingBar from "./components/LoadingBar";
import { Symbology } from "./types/Symbology.type";

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

    listen<string>('add-vector-layer',  async (event) => {
        const symbology = await invoke<string>("get_layer_symbology", {
            table: event.payload[0],
            schema: event.payload[1],
        });

        const layer: VectorLayer = {
            name: event.payload[0],
            schema: event.payload[1],
            visible: true,
            symbology: JSON.parse(JSON.parse(symbology)) as Symbology
        }

        dispatch(addVectorLayer({
            layer
        }));
    });
   
    listen<string>('wipe-layers', (event) => {
        if (event.payload)
            dispatch(removeAllVectorLayers());
    });

    return (
      <main>
        <div className="w-full h-[95vh] grid grid-cols-[14%_86%] grid-rows-1">
            <div className="bg-slate-950 text-white overflow-y-auto">
                { history.map((elem) => elem) }
            </div>
            <div className="grid grid-cols-1 grid-rows-[3vh_95vh]">
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
        }} ref={replInput} className="font-mono border-solid border-t-2 border-slate-800 w-full h-[25px] pl-1 bg-slate-950 text-slate-500 resize-none text-sm focus:outline-none focus:border-2 focus:border-blue-500 rounded-md"></textarea>
            <input type="submit" style={{ display: "none" }} />
        </form>
        <LoadingBar />
      </main>
    )
}

export default App;
