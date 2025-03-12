import { useState, useRef } from "react";
import { listen, emit } from "@tauri-apps/api/event";

type TableViewTab = {
    name: string,
    data: [{}]
}

function TableView() {
    const table = useRef<HTMLTableElement>(null);
    const [tabs, setTabs] = useState<TableViewTab[]>([]);
    const [x, setX] = useState(0);
    const [y, setY] = useState(100);
    const [drag, setDrag] = useState(false);


    listen<string>('open-table', (event) => {
        const json = JSON.parse(JSON.parse(event.payload[1])) // event.payload is a double stringified JSON object
        if (json["json_agg"]) {
            setTabs([...tabs, {
                name: event.payload[0],
                data: json["json_agg"]
            }]);
        }
    });

    return (
        <div
            className="z-[995] absolute bg-white border-solid border-stone-200 border-2 rounded-md text-black w-1/2 h-[25vh]"
            style={{
                transform: `translate(${x}px, ${y}px)`
            }}
        >
            <div
                className="bg-stone-300"
                onMouseMove={ (event) => {
                    if (drag) {
                        setX(x + event.movementX);
                        setY(y + event.movementY);
                    }
                }}

                onMouseDown={ () => {
                    setDrag(true);
                }}

                onMouseUp={ () => {
                    setDrag(false);
                }}

                onMouseLeave={ () => {
                    setDrag(false);
                }}
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-x-lg ml-[97%] h-[32px]" viewBox="0 0 16 16" onClick={() => { emit("close-table"); setTabs([]); }}>
                    <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8z"/>
                </svg>
            </div>

            <div className={`grid grid-rows-1 auto-cols-max w-full`}>
                { tabs.map((tab) => {
                    return (<h2>{tab.name}</h2>)
                }) }
            </div>


            { tabs.map((tab) => {
                return (
                    <table
                        ref={table}
                        className="overflow-scroll w-full"
                    >
                        <thead>
                        { Object.keys(tab.data[0])
                            .filter((col) => col != "geom")
                            .map((col) => {
                                return (<th className={`w-[${100 / Object.keys(tab.data[0]).length - 1}%] border-solid border-blue-200 border-1 bg-blue-200`}>{col}</th>)
                            }) }
                        </thead>
                        <tbody>
                            <tr>
                            { Object.keys(tab.data[0])
                                .filter((col) => col != "geom") // TODO: filter this out in the PSQL query
                                .map((col) => {
                                    return tab.data.map((row) => {
                                        return (
                                            <td className="border-solid border-1 border-blue-200 overflow-scroll text-center">{JSON.stringify(row[col])}</td>
                                        )
                                    })
                              }) }
                            </tr>
                        </tbody>
                    </table>
                )
            }) }
            <div>
                
            </div>
        </div>
    )
}

export default TableView;
