import { useEffect, useRef, useState } from "react";
import { useSelector } from "react-redux";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { parse } from "wkt";
import { VectorLayer } from "../types/Layer.type";
import L from "leaflet";

function Map() {
  let map = useRef<L.Map>(undefined);
  const [redrawing, setRedrawing] = useState(false);
  const [layersPaneVisible, setLayersPaneVisible] = useState(false);
  const vectorLayers = useSelector((state: any) => state.map.vectorLayers);
  const rasterLayers = useSelector((state: any) => state.map.rasterLayers);

  function toggleLayersPane() {
    setLayersPaneVisible(!layersPaneVisible);
  }

  function redraw() {
      if (redrawing || !map.current)
          return

      emit("loading", 75);
      setRedrawing(true);
      map.current!.eachLayer((map_layer) => {
          map.current!.removeLayer(map_layer)
      });

      L.tileLayer('https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}{r}.png', {
          attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors &copy; <a href="https://carto.com/attributions">CARTO</a>',
          subdomains: 'abcd',
          maxZoom: 20,
          detectRetina: false
      }).addTo(map.current!);

      emit("loading", 90);

      const geomPromises = []
      for (const lyr of vectorLayers) {
          const bounds = map.current!.getBounds()
          geomPromises.push(invoke<string[]>("get_as_wkt", {
              table: lyr.layer.name,
              bb: [[bounds.getEast(), bounds.getSouth()], [bounds.getWest(), bounds.getNorth()]]}
          ).then((result) => {
              result.forEach((geom) => {
                L.geoJson(parse(geom)).addTo(map.current!);
              });
          }));
      }

      Promise.all(geomPromises).then(() => {
          setRedrawing(false);
          emit("loading", 0);
      });
  }

  useEffect(() => {
      if (!map.current) {
          map.current = L.map("map", { renderer: new L.Canvas(), fadeAnimation: false });
          map.current!.setView([0, 0], 2);
          redraw();
      }

      map.current!.addEventListener("zoomend", () => {
          redraw();
      }, { once: true });

      map.current!.addEventListener("dragend", () => {
          redraw();
      }, { once: true });

      redraw();
  }, [vectorLayers, rasterLayers]);

  return (
    <>
        <div
            id="toggle-layers-btn"
            className="ml-2 z-[998] absolute rounded-full btn bg-white text-center border-solid border-stone-300 border-2"
            style={{
                bottom: 120,
            }}
            onClick= {() => toggleLayersPane()}
        >
            <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" fill="currentColor" className="bi bi-layers text-stone-600 p-4" viewBox="3 3 10 10">
                <path d="M8.235 1.559a.5.5 0 0 0-.47 0l-7.5 4a.5.5 0 0 0 0 .882L3.188 8 .264 9.559a.5.5 0 0 0 0 .882l7.5 4a.5.5 0 0 0 .47 0l7.5-4a.5.5 0 0 0 0-.882L12.813 8l2.922-1.559a.5.5 0 0 0 0-.882zm3.515 7.008L14.438 10 8 13.433 1.562 10 4.25 8.567l3.515 1.874a.5.5 0 0 0 .47 0zM8 9.433 1.562 6 8 2.567 14.438 6z"/>
            </svg>
          </div>
          <div id="layers-pane" className="z-[998] absolute w-80 h-100 bg-white rounded overflow-auto border border-2 border-solid border-stone-300" style={{
              bottom: 165,
              marginLeft: 20,
              visibility: layersPaneVisible ? "visible" : "hidden"
          }}>
              <h1 className="text-xl mb-2 p-2">Layers</h1>
              <table className="w-full">
                {vectorLayers.map((lyr: { layer: VectorLayer }) => {
                    return (
                    <tr className="border-solid border-1 border-stone-200">
                        <input
                            className="ml-4"
                            type="checkbox"
                            id={lyr.layer.name}
                            value=""
                            checked={true}
                            onClick={() => {
                            //dispatch(toggleLayer(layer.name));
                          }}
                        />
                        <label
                          htmlFor={lyr.layer.name}
                          className="pl-2"
                        >
                       {lyr.layer.name}
                        </label>
                      </tr>
                    );
                })}
              </table>
        </div>
        <div id="map" className="position-absolute m-auto h-[86vh] w-full"></div>
    </>
  );
}

export default Map;
