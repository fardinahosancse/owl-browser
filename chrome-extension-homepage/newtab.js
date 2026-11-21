// Owl. Calm Home: the page. Everything it shows comes from this browser and stays on this computer.
(function () {
  const $ = (i) => document.getElementById(i);
  const p2 = (n) => String(n).padStart(2, "0");
  // runs outside the extension too (opening the file directly), with sample-free empty data
  const ext = typeof chrome !== "undefined" && chrome.storage && chrome.storage.local;

  const part = (h) => (h < 5 ? "Night" : h < 12 ? "Morning" : h < 17 ? "Afternoon" : h < 21 ? "Evening" : "Night");
  const greet = (h) => (h < 5 ? "Still up" : h < 12 ? "Good morning" : h < 17 ? "Good afternoon" : h < 21 ? "Good evening" : "Good night");
  const ago = (ms) => {
    const s = Math.max(0, Math.floor((Date.now() - ms) / 1000));
    return s < 60 ? "just now" : s < 3600 ? Math.floor(s / 60) + " min ago" : s < 86400 ? Math.floor(s / 3600) + " h ago" : Math.floor(s / 86400) + " d ago";
  };
  const dur = (s) => {
    s = Math.round(s || 0);
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60);
    return h ? `${h}h ${m}m` : `${m}m`;
  };
  const todayKey = () => {
    const d = new Date();
    return `${d.getFullYear()}-${p2(d.getMonth() + 1)}-${p2(d.getDate())}`;
  };
  const hostOf = (u) => {
    try {
      const x = new URL(u);
      return x.protocol === "http:" || x.protocol === "https:" ? x.hostname.replace(/^www\./, "") : "";
    } catch {
      return "";
    }
  };
  const el = (t, c, x) => {
    const e = document.createElement(t);
    if (c) e.className = c;
    if (x != null) e.textContent = x;
    return e;
  };

  // ---------- clock and the line of the day ----------
  const cw = document.querySelector(".cw");
  function tick() {
    const d = new Date(), h = d.getHours(), m = d.getMinutes();
    $("chh").textContent = p2(h);
    $("cmm").textContent = p2(m);
    $("cg").textContent = greet(h);
    $("cd").textContent = d.toLocaleDateString("en", { weekday: "long", day: "numeric", month: "long" });
    const pc = ((h * 60 + m) / 1440) * 100 + "%";
    $("dfl").style.setProperty("--p", pc);
    $("dnw").style.setProperty("--p", pc);
    $("cpart").textContent = part(h) + " · " + Math.round((1440 - h * 60 - m) / 60) + " hours left in the day";
    // the light drifts across the page with the time of day
    cw.style.setProperty("--sx", 15 + (70 * (h * 60 + m)) / 1440 + "%");
    cw.style.setProperty("--sy", 20 + (30 * Math.abs(12 - h)) / 12 + "%");
    document.querySelectorAll("time[data-t]").forEach((x) => (x.textContent = ago(+x.dataset.t)));
  }
  tick();
  setInterval(tick, 15000);

  // ---------- pages you left: distinct sites from history, newest first ----------
  async function recent() {
    if (!ext || !chrome.history) return;
    const items = await chrome.history.search({ text: "", maxResults: 200, startTime: Date.now() - 14 * 86400e3 });
    const seen = new Set(), box = $("recent"), rows = [];
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
    if (rows.length) box.replaceChildren(...rows);
  }

  // ---------- the sentence, where time goes, hints ----------
  async function numbers() {
    if (!ext) return;
    try {
      await chrome.runtime.sendMessage("refresh");
    } catch {}
    const s = await chrome.storage.local.get(["daily", "usage", "kickToday", "kickTotal", "kickDay", "kickOn"]);
    const usage = s.usage || {}, daily = s.daily || {};
    $("stoday").textContent = dur(daily[todayKey()]);
    $("sbk").textContent = (s.kickDay === todayKey() ? s.kickToday || 0 : 0).toLocaleString();
    $("stot").textContent = (s.kickTotal || 0).toLocaleString();

    const top = Object.entries(usage).sort((a, b) => b[1] - a[1]).slice(0, 3);
    if (top.length) {
      const max = Math.max(top[0][1], 1);
      $("spent").replaceChildren(
        ...top.map(([h, v]) => {
          const r = el("div", "sp");
          const bar = el("i");
          bar.style.setProperty("--w", Math.max(4, Math.round((v * 100) / max)) + "%");
          r.append(el("span", "", h), el("b", "", dur(v)), bar);
          return r;
        })
      );
      $("spentbox").hidden = false;
    }

    // short, honest hints that follow your settings
    const hints = [];
    if (s.kickOn === false) hints.push("OwlKick is off. Turn it on in Settings to block trackers.");
    if (!localStorage.getItem("owl.name")) hints.push("Add your name in Settings for a personal greeting.");
    hints.push("Everything you do here stays on this computer.");
    $("hint").textContent = hints[0];

    if (chrome.downloads) {
      const run = await chrome.downloads.search({ state: "in_progress" });
      $("sdl").textContent = run.length ? " " + run.length + (run.length === 1 ? " download is" : " downloads are") + " running." : "";
    }
  }
  recent();
  numbers();
  setInterval(numbers, 30000);
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
    if (localStorage.getItem("owl.search") === "browser" && ext && chrome.search) {
      chrome.search.query({ text: v, disposition: "CURRENT_TAB" });
    } else {
      location.href = "https://duckduckgo.com/?q=" + encodeURIComponent(v);
    }
  };
  document.addEventListener("keydown", (e) => {
    if (e.key === "/" && document.activeElement !== $("cq") && $("pane").hidden) {
      e.preventDefault();
      $("cq").focus();
    }
    if (e.key === "Escape" && !$("pane").hidden) closePane();
  });

  // chrome:// pages cannot be opened by a plain link from an extension page
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
    document.querySelectorAll("[data-theme-pick]").forEach((b) => {
      b.classList.toggle("on", b.dataset.themePick === pick);
      b.setAttribute("aria-pressed", b.dataset.themePick === pick);
    });
  }
  matchMedia("(prefers-color-scheme: dark)").addEventListener("change", applyTheme);
  document.querySelectorAll("[data-theme-pick]").forEach((b) =>
    b.addEventListener("click", () => {
      localStorage.setItem("owl.theme", b.dataset.themePick);
      applyTheme();
    })
  );
  const showSearch = () =>
    document.querySelectorAll("[data-search]").forEach((b) => {
      const on = b.dataset.search === (localStorage.getItem("owl.search") || "ddg");
      b.classList.toggle("on", on);
      b.setAttribute("aria-pressed", on);
    });
  document.querySelectorAll("[data-search]").forEach((b) =>
    b.addEventListener("click", () => {
      localStorage.setItem("owl.search", b.dataset.search);
      showSearch();
    })
  );

  async function showKick() {
    const on = ext ? (await chrome.storage.local.get("kickOn")).kickOn !== false : true;
    $("kick").setAttribute("aria-checked", on);
  }
  $("kick").addEventListener("click", async () => {
    const on = $("kick").getAttribute("aria-checked") !== "true";
    $("kick").setAttribute("aria-checked", on);
    if (ext) await chrome.runtime.sendMessage({ kick: on });
    numbers();
  });
  $("clear").addEventListener("click", async () => {
    if (!confirm("Clear the time-online numbers kept by this home page?")) return;
    if (ext) await chrome.storage.local.remove(["daily", "usage", "cur"]);
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
    applyTheme();
    showSearch();
    showKick();
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
