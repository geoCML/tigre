import { useState, useRef } from "react";
import { listen, emit } from "@tauri-apps/api/event";

type TableViewTab = {
    name: string,
    data: [{}]
}

type TableViewProps = {
    visible: boolean
}

function TableView({ visible }: TableViewProps) {
    const table = useRef<HTMLTableElement>(null);
    const [tabs, setTabs] = useState<TableViewTab[]>([]);
    const [x, setX] = useState(0);
    const [y, setY] = useState(100);
    const [drag, setDrag] = useState(false);

    listen<string>('open-table', (event) => {
        const json = JSON.parse(JSON.parse(event.payload[1])); // event.payload is a double stringified JSON object

        if (json["json_agg"]) {
            setTabs([{
                name: event.payload[0],
                data: json["json_agg"]
            }]);
        }
    });

    return (
        <div
            className="z-[995] overflow-auto absolute bg-white border-solid border-stone-200 border-2 rounded-md text-black w-1/2 h-[25vh]"
            style={{
                transform: `translate(${x}px, ${y}px)`,
                display: visible ? "block" : "none"
            }}
        >
            <div
                className="sticky bg-stone-300 top-0 left-0"
                onMouseMove={(event) => {
                    if (drag) {
                        setX(x + event.movementX);
                        setY(y + event.movementY);
                    }
                }}

                onMouseDown={() => {
                    setDrag(true);
                }}

                onMouseUp={() => {
                    setDrag(false);
                }}

                onMouseLeave={() => {
                    setDrag(false);
                }}
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-x-lg ml-[97%] h-[32px]" viewBox="0 0 16 16" onClick={() => { emit("close-table"); setTabs([]); }}>
                    <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z" />
                </svg>
            </div>

            <div className="grid grid-rows-1 auto-cols-max w-full">
                {tabs.map((tab) => {
                    return (
                        <div className="grid grid-rows-1 grid-cols-2">
                            <h2 className="bg-blue-200 p-1">{tab.name}</h2>
                            <div className="p-1 text-xs pt-2 italic">Showing {tab.data.length} rows of data.</div>
                        </div>
                    )
                })}
            </div>

            {tabs.map((tab) => {
                return (
                    <table
                        ref={table}
                        className="w-full"
                    >
                        <thead>
                            {Object.keys(tab.data[0])
                                .filter((col) => col != "geom")
                                .map((col) => {
                                    return (<th className={`p-2 w-[${100 / Object.keys(tab.data[0]).length - 1}%] border-solid border-blue-200 border-1 bg-blue-200`}>{col}</th>)
                                })}
                        </thead>
                        <tbody>
                            {tab.data.map((row) => {
                                    return (
                                        <tr>
                                            {Object.keys(tab.data[0])
                                                .filter((col) => col != "geom")
                                                .map((col) => {
                                                    return (
                                                        <td className="border-solid border-1 border-blue-200 overflow-scroll text-center">
                                                            {JSON.stringify((row as Record<string, unknown>)[col])}
                                                        </td>
                                                    )
                                                })}
                                        </tr>
                                    )
                            })}
                        </tbody>
                    </table>
                )
            })}
        </div>
    )
}

export default TableView;
