// CARTO basemap keys are public browser credentials. Keep this key restricted
// to GOUP's production and custom-domain origins in the CARTO dashboard.
export const CARTO_BASEMAP_API_KEY = "cb1_3rxb_1_5552a46eb5cde28d77aa7fc6";

export const getCartoVoyagerTileUrl = (retina = false) =>
  `https://{s}.basemaps.cartocdn.com/rastertiles/voyager/{z}/{x}/{y}${
    retina ? "@2x.png" : ".png"
  }?key=${CARTO_BASEMAP_API_KEY}`;
