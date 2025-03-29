import { useEffect, useRef, useState } from "react";
import { useSelector } from "react-redux";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import LayerPaneItem from "./LayerPaneItem";
import L from "leaflet";
import TableView from "./TableView";

function Map() {
    let map = useRef<L.Map>(undefined);
    const [redrawing, setRedrawing] = useState(false);
    const [layersPaneVisible, setLayersPaneVisible] = useState(false);
    const [tableViewVisible, setTableViewVisible] = useState(false);
    const [filterToolVisible, setFilterToolVisible] = useState(false);
    const [filter, setFilter] = useState("");
    const vectorLayers = useSelector((state: any) => state.map.vectorLayers);
    const rasterLayers = useSelector((state: any) => state.map.rasterLayers);

    function toggleLayersPane() {
        setLayersPaneVisible(!layersPaneVisible);
    }

    function toggleFilterTool() {
        setFilterToolVisible(!filterToolVisible);
    }

    listen<string>('open-table', (_event) => {
        setTableViewVisible(true);
    });


    listen<string>('close-table', (_event) => {
        setTableViewVisible(false);
    });


    function redraw() {
        if (!map.current)
            return

        emit("loading", 75);

        const geomPromises = []

        if (Object.keys(vectorLayers).length === 0) {
            setRedrawing(false);
            emit("loading", 0);
            return;
        }

        map.current.eachLayer((layer) => {
            if (layer instanceof L.TileLayer) return;  // Keep the base tile layer
            map.current!.removeLayer(layer);
        });

        for (const lyr of Object.keys(vectorLayers)) {
            if (!vectorLayers[lyr].layer.visible)
                continue;

            geomPromises.push(invoke<string[]>("get_as_json_gpkg", {
                table: vectorLayers[lyr].layer.name,
                schema: vectorLayers[lyr].layer.schema,
            }).then((result) => {
                result.forEach((geom) => {
                    L.geoJson(JSON.parse(geom), {
                        style: vectorLayers[lyr].layer.symbology
                    }).addTo(map.current!);
                });
            }));
        }

        map.current.removeEventListener("click");
        map.current.on("click", (event) => {
            Object.keys(vectorLayers)
                .filter((lyr) => vectorLayers[lyr].layer.visible)
                .forEach((lyr) => {
                    (document.getElementById("repl-input") as HTMLTextAreaElement)!.value = `inspect ${lyr} \`${event.latlng.lng}, ${event.latlng.lat}\``;
                    (document.getElementById("repl-form") as HTMLFormElement)!.requestSubmit();
                });
        }, { once: true });

        Promise.all(geomPromises).then(() => {
            setRedrawing(false);
            emit("loading", 0);
        });
    }

    useEffect(() => {
        if (redrawing) {
            const timeoutId = setTimeout(() => {
                redraw();
            }, 100); // 100ms debounce
            return () => clearTimeout(timeoutId);
        }
    }, [redrawing]);

    useEffect(() => {
        if (!map.current) {
            map.current = L.map("map", {
                renderer: new L.Canvas(),
                fadeAnimation: false,
                zoomAnimation: true,
                zoomSnap: 0.85
            });

            map.current!.setView([0, 0], 2);

            L.tileLayer('https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}{r}.png', {
                attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors &copy; <a href="https://carto.com/attributions">CARTO</a>',
                subdomains: 'abcd',
                maxZoom: 20,
                detectRetina: false
            }).addTo(map.current!);

            setRedrawing(true);
        }

        setRedrawing(true);
    }, [vectorLayers, rasterLayers]);

    return (
        <>
            <div
                id="toggle-layers-btn"
                className="ml-2 z-[998] absolute rounded-full btn bg-slate-800 text-center border-solid border-slate-300 border-2"
                style={{
                    bottom: 80,
                }}
                onClick={() => toggleLayersPane()}
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" fill="currentColor" className="bi bi-layers text-white p-4" viewBox="3 3 10 10">
                    <path d="M8.235 1.559a.5.5 0 0 0-.47 0l-7.5 4a.5.5 0 0 0 0 .882L3.188 8 .264 9.559a.5.5 0 0 0 0 .882l7.5 4a.5.5 0 0 0 .47 0l7.5-4a.5.5 0 0 0 0-.882L12.813 8l2.922-1.559a.5.5 0 0 0 0-.882zm3.515 7.008L14.438 10 8 13.433 1.562 10 4.25 8.567l3.515 1.874a.5.5 0 0 0 .47 0zM8 9.433 1.562 6 8 2.567 14.438 6z" />
                </svg>
            </div>
            <div id="layers-pane" className="z-[995] rounded-md absolute text-white w-1/3 h-100 bg-slate-800 overflow-auto border border-2 border-solid border-slate-300" style={{
                bottom: 125,
                marginLeft: 20,
                visibility: layersPaneVisible ? "visible" : "hidden"
            }}>
                <div className="grid grid-rows-1 grid-cols-1">
                    <div className="grid grid-cols-[93%_7%] h-10 border-solid border-b-2 border-slate-300">
                        <h1 className="text-xl mb-2 p-2">Layers</h1>
                        <div className="btn rounded-md mt-3 mr-2 mb-4 hover:bg-slate-300" onClick={() => toggleFilterTool()}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" fill="currentColor" className="ml-[4px] mt-[3px] bi bi-filter" viewBox="0 0 16 16">
                                <path d="M6 10.5a.5.5 0 0 1 .5-.5h3a.5.5 0 0 1 0 1h-3a.5.5 0 0 1-.5-.5m-2-3a.5.5 0 0 1 .5-.5h7a.5.5 0 0 1 0 1h-7a.5.5 0 0 1-.5-.5m-2-3a.5.5 0 0 1 .5-.5h11a.5.5 0 0 1 0 1h-11a.5.5 0 0 1-.5-.5" />
                            </svg>
                        </div>
                    </div>
                    {filterToolVisible ? (
                        <div className="h-10 w-full">
                            <input className="pl-2 w-full border-solid border-1 border-slate-300 bg-slate-950" type="text" placeholder="Filter by name or schema..." onChange={(event) => {
                                setFilter(event.target.value);
                            }} />
                        </div>
                    ) : (<></>)}
                </div>
                <table className="w-full">
                    <tbody>
                        {Object.keys(vectorLayers).map((lyr: string) => {
                            if (filter === "" || vectorLayers[lyr].layer.name.includes(filter) || vectorLayers[lyr].layer.schema.includes(filter)) {
                                return (
                                    <LayerPaneItem item={vectorLayers[lyr]} />
                                );
                            }
                        })}
                    </tbody>
                </table>
            </div>
            <TableView visible={tableViewVisible} />
            <div id="map" style={{
                userSelect: "none",
                WebkitUserSelect: "none"
            }} className="position-absolute mt-0 h-[91.5vh] w-full"></div>
        </>
    );
}

export default Map;
