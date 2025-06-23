import chroma from "chroma-js";
import colors from "tailwindcss/colors";

export const hotCold: ColorScale = {
  name: "Hot-Cold",
  light: chroma
    .scale([colors.blue[500], colors.neutral[600], colors.red[500]])
    .domain([0, 0.5, 1])
    .mode("lab"),
  dark: chroma
    .scale([colors.blue[500], colors.neutral[200], colors.red[500]])
    .domain([0, 0.5, 1])
    .mode("lab"),
};

export const orangeBlue: ColorScale = {
  name: "Orange-Blue",
  light: chroma
    .scale([colors.amber[600], colors.neutral[600], colors.blue[600]])
    .domain([0, 0.5, 1])
    .mode("lab")
    .classes([0, 0.1, 0.35, 0.65, 0.9, 1]),
  dark: chroma
    .scale([colors.amber[500], colors.neutral[200], colors.blue[500]])
    .domain([0, 0.5, 1])
    .mode("lab")
    .classes([0, 0.1, 0.35, 0.65, 0.9, 1]),
};

export const defaultScale: ColorScale = orangeBlue;

export const scales = { orangeBlue, hotCold };

export type ScaleId = keyof typeof scales;

export interface ColorScale {
  name: string;
  light: chroma.Scale;
  dark: chroma.Scale;
}
