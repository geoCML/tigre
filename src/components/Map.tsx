import { useEffect, useRef, useState } from "react";
import { useSelector } from "react-redux";
import { invoke } from "@tauri-apps/api/core";
import { VectorLayer, RasterLayer } from "../types/Layer.type";
import L from "leaflet";

function Map() {
  let map = useRef<L.Map>(undefined);
  const [redrawing, setRedrawing] = useState(false);
  const vectorLayers = useSelector((state: any) => state.map.vectorLayers);
  const rasterLayers = useSelector((state: any) => state.map.rasterLayers);

  function redraw() {
      if (redrawing || !map.current)
          return

      setRedrawing(true);
      map.current!.eachLayer((map_layer) => {
          map.current!.removeLayer(map_layer)
      });

      L.tileLayer('https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}{r}.png', {
          attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors &copy; <a href="https://carto.com/attributions">CARTO</a>',
          subdomains: 'abcd',
          maxZoom: 20
      }).addTo(map.current!);

      const geomPromises = []
      for (const lyr of vectorLayers) {
          const bounds = map.current!.getBounds()
          geomPromises.push(invoke<string>("get_as_json", {
              table: lyr.layer.name,
              bb: [[bounds.getEast(), bounds.getSouth()], [bounds.getWest(), bounds.getNorth()]]}).then((result) => {
            L.geoJson(JSON.parse(result)).addTo(map.current!);
          }));
      }

      Promise.all(geomPromises).then(() => {
          setRedrawing(false);
      });
  }

  useEffect(() => {
      if (!map.current) {
          map.current = L.map("map", { renderer: new L.SVG(), fadeAnimation: false });
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
