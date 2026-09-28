const placements = document.querySelectorAll("[data-sponsor-placement]");

const nonce = (() => {
  const key = "goup-sponsor-engagement-nonce";
  let value = sessionStorage.getItem(key);
  if (!value) {
    value = crypto.randomUUID();
    sessionStorage.setItem(key, value);
  }
  return value;
})();

const record = (placement, metric) => {
  const { eventId, sponsorId } = placement.dataset;
  const url = `/event-sponsors/${eventId}/${sponsorId}/engagement`;
  const body = JSON.stringify({ metric, session_nonce: nonce });
  if (metric === "click" && navigator.sendBeacon) {
    navigator.sendBeacon(url, new Blob([body], { type: "application/json" }));
    return;
  }
  fetch(url, {
    method: "POST",
    body,
    headers: { "content-type": "application/json" },
    credentials: "omit",
    keepalive: true,
  }).catch(() => {});
};

const observer = new IntersectionObserver(
  (entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      record(entry.target, "impression");
      observer.unobserve(entry.target);
    }
  },
  { threshold: 0.5 },
);

for (const placement of placements) {
  observer.observe(placement);
  placement.querySelector("a")?.addEventListener("click", () => {
    record(placement, "click");
  });
}
