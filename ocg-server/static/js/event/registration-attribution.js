const ATTRIBUTION_KEYS = [
  "source",
  "referral_code",
  "utm_source",
  "utm_medium",
  "utm_campaign",
  "utm_content",
  "utm_term",
];

/**
 * Returns normalized first-touch fields for registration forms.
 * @param {string} url
 * @param {string} referrer
 * @returns {Record<string, string>}
 */
export const registrationAttribution = (url, referrer = "") => {
  const params = new URL(url).searchParams;
  const attribution = {
    source: (params.get("source") || params.get("utm_source") || "direct").slice(0, 255),
  };

  ATTRIBUTION_KEYS.forEach((key) => {
    const value = params.get(key);
    if (value) {
      attribution[key] = value.slice(0, 255);
    }
  });

  const referralCode = params.get("referral_code") || params.get("ref");
  if (referralCode) {
    attribution.referral_code = referralCode.slice(0, 255);
  }
  if (referrer) {
    attribution.referrer = referrer.slice(0, 2000);
  }
  return attribution;
};

const storageKey = `event-registration-attribution:${window.location.pathname}`;
let attribution = registrationAttribution(window.location.href, document.referrer);
try {
  const stored = window.sessionStorage.getItem(storageKey);
  if (stored) {
    attribution = JSON.parse(stored);
  } else {
    window.sessionStorage.setItem(storageKey, JSON.stringify(attribution));
  }
} catch {
  // Registration must continue when storage is unavailable or contains invalid data.
}

document.addEventListener("htmx:configRequest", (event) => {
  const source = event.detail.elt;
  if (
    !(source instanceof Element) ||
    !source.matches('[data-attendance-role="attend-btn"], [data-attendance-role="checkout-form"]')
  ) {
    return;
  }
  Object.assign(event.detail.parameters, attribution);
});
