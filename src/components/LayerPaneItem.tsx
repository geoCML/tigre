import { VectorLayer, RasterLayer } from "../types/Layer.type";
import { useDispatch, useSelector } from "react-redux";
import { useState, useRef } from "react";
import { toggleVectorLayerVisibility } from "../map.slice";


type LayerPaneItemProps = {
    item: { layer: VectorLayer | RasterLayer }
}

function LayerPaneItem(props: LayerPaneItemProps) {
    const dispatch = useDispatch();
    const vectorLayers = useSelector((state: any) => state.map.vectorLayers);
    const [contextMenuVisible, setContentMenuVisible] = useState(false);
    const bufferDistanceInput = useRef(null);
    const intersectLayerInput = useRef(null);


    function toggleContextMenu() {
        setContentMenuVisible(!contextMenuVisible);
    }

    return (
        <tr className="grid grid-rows-1 grid-cols-[85%_15%] border-solid border-1 border-stone-200">
            <div>
                <input
                    className="ml-4"
                    type="checkbox"
                    id={props.item.layer.name}
                    value=""
                    checked={props.item.layer.visible}
                    onChange={() => {
                        dispatch(toggleVectorLayerVisibility(props.item.layer.name));
                    }}
                />
                <label
                    htmlFor={props.item.layer.name}
                    className="pl-2"
                >
                    <span className="text-xs">{props.item.layer.schema}.</span>{props.item.layer.name}
                </label>
            </div>

            <div className="grid grid-rows-1 grid-cols-2">
                <div className="btn border-solid border-2 border-stone-200 rounded-md w-3/4 hover:bg-stone-300" onClick={() => toggleContextMenu()}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="ml-[6px] mt-[2px] bi bi-hammer" viewBox="0 0 16 16">
                        <path d="M9.972 2.508a.5.5 0 0 0-.16-.556l-.178-.129a5 5 0 0 0-2.076-.783C6.215.862 4.504 1.229 2.84 3.133H1.786a.5.5 0 0 0-.354.147L.146 4.567a.5.5 0 0 0 0 .706l2.571 2.579a.5.5 0 0 0 .708 0l1.286-1.29a.5.5 0 0 0 .146-.353V5.57l8.387 8.873A.5.5 0 0 0 14 14.5l1.5-1.5a.5.5 0 0 0 .017-.689l-9.129-8.63c.747-.456 1.772-.839 3.112-.839a.5.5 0 0 0 .472-.334"/>
                    </svg>
                </div>
                <div className="btn border-solid border-2 border-stone-200 rounded-md w-3/4 hover:bg-stone-300" onClick={() => toggleContextMenu()}>
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="ml-[6px] mt-[2px] bi bi-table" viewBox="0 0 16 16">
                        <path d="M0 2a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H2a2 2 0 0 1-2-2zm15 2h-4v3h4zm0 4h-4v3h4zm0 4h-4v3h3a1 1 0 0 0 1-1zm-5 3v-3H6v3zm-5 0v-3H1v2a1 1 0 0 0 1 1zm-4-4h4V8H1zm0-4h4V4H1zm5-3v3h4V4zm4 4H6v3h4z"/>
                    </svg>
                </div>
            </div>

                { contextMenuVisible ? (
                    <table className="absolute border-solid border-2 border-stone-200 bg-white w-3/4 left-1/4 mt-[20px]">
                        <tbody>
                            <tr className="grid grid-cols-[10%_15%_75%] grid-rows-1 text-xs border-solid border-1 border-stone-200">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-arrows-expand-vertical mt-1" viewBox="0 0 16 16">
                                    <path d="M8 15a.5.5 0 0 1-.5-.5v-13a.5.5 0 0 1 1 0v13a.5.5 0 0 1-.5.5M.146 8.354a.5.5 0 0 1 0-.708l2-2a.5.5 0 1 1 .708.708L1.707 7.5H5.5a.5.5 0 0 1 0 1H1.707l1.147 1.146a.5.5 0 0 1-.708.708zM10 8a.5.5 0 0 1 .5-.5h3.793l-1.147-1.146a.5.5 0 0 1 .708-.708l2 2a.5.5 0 0 1 0 .708l-2 2a.5.5 0 0 1-.708-.708L14.293 8.5H10.5A.5.5 0 0 1 10 8"/>
                                </svg>
                                <p className="mt-1">Buffer</p>
                                <div className="mt-[3px] pl-2">
                                    <input ref={bufferDistanceInput} className="w-3/4" type="text" placeholder="distance"/>
                                    <input className="w-1/4 btn bg-blue-600 text-white hover:bg-blue-800" type="submit" value="Run" onClick={ () => {
                                        (document.getElementById("repl-input") as HTMLTextAreaElement)!.value = `buffer ${props.item.layer.schema}.${props.item.layer.name} ${(bufferDistanceInput.current! as HTMLInputElement).value}`;
                                        (document.getElementById("repl-form") as HTMLFormElement)!.requestSubmit();
                                    } }/>
                                </div>
                            </tr>

                            <tr className="grid grid-cols-[10%_15%_75%] grid-rows-1 text-xs border-solid border-1 border-stone-200">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-intersect mt-1" viewBox="0 0 16 16">
                                    <path d="M0 2a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v2h2a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-2H2a2 2 0 0 1-2-2zm5 10v2a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1V6a1 1 0 0 0-1-1h-2v5a2 2 0 0 1-2 2zm6-8V2a1 1 0 0 0-1-1H2a1 1 0 0 0-1 1v8a1 1 0 0 0 1 1h2V6a2 2 0 0 1 2-2z"/>
                                </svg>
                                <p className="mt-1">Intersect</p>
                                <div className="mt-[3px] pl-2">
                                    <select ref={intersectLayerInput} className="w-3/4">
                                        {
                                            Object.keys(vectorLayers).map((lyr: string) => {
                                                return (<option value={`${vectorLayers[lyr].layer.schema}.${lyr}`}>{vectorLayers[lyr].layer.schema}.{lyr}</option>)
                                            })
                                        }
                                    </select>
                                    <input className="w-1/4 btn bg-blue-600 text-white hover:bg-blue-800" type="submit" value="Run" onClick={ () => {
                                        (document.getElementById("repl-input") as HTMLTextAreaElement)!.value = `intersect ${props.item.layer.schema}.${props.item.layer.name} ${(intersectLayerInput.current! as HTMLSelectElement).value}`;
                                        (document.getElementById("repl-form") as HTMLFormElement)!.requestSubmit();
                                    } }/>
                                </div>
                            </tr>

                        </tbody>
                    </table>
                ) : ( <></> )}

        </tr>
    )
}

export default LayerPaneItem;
