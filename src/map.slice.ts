import { createSlice } from "@reduxjs/toolkit";
import { VectorLayer, RasterLayer } from "./types/Layer.type";

type AddLayerAction = {
    payload: {
        layer: VectorLayer | RasterLayer
    }
}

export const MapSlice = createSlice({
  name: "Map",
  initialState: {
    vectorLayers: [],
    rasterLayers: [],
  },

  reducers: {
    addVectorLayer: (state: any, action: AddLayerAction) => {
      state.vectorLayers.push(action.payload);
    },

    addRasterLayer: (state: any, action: AddLayerAction) => {
      state.rasterLayers.push(action.payload);
    },

    removeAllVectorLayers: (state: any) => {
        state.vectorLayers = [];
    }
  },
});

export const {
    addVectorLayer,
    addRasterLayer,
    removeAllVectorLayers
} = MapSlice.actions;

export default MapSlice.reducer;
