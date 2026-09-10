import { expect } from "@open-wc/testing";

import {
  escapeHtml,
  insertTrustedHtml,
  readTrustedHtml,
  setTrustedHtml,
} from "/static/js/common/trusted-html.js";
import { resetDom } from "/tests/unit/test-utils/dom.js";

describe("common trusted html", () => {
  beforeEach(() => {
    resetDom();
  });

  afterEach(() => {
    resetDom();
  });

  it("escapes untrusted text for html insertion", () => {
    expect(escapeHtml(`<img src=x onerror="alert(1)">`)).to.equal(
      "&lt;img src=x onerror=&quot;alert(1)&quot;&gt;",
    );
    expect(escapeHtml("Tom & Jerry")).to.equal("Tom &amp; Jerry");
    expect(escapeHtml(null)).to.equal("");
  });

  it("reads and writes trusted html fragments", () => {
    // Build the element that owns a server-rendered HTML fragment.
    const element = document.createElement("section");

    // The helpers preserve markup for fragments that are already trusted.
    setTrustedHtml(element, "<strong>Accepted</strong>");
    expect(readTrustedHtml(element)).to.equal("<strong>Accepted</strong>");
    insertTrustedHtml(element, "beforeend", "<em> speaker</em>");
    expect(readTrustedHtml(element)).to.equal("<strong>Accepted</strong><em> speaker</em>");
  });

  it("normalizes missing trusted html values", () => {
    // Build the element that receives optional trusted markup.
    const element = document.createElement("section");

    // Missing values clear the target, and missing elements are ignored.
    setTrustedHtml(element, null);
    expect(readTrustedHtml(element)).to.equal("");
    expect(readTrustedHtml(null)).to.equal("");
    expect(() => setTrustedHtml(null, "<span>Ignored</span>")).not.to.throw();
    expect(() => insertTrustedHtml(null, "beforeend", "<span>Ignored</span>")).not.to.throw();
  });
});
