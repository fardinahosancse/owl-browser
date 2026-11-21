// Shared helpers for the home page and the report.
const Owl = (() => {
  const p2 = (n) => String(n).padStart(2, "0");
  const ext = typeof chrome !== "undefined" && chrome.storage && chrome.storage.local;
  const dayKey = (d = new Date()) => `${d.getFullYear()}-${p2(d.getMonth() + 1)}-${p2(d.getDate())}`;
  const hostOf = (u) => {
    try {
      const x = new URL(u);
      return x.protocol === "http:" || x.protocol === "https:" ? x.hostname.replace(/^www\./, "") : "";
    } catch {
      return "";
    }
  };
  const dur = (s) => {
    s = Math.round(s || 0);
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60);
    return h ? `${h}h ${m}m` : s && !m ? "<1m" : `${m}m`;
  };
  const el = (t, c, x) => {
    const e = document.createElement(t);
    if (c) e.className = c;
    if (x != null) e.textContent = x;
    return e;
  };
  const favicon = (pageUrl, size = 32) =>
    ext && chrome.runtime.getURL ? `${chrome.runtime.getURL("/_favicon/")}?pageUrl=${encodeURIComponent(pageUrl)}&size=${size}` : "";
  // "youtube.com" from "m.youtube.com"; the site, not the server
  const site = (h) => {
    const parts = h.split(".");
    const two = /^(co|com|org|net|ac|gov|edu)\.[a-z]{2}$/.test(parts.slice(-2).join("."));
    return parts.slice(two ? -3 : -2).join(".");
  };
  const clock12 = () => localStorage.getItem("owl.clock") === "12";
  // The page files can be newer than what the browser has loaded: when the extension folder is
  // updated in place, the browser uses the new pages at once but keeps the old manifest (and its
  // permissions) until the extension is reloaded on the extensions page.
  const VERSION = "1.1.1";
  const stale = () => ext && chrome.runtime.getManifest && chrome.runtime.getManifest().version !== VERSION;
  const RELOAD = "Owl. Calm Home was updated. Open the extensions page and press Reload on Owl. Calm Home to finish the update.";
  return { p2, ext, dayKey, hostOf, dur, el, favicon, site, clock12, stale, RELOAD };
})();
