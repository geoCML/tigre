import { Symbology } from "./Symbology.type";

export type VectorLayer = {
    schema: string,
    name: string,
    visible: boolean,
    symbology: Symbology
}

export type RasterLayer = {
    schema: string,
    name: string
}
