// Owl. on Chromium: lets Owl. Calm Home (written as a browser extension) run as Owl.'s own home
// page. It stands in for the extension APIs the page uses, with data Owl. keeps itself:
//   - history and shortcuts: visits Owl. records (window.OWL_DATA, from owl-data.js);
//   - storage: this page's local storage.
// Favourites, downloads and the Windows helper are not available in this build yet, so those
// APIs are absent and the page hides those parts.
(function () {
  const D = window.OWL_DATA || { history: [], top: [] };
  const KEY = "owl.extstore";
  const load = () => {
    try {
      return JSON.parse(localStorage.getItem(KEY) || "{}");
    } catch {
      return {};
    }
  };
  const save = (o) => localStorage.setItem(KEY, JSON.stringify(o));
  window.chrome = {
    storage: {
      local: {
        async get(keys) {
          const all = load();
          if (keys == null) return all;
          const out = {};
          for (const k of [].concat(keys)) if (k in all) out[k] = all[k];
          return out;
        },
        async set(obj) {
          save(Object.assign(load(), obj));
        },
        async remove(keys) {
          const all = load();
          for (const k of [].concat(keys)) delete all[k];
          save(all);
        },
      },
    },
    runtime: {
      getManifest: () => ({ version: "1.3.0" }),
      sendMessage: async () => true,
      lastError: null,
    },
    history: {
      async search({ text = "", startTime = 0, endTime = Date.now() + 1, maxResults = 100 }) {
        const seen = new Map();
        for (const v of D.history) {
          if (v.t < startTime || v.t >= endTime) continue;
          if (text && !(v.url + " " + v.title).toLowerCase().includes(text.toLowerCase())) continue;
          const e = seen.get(v.url);
          if (!e || e.lastVisitTime < v.t) seen.set(v.url, { url: v.url, title: v.title, lastVisitTime: v.t });
        }
        return [...seen.values()].sort((a, b) => b.lastVisitTime - a.lastVisitTime).slice(0, maxResults);
      },
      async getVisits({ url }) {
        return D.history.filter((v) => v.url === url).map((v) => ({ visitTime: v.t }));
      },
    },
    topSites: {
      async get() {
        return D.top;
      },
    },
    tabs: {
      update({ url }) {
        location.href = url;
      },
    },
  };
})();
