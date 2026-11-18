import { expect } from "@open-wc/testing";

const HOME_TEMPLATES = [
  "/ocg-server/templates/dashboard/alliance/home.html",
  "/ocg-server/templates/dashboard/group/home.html",
  "/ocg-server/templates/dashboard/user/home.html",
];

describe("dashboard mobile home templates", () => {
  for (const path of HOME_TEMPLATES) {
    it(`opens ${path} on mobile`, async () => {
      const response = await fetch(path);
      expect(response.ok).to.equal(true);

      const template = (await response.text()).replace(/\s+/g, " ");
      expect(template).to.include("{% block mobile_dashboard_view -%} {% endblock mobile_dashboard_view -%}");
      expect(template).to.include(
        "{% block dashboard_main_classes -%} {% endblock dashboard_main_classes -%}",
      );
      expect(template).to.not.include("This dashboard is not optimized yet for mobile devices");
    });
  }
});
