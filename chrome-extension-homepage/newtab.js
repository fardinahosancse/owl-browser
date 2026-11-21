// Owl. Calm Home: the page. Everything it shows comes from this browser and stays on this
// computer; only the weather asks Open-Meteo, and only for the place you chose.
(function () {
  const { p2, ext, dayKey, hostOf, dur, el, favicon, clock12 } = Owl;
  const $ = (i) => document.getElementById(i);

  const part = (h) => (h < 5 ? "Night" : h < 12 ? "Morning" : h < 17 ? "Afternoon" : h < 21 ? "Evening" : "Night");
  const greet = (h) => (h < 5 ? "Still up" : h < 12 ? "Good morning" : h < 17 ? "Good afternoon" : h < 21 ? "Good evening" : "Good night");
  const ago = (ms) => {
    const s = Math.max(0, Math.floor((Date.now() - ms) / 1000));
    return s < 60 ? "just now" : s < 3600 ? Math.floor(s / 60) + " min ago" : s < 86400 ? Math.floor(s / 3600) + " h ago" : Math.floor(s / 86400) + " d ago";
  };

  // ---------- clock and the line of the day ----------
  const cw = document.querySelector(".cw");
  function tick() {
    const d = new Date(), h = d.getHours(), m = d.getMinutes(), twelve = clock12();
    $("chh").textContent = twelve ? String(h % 12 || 12) : p2(h);
    $("cmm").textContent = p2(m);
    $("cap").textContent = twelve ? (h < 12 ? "AM" : "PM") : "";
    $("cg").textContent = greet(h);
    $("cd").textContent = d.toLocaleDateString("en", { weekday: "long", day: "numeric", month: "long" });
    const pc = ((h * 60 + m) / 1440) * 100 + "%";
    $("dfl").style.setProperty("--p", pc);
    $("dnw").style.setProperty("--p", pc);
    ["0", "6", "12", "18", "24"].forEach((x, i) => ($("h" + i).textContent = twelve ? ["12a", "6a", "12p", "6p", "12a"][i] : p2(+x)));
    $("cpart").textContent = part(h) + " · " + Math.round((1440 - h * 60 - m) / 60) + " hours left in the day";
    cw.style.setProperty("--sx", 15 + (70 * (h * 60 + m)) / 1440 + "%");
    cw.style.setProperty("--sy", 20 + (30 * Math.abs(12 - h)) / 12 + "%");
    document.querySelectorAll("time[data-t]").forEach((x) => (x.textContent = ago(+x.dataset.t)));
  }
  tick();
  setInterval(tick, 15000);

  // ---------- bookmarks bar: the browser's own, so its sync (Chrome, Brave, Edge) carries over ----------
  const pop = $("bmpop");
  function bmLink(n) {
    const a = el("a", "bmi");
    a.href = n.url;
    a.title = n.title ? `${n.title}\n${n.url}` : n.url;
    const img = el("img");
    img.alt = "";
    img.src = favicon(n.url, 32);
    a.append(img, el("span", "", n.title || hostOf(n.url) || n.url));
    return a;
  }
  function openFolder(btn, node) {
    if (!pop.hidden && pop.dataset.id === node.id) return closePop();
    pop.dataset.id = node.id;
    pop.replaceChildren();
    const list = (node.children || []).filter((c) => c.url || c.children);
    if (!list.length) pop.append(el("p", "quiet", "This folder is empty."));
    for (const c of list) {
      if (c.url) pop.append(bmLink(c));
      else {
        const f = el("button", "bmi folder", c.title || "Folder");
        f.type = "button";
        f.onclick = (e) => {
          e.stopPropagation();
          openFolder(btn, c);
        };
        pop.append(f);
      }
    }
    const r = btn.getBoundingClientRect();
    pop.style.setProperty("--x", Math.min(r.left, innerWidth - 340) + "px");
    pop.style.setProperty("--y", r.bottom + 6 + "px");
    pop.hidden = false;
    btn.setAttribute("aria-expanded", "true");
  }
  function closePop() {
    pop.hidden = true;
    document.querySelectorAll(".bm [aria-expanded=true]").forEach((b) => b.setAttribute("aria-expanded", "false"));
  }
  document.addEventListener("click", (e) => {
    if (!pop.hidden && !pop.contains(e.target) && !e.target.closest(".bm .folder")) closePop();
  });
  async function bookmarks() {
    const bar = $("bm");
    if (!ext || !chrome.bookmarks || localStorage.getItem("owl.bm") === "off") {
      bar.hidden = true;
      return;
    }
    const [root] = await chrome.bookmarks.getTree();
    const folder = root.children.find((c) => c.folderType === "bookmarks-bar") || root.children[0];
    const items = (folder && folder.children) || [];
    bar.replaceChildren();
    for (const n of items) {
      if (n.url) bar.append(bmLink(n));
      else if (n.children) {
        const b = el("button", "bmi folder", n.title || "Folder");
        b.type = "button";
        b.setAttribute("aria-haspopup", "true");
        b.setAttribute("aria-expanded", "false");
        b.onclick = () => openFolder(b, n);
        bar.append(b);
      }
    }
    bar.hidden = !items.length;
  }
  if (ext && chrome.bookmarks) {
    for (const ev of ["onCreated", "onRemoved", "onChanged", "onMoved", "onChildrenReordered", "onImportEnded"]) {
      if (chrome.bookmarks[ev]) chrome.bookmarks[ev].addListener(() => bookmarks());
    }
  }

  // ---------- pages you left: distinct sites from history, newest first ----------
  async function recent() {
    if (!ext || !chrome.history) return;
    const items = await chrome.history.search({ text: "", maxResults: 200, startTime: Date.now() - 14 * 86400e3 });
    const seen = new Set(), rows = [];
    for (const it of items.sort((a, b) => b.lastVisitTime - a.lastVisitTime)) {
      const h = hostOf(it.url);
      if (!h || seen.has(h)) continue;
      seen.add(h);
      const a = el("a", "rc");
      a.href = it.url;
      a.style.setProperty("--d", seen.size);
      a.append(el("span", "rn", p2(seen.size)), el("span", "rt", (it.title || "").trim().slice(0, 90) || h), el("span", "rh", h));
      const t = el("time");
      t.dataset.t = it.lastVisitTime;
      t.textContent = ago(it.lastVisitTime);
      a.append(t);
      rows.push(a);
      if (seen.size >= 6) break;
    }
    if (rows.length) $("recent").replaceChildren(...rows);
  }

  // ---------- today's numbers ----------
  async function numbers() {
    if (!ext) return;
    try {
      await chrome.runtime.sendMessage("refresh");
    } catch {}
    const s = await chrome.storage.local.get(["daily", "days"]);
    const today = (s.days || {})[dayKey()] || {};
    $("stoday").textContent = dur((s.daily || {})[dayKey()]);
    const top = Object.entries(today).sort((a, b) => b[1] - a[1]);
    $("stop").textContent = top.length ? `, most of it on ${top[0][0]}` : "";
    if (top.length) {
      const max = Math.max(top[0][1], 1);
      $("spent").replaceChildren(
        ...top.slice(0, 4).map(([h, v]) => {
          const r = el("div", "sp");
          const bar = el("i");
          bar.style.setProperty("--w", Math.max(4, Math.round((v * 100) / max)) + "%");
          r.append(el("span", "", h), el("b", "", dur(v)), bar);
          return r;
        })
      );
      $("spentbox").hidden = false;
    }
    const hints = [];
    if (!localStorage.getItem("owl.name")) hints.push("Add your name in Settings for a personal greeting.");
    if (!localStorage.getItem("owl.wx")) hints.push("Turn on the weather in Settings: your location or any city.");
    hints.push("Open the Report for your day, week and month.");
    $("hint").textContent = hints[0];
    if (chrome.downloads) {
      const run = await chrome.downloads.search({ state: "in_progress" });
      $("sdl").textContent = run.length ? " " + run.length + (run.length === 1 ? " download is" : " downloads are") + " running." : "";
    }
  }

  // ---------- weather (Open-Meteo: free, no key) ----------
  const WMO = { 0: "Clear", 1: "Mostly clear", 2: "Partly cloudy", 3: "Overcast", 45: "Fog", 48: "Rime fog", 51: "Light drizzle", 53: "Drizzle", 55: "Heavy drizzle", 56: "Freezing drizzle", 57: "Freezing drizzle", 61: "Light rain", 63: "Rain", 65: "Heavy rain", 66: "Freezing rain", 67: "Freezing rain", 71: "Light snow", 73: "Snow", 75: "Heavy snow", 77: "Snow grains", 80: "Rain showers", 81: "Rain showers", 82: "Violent showers", 85: "Snow showers", 86: "Snow showers", 95: "Thunderstorm", 96: "Thunderstorm, hail", 99: "Thunderstorm, hail" };
  const wxPlace = () => {
    try {
      return JSON.parse(localStorage.getItem("owl.wxplace") || "null");
    } catch {
      return null;
    }
  };
  const here = () =>
    new Promise((ok, no) => navigator.geolocation.getCurrentPosition((p) => ok({ lat: +p.coords.latitude.toFixed(2), lon: +p.coords.longitude.toFixed(2), name: "" }), no, { timeout: 10000, maximumAge: 3600e3 }));
  async function weather(force) {
    const mode = localStorage.getItem("owl.wx") || "off";
    const box = $("wx");
    if (mode === "off") return void (box.hidden = true);
    let place = mode === "city" ? wxPlace() : null;
    try {
      if (mode === "here") place = await here();
    } catch {
      $("wxnote").textContent = "Your browser did not share the location. Allow it, or pick a city.";
      return void (box.hidden = true);
    }
    if (!place) return void (box.hidden = true);
    const unit = localStorage.getItem("owl.unit") === "f" ? "fahrenheit" : "celsius";
    const key = `${place.lat},${place.lon},${unit}`;
    let data;
    try {
      const c = JSON.parse(localStorage.getItem("owl.wxcache") || "null");
      if (!force && c && c.key === key && Date.now() - c.t < 30 * 60e3) data = c.data;
    } catch {}
    if (!data) {
      try {
        const u = `https://api.open-meteo.com/v1/forecast?latitude=${place.lat}&longitude=${place.lon}&current=temperature_2m,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=4&temperature_unit=${unit}`;
        data = await (await fetch(u)).json();
        localStorage.setItem("owl.wxcache", JSON.stringify({ key, t: Date.now(), data }));
      } catch {
        return void (box.hidden = true);
      }
    }
    const deg = "°";
    $("wxplace").textContent = place.name || (data.timezone || "").split("/").pop().replace(/_/g, " ");
    $("wxt").textContent = Math.round(data.current.temperature_2m) + deg;
    $("wxc").textContent = WMO[data.current.weather_code] || "";
    $("wxhl").textContent = `High ${Math.round(data.daily.temperature_2m_max[0])}${deg} · Low ${Math.round(data.daily.temperature_2m_min[0])}${deg}`;
    $("wxd").replaceChildren(
      ...data.daily.time.slice(1, 4).map((t, i) => {
        const d = el("div");
        d.append(
          el("span", "", new Date(t + "T12:00").toLocaleDateString("en", { weekday: "short" })),
          el("b", "", `${Math.round(data.daily.temperature_2m_max[i + 1])}${deg} / ${Math.round(data.daily.temperature_2m_min[i + 1])}${deg}`),
          el("small", "", WMO[data.daily.weather_code[i + 1]] || "")
        );
        return d;
      })
    );
    box.hidden = false;
  }

  recent();
  numbers();
  bookmarks();
  weather();
  setInterval(numbers, 30000);
  setInterval(() => weather(), 30 * 60e3);
  document.addEventListener("visibilitychange", () => {
    if (!document.hidden) {
      recent();
      numbers();
    }
  });

  // ---------- search ----------
  $("cf2").onsubmit = (e) => {
    e.preventDefault();
    const v = $("cq").value.trim();
    if (!v) return;
    if (/^[a-z]+:\/\//i.test(v)) return void (location.href = v);
    if (!/\s/.test(v) && /\./.test(v)) return void (location.href = "https://" + v);
    if (localStorage.getItem("owl.search") === "browser" && ext && chrome.search) chrome.search.query({ text: v, disposition: "CURRENT_TAB" });
    else location.href = "https://duckduckgo.com/?q=" + encodeURIComponent(v);
  };
  document.addEventListener("keydown", (e) => {
    if (e.key === "/" && document.activeElement !== $("cq") && $("pane").hidden) {
      e.preventDefault();
      $("cq").focus();
    }
    if (e.key === "Escape") {
      if (!pop.hidden) closePop();
      else if (!$("pane").hidden) closePane();
    }
  });
  document.querySelectorAll("a[data-chrome]").forEach((a) =>
    a.addEventListener("click", (e) => {
      if (!ext) return;
      e.preventDefault();
      chrome.tabs.update({ url: a.getAttribute("href") });
    })
  );

  // ---------- settings ----------
  const name = () => (localStorage.getItem("owl.name") || "").trim().split(/\s+/)[0] || "";
  const showName = () => ($("cn").textContent = name() ? ", " + name() : "");
  showName();
  function applyTheme() {
    const pick = localStorage.getItem("owl.theme") || "system";
    const dark = pick === "dark" || (pick === "system" && matchMedia("(prefers-color-scheme: dark)").matches);
    document.documentElement.dataset.theme = dark ? "dark" : "light";
  }
  matchMedia("(prefers-color-scheme: dark)").addEventListener("change", applyTheme);
  // segmented choices: [selector attribute, storage key, default, after]
  const SEGS = [
    ["themePick", "owl.theme", "system", applyTheme],
    ["clock", "owl.clock", "24", tick],
    ["search", "owl.search", "ddg", null],
    ["unit", "owl.unit", "c", () => weather(true)],
    ["wx", "owl.wx", "off", () => { $("cityf").hidden = localStorage.getItem("owl.wx") !== "city"; weather(true); }],
  ];
  const attr = (k) => "data-" + k.replace(/[A-Z]/g, (c) => "-" + c.toLowerCase());
  function showSegs() {
    for (const [k, store, def] of SEGS) {
      const cur = localStorage.getItem(store) || def;
      document.querySelectorAll(`[${attr(k)}]`).forEach((b) => {
        b.classList.toggle("on", b.dataset[k] === cur);
        b.setAttribute("aria-pressed", b.dataset[k] === cur);
      });
    }
    $("cityf").hidden = (localStorage.getItem("owl.wx") || "off") !== "city";
    $("bmon").setAttribute("aria-checked", localStorage.getItem("owl.bm") !== "off");
  }
  for (const [k, store, , after] of SEGS) {
    document.querySelectorAll(`[${attr(k)}]`).forEach((b) =>
      b.addEventListener("click", () => {
        localStorage.setItem(store, b.dataset[k]);
        showSegs();
        if (after) after();
      })
    );
  }
  $("bmon").addEventListener("click", () => {
    localStorage.setItem("owl.bm", localStorage.getItem("owl.bm") === "off" ? "on" : "off");
    showSegs();
    bookmarks();
  });
  $("cityf").addEventListener("submit", async (e) => {
    e.preventDefault();
    const q = $("city").value.trim();
    if (!q) return;
    const box = $("cityr");
    box.replaceChildren(el("p", "small", "Looking..."));
    try {
      const r = await (await fetch(`https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(q)}&count=5&language=en&format=json`)).json();
      const list = r.results || [];
      box.replaceChildren(...(list.length ? [] : [el("p", "small", "No place found with that name.")]));
      for (const c of list) {
        const b = el("button", "ghost", [c.name, c.admin1, c.country].filter(Boolean).join(", "));
        b.type = "button";
        b.onclick = () => {
          localStorage.setItem("owl.wxplace", JSON.stringify({ lat: +c.latitude.toFixed(2), lon: +c.longitude.toFixed(2), name: c.name }));
          box.replaceChildren(el("p", "small", "Weather for " + c.name + "."));
          weather(true);
        };
        box.append(b);
      }
    } catch {
      box.replaceChildren(el("p", "small", "Could not reach Open-Meteo. Try again later."));
    }
  });
  $("clear").addEventListener("click", async () => {
    if (!confirm("Clear the time-online numbers kept by this home page? The reports start again from today.")) return;
    if (ext) await chrome.storage.local.remove(["daily", "days", "hours", "usage", "cur"]);
    $("spentbox").hidden = true;
    numbers();
  });
  $("pname").value = localStorage.getItem("owl.name") || "";
  $("pname").addEventListener("input", () => {
    localStorage.setItem("owl.name", $("pname").value.trim());
    showName();
  });
  function openPane() {
    $("pane").hidden = false;
    $("gear").setAttribute("aria-expanded", "true");
    showSegs();
    $("pname").focus();
  }
  function closePane() {
    $("pane").hidden = true;
    $("gear").setAttribute("aria-expanded", "false");
    $("gear").focus();
    numbers();
  }
  $("gear").addEventListener("click", () => ($("pane").hidden ? openPane() : closePane()));
  $("done").addEventListener("click", closePane);
  applyTheme();
})();
