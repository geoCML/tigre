import { VectorLayer, RasterLayer } from "../types/Layer.type";
import { useDispatch } from "react-redux";
import { useState, useRef } from "react";
import { toggleVectorLayerVisibility } from "../map.slice";


type LayerPaneItemProps = {
    item: { layer: VectorLayer | RasterLayer }
}

function LayerPaneItem(props: LayerPaneItemProps) {
    const dispatch = useDispatch();
    const [contextMenuVisible, setContentMenuVisible] = useState(false);
    const bufferDistanceInput = useRef(null);

    function toggleContextMenu() {
        setContentMenuVisible(!contextMenuVisible);
    }

    return (
        <tr className="grid grid-rows-1 grid-cols-[91%_9%] border-solid border-1 border-stone-200">
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

            <div className="btn border-solid border-2 border-stone-200 rounded-md w-3/4 hover:bg-stone-300" onClick={() => toggleContextMenu()}>
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-hammer" viewBox="0 0 16 16">
                    <path d="M9.972 2.508a.5.5 0 0 0-.16-.556l-.178-.129a5 5 0 0 0-2.076-.783C6.215.862 4.504 1.229 2.84 3.133H1.786a.5.5 0 0 0-.354.147L.146 4.567a.5.5 0 0 0 0 .706l2.571 2.579a.5.5 0 0 0 .708 0l1.286-1.29a.5.5 0 0 0 .146-.353V5.57l8.387 8.873A.5.5 0 0 0 14 14.5l1.5-1.5a.5.5 0 0 0 .017-.689l-9.129-8.63c.747-.456 1.772-.839 3.112-.839a.5.5 0 0 0 .472-.334"/>
                </svg>
            </div>

                { contextMenuVisible ? (
                    <table className="absolute left-1/4 border-solid border-2 border-stone-200 bg-white w-3/4 mt-[20px]">
                        <tbody>
                            <tr className="grid grid-cols-[10%_15%_75%] grid-rows-1 text-xs border-solid border-1 border-stone-200">
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" className="bi bi-arrows-expand-vertical mt-1" viewBox="0 0 16 16">
                                    <path d="M8 15a.5.5 0 0 1-.5-.5v-13a.5.5 0 0 1 1 0v13a.5.5 0 0 1-.5.5M.146 8.354a.5.5 0 0 1 0-.708l2-2a.5.5 0 1 1 .708.708L1.707 7.5H5.5a.5.5 0 0 1 0 1H1.707l1.147 1.146a.5.5 0 0 1-.708.708zM10 8a.5.5 0 0 1 .5-.5h3.793l-1.147-1.146a.5.5 0 0 1 .708-.708l2 2a.5.5 0 0 1 0 .708l-2 2a.5.5 0 0 1-.708-.708L14.293 8.5H10.5A.5.5 0 0 1 10 8"/>
                                </svg>
                                <p className="mt-1">Buffer</p>
                                <div className="mt-[3px] pl-2">
                                    <input ref={bufferDistanceInput} className="w-3/4" type="text" placeholder="distance"/>
                                    <input className="w-1/4 btn bg-blue-600 text-white hover:bg-blue-800" type="submit" value="Run" onClick={ () => {
                                        (document.getElementById("repl-input") as HTMLTextAreaElement)!.value = `buffer '${props.item.layer.schema}.${props.item.layer.name}' ${(bufferDistanceInput.current! as HTMLInputElement).value}`;
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
