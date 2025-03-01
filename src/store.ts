import { configureStore } from "@reduxjs/toolkit";
import mapReducer from "./map.slice";

export default configureStore({
  reducer: {
    map: mapReducer,
  },
});
