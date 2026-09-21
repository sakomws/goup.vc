import { expect } from "@open-wc/testing";

import { CARTO_BASEMAP_API_KEY, getCartoVoyagerTileUrl } from "/static/js/common/map-tiles.js";

describe("CARTO map tiles", () => {
  it("includes the browser API key in standard tiles", () => {
    expect(getCartoVoyagerTileUrl()).to.equal(
      `https://{s}.basemaps.cartocdn.com/rastertiles/voyager/{z}/{x}/{y}.png?key=${CARTO_BASEMAP_API_KEY}`,
    );
  });

  it("uses high-resolution keyed tiles on retina displays", () => {
    expect(getCartoVoyagerTileUrl(true)).to.include(`/{y}@2x.png?key=${CARTO_BASEMAP_API_KEY}`);
  });
});
