/**
 * Lightweight, extensible internationalization (i18n) module for GOUP.
 */

export const SUPPORTED_LOCALES = {
  en: { name: "English", code: "en", label: "EN" },
  az: { name: "Azərbaycan", code: "az", label: "AZ" },
};

export const DEFAULT_LOCALE = "en";
export const STORAGE_KEY = "ocg_lang";

const translationsCache = new Map();

/**
 * Gets a cookie value by name.
 * @param {string} name
 * @returns {string|null}
 */
function getCookie(name) {
  const match = document.cookie.match(new RegExp(`(^|;\\s*)${name}=([^;]+)`));
  return match ? decodeURIComponent(match[2]) : null;
}

/**
 * Sets a cookie with standard 1-year persistence.
 * @param {string} name
 * @param {string} value
 */
function setCookie(name, value) {
  const maxAge = 365 * 24 * 60 * 60;
  document.cookie = `${name}=${encodeURIComponent(value)}; path=/; max-age=${maxAge}; SameSite=Lax`;
}

/**
 * Resolves current active language code.
 * @returns {string}
 */
export function getCurrentLanguage() {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved && SUPPORTED_LOCALES[saved]) {
      return saved;
    }
  } catch {
    // localStorage may be unavailable in private modes
  }

  const cookieLang = getCookie(STORAGE_KEY);
  if (cookieLang && SUPPORTED_LOCALES[cookieLang]) {
    return cookieLang;
  }

  return DEFAULT_LOCALE;
}

/**
 * Fetches translations dictionary for a given language.
 * @param {string} lang
 * @returns {Promise<Object>}
 */
async function loadTranslations(lang) {
  if (translationsCache.has(lang)) {
    return translationsCache.get(lang);
  }

  try {
    const response = await fetch(`/static/locales/${lang}.json`);
    if (!response.ok) {
      throw new Error(`Failed to load ${lang} locale`);
    }
    const data = await response.json();
    translationsCache.set(lang, data);
    return data;
  } catch (error) {
    console.warn(`[i18n] Could not load translations for '${lang}':`, error);
    return null;
  }
}

/**
 * Resolves a dot-notated key from an object.
 * @param {Object} obj
 * @param {string} path
 * @returns {string|null}
 */
function getNestedValue(obj, path) {
  if (!obj || !path) return null;
  return path.split(".").reduce((acc, part) => (acc && acc[part] !== undefined ? acc[part] : null), obj);
}

/**
 * Translates all data-i18n elements within a root container.
 * @param {HTMLElement|Document} root
 * @param {string} lang
 * @param {Object} dictionary
 */
export function translateTree(root = document, lang = getCurrentLanguage(), dictionary = null) {
  const target = root instanceof Element || root instanceof Document ? root : document;

  target.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n");
    if (!el.dataset.i18nDefault) {
      el.dataset.i18nDefault = el.textContent.trim();
    }

    if (lang === DEFAULT_LOCALE || !dictionary) {
      el.textContent = el.dataset.i18nDefault;
    } else {
      const translation = getNestedValue(dictionary, key);
      if (translation) {
        el.textContent = translation;
      }
    }
  });

  target.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
    const key = el.getAttribute("data-i18n-placeholder");
    if (!el.dataset.i18nPlaceholderDefault) {
      el.dataset.i18nPlaceholderDefault = el.getAttribute("placeholder") || "";
    }

    if (lang === DEFAULT_LOCALE || !dictionary) {
      el.setAttribute("placeholder", el.dataset.i18nPlaceholderDefault);
    } else {
      const translation = getNestedValue(dictionary, key);
      if (translation) {
        el.setAttribute("placeholder", translation);
      }
    }
  });

  target.querySelectorAll("[data-i18n-title]").forEach((el) => {
    const key = el.getAttribute("data-i18n-title");
    if (!el.dataset.i18nTitleDefault) {
      el.dataset.i18nTitleDefault = el.getAttribute("title") || "";
    }

    if (lang === DEFAULT_LOCALE || !dictionary) {
      el.setAttribute("title", el.dataset.i18nTitleDefault);
    } else {
      const translation = getNestedValue(dictionary, key);
      if (translation) {
        el.setAttribute("title", translation);
      }
    }
  });
}

/**
 * Updates UI controls indicating the active language.
 * @param {string} lang
 */
function updateLanguageButtons(lang) {
  document.querySelectorAll("[data-lang-btn]").forEach((btn) => {
    const btnLang = btn.getAttribute("data-lang-btn");
    const isActive = btnLang === lang;
    btn.setAttribute("aria-pressed", isActive ? "true" : "false");
    btn.classList.toggle("bg-[#1A1510]", isActive);
    btn.classList.toggle("text-white", isActive);
    btn.classList.toggle("text-[#3a2f25]", !isActive);
    btn.classList.toggle("hover:bg-black/5", !isActive);
  });

  document.querySelectorAll("#current-lang-label, [data-current-lang-label]").forEach((el) => {
    if (SUPPORTED_LOCALES[lang]) {
      el.textContent = SUPPORTED_LOCALES[lang].label;
    }
  });
}

/**
 * Changes active locale and re-renders text.
 * @param {string} lang
 */
export async function setLanguage(lang) {
  if (!SUPPORTED_LOCALES[lang]) {
    console.warn(`[i18n] Unsupported locale: ${lang}`);
    return;
  }

  try {
    localStorage.setItem(STORAGE_KEY, lang);
  } catch {
    // Ignore storage issues
  }
  setCookie(STORAGE_KEY, lang);

  document.documentElement.lang = lang;
  updateLanguageButtons(lang);

  const dict = lang === DEFAULT_LOCALE ? null : await loadTranslations(lang);
  translateTree(document, lang, dict);

  document.dispatchEvent(new CustomEvent("ocg:languageChanged", { detail: { lang } }));
}

/**
 * Initializes internationalization and binds HTMX events.
 */
export async function initI18n() {
  const currentLang = getCurrentLanguage();
  document.documentElement.lang = currentLang;
  updateLanguageButtons(currentLang);

  if (currentLang !== DEFAULT_LOCALE) {
    const dict = await loadTranslations(currentLang);
    translateTree(document, currentLang, dict);
  }

  // Delegate clicks on any language switcher buttons
  document.addEventListener("click", (event) => {
    const btn = event.target.closest("[data-lang-select]");
    if (btn) {
      event.preventDefault();
      const selectedLang = btn.getAttribute("data-lang-select");
      if (selectedLang) {
        setLanguage(selectedLang);
      }
    }
  });

  // Re-translate content dynamically injected by HTMX
  document.addEventListener("htmx:afterSwap", (event) => {
    const lang = getCurrentLanguage();
    if (lang !== DEFAULT_LOCALE) {
      loadTranslations(lang).then((dict) => {
        translateTree(event.detail.target, lang, dict);
      });
    }
  });
}

// Auto-initialize when script loads
if (typeof document !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => initI18n());
  } else {
    initI18n();
  }
}

// Global hooks for external inline triggers
if (typeof window !== "undefined") {
  window.setOcgLanguage = setLanguage;
  window.getOcgLanguage = getCurrentLanguage;
}
