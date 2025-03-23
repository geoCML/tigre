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
    vectorLayers: {},
    rasterLayers: {},
  },

  reducers: {
    addVectorLayer: (state: any, action: AddLayerAction) => {
      state.vectorLayers[action.payload.layer.name] = action.payload;
    },

    removeVectorLayer: (state: any, action: AddLayerAction) => {
      delete state.vectorLayers[action.payload.layer.name];
    },

    addRasterLayer: (state: any, action: AddLayerAction) => {
      state.rasterLayers[action.payload.layer.name] = action.payload;
    },

    toggleVectorLayerVisibility(state: any, action: { payload: string }) {
      state.vectorLayers[action.payload].layer.visible = !state.vectorLayers[action.payload].layer.visible;
    },

    removeAllVectorLayers: (state: any) => {
        state.vectorLayers = {};
    }
  },
});

export const {
    addVectorLayer,
    addRasterLayer,
    toggleVectorLayerVisibility,
    removeAllVectorLayers,
    removeVectorLayer
} = MapSlice.actions;

export default MapSlice.reducer;
