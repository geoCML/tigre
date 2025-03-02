import { useEffect, useRef, useState } from "react";
import { useSelector } from "react-redux";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { parse } from "wkt";
import L from "leaflet";

function Map() {
  let map = useRef<L.Map>(undefined);
  const [redrawing, setRedrawing] = useState(false);
  const vectorLayers = useSelector((state: any) => state.map.vectorLayers);
  const rasterLayers = useSelector((state: any) => state.map.rasterLayers);

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
    <div id="map" className="position-absolute m-auto h-[90vh] w-full"></div>
  );
}

export default Map;
