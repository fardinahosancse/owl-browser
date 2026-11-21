// Owl. Calm Home: the report. Day, week or month: time online (counted by this extension) and
// visits (from the browser's history), with a spotlight on one site, YouTube first.
(function () {
  const { p2, ext, dayKey, hostOf, dur, el, favicon, site, clock12 } = Owl;
  const $ = (i) => document.getElementById(i);
  const DAY = 86400e3;
  let period = new URLSearchParams(location.search).get("p") || localStorage.getItem("owl.rp") || "week";
  let offset = 0;
  let spotSite = localStorage.getItem("owl.spot") || "youtube.com";

  const startOfDay = (d) => new Date(d.getFullYear(), d.getMonth(), d.getDate());
  // [start, end) of the period `off` steps back from now
  function bounds(p, off) {
    const now = new Date();
    if (p === "day") {
      const s = startOfDay(new Date(now.getFullYear(), now.getMonth(), now.getDate() + off));
      return [s, new Date(s.getFullYear(), s.getMonth(), s.getDate() + 1)];
    }
    if (p === "week") {
      const t = startOfDay(now);
      const mon = new Date(t.getFullYear(), t.getMonth(), t.getDate() - ((t.getDay() + 6) % 7) + off * 7);
      return [mon, new Date(mon.getFullYear(), mon.getMonth(), mon.getDate() + 7)];
    }
    const s = new Date(now.getFullYear(), now.getMonth() + off, 1);
    return [s, new Date(s.getFullYear(), s.getMonth() + 1, 1)];
  }
  const daysIn = (s, e) => {
    const out = [];
    for (let d = new Date(s); d < e; d = new Date(d.getFullYear(), d.getMonth(), d.getDate() + 1)) out.push(new Date(d));
    return out;
  };
  const fmtHour = (h) => (clock12() ? `${h % 12 || 12} ${h < 12 ? "AM" : "PM"}` : `${p2(h)}:00`);
  const pct = (a, b) => (b ? Math.round(((a - b) / b) * 100) : null);
  const change = (a, b, what) => {
    const c = pct(a, b);
    if (c === null) return { t: "", up: false };
    if (Math.abs(c) < 3) return { t: `about the same as the ${what} before`, up: false };
    return { t: `${Math.abs(c)}% ${c > 0 ? "more" : "less"} than the ${what} before`, up: c > 0 };
  };

  // ---------- data ----------
  async function stored() {
    if (!ext) return { days: {}, hours: {} };
    try {
      await chrome.runtime.sendMessage("refresh");
    } catch {}
    const s = await chrome.storage.local.get(["days", "hours"]);
    return { days: s.days || {}, hours: s.hours || {} };
  }
  // visits per site in [s, e), plus the YouTube videos opened
  async function visits(s, e) {
    const out = { bySite: {}, total: 0, videos: new Set() };
    if (!ext || !chrome.history) return out;
    const items = await chrome.history.search({ text: "", startTime: s.getTime(), endTime: e.getTime(), maxResults: 5000 });
    const list = items.slice(0, 1500);
    for (let i = 0; i < list.length; i += 60) {
      const part = list.slice(i, i + 60);
      const all = await Promise.all(part.map((it) => chrome.history.getVisits({ url: it.url })));
      part.forEach((it, j) => {
        const n = all[j].filter((v) => v.visitTime >= s.getTime() && v.visitTime < e.getTime()).length;
        const h = hostOf(it.url);
        if (!h || !n) return;
        const k = site(h);
        out.bySite[k] = (out.bySite[k] || 0) + n;
        out.total += n;
        try {
          const u = new URL(it.url);
          if (/(^|\.)youtube\.com$/.test(u.hostname) && (u.pathname === "/watch" || u.pathname.startsWith("/shorts/"))) out.videos.add(u.searchParams.get("v") || u.pathname);
          if (u.hostname === "youtu.be") out.videos.add(u.pathname);
        } catch {}
      });
    }
    return out;
  }
  function timeIn(days, s, e) {
    const bySite = {}, perDay = [];
    let total = 0;
    for (const d of daysIn(s, e)) {
      const rec = days[dayKey(d)] || {};
      let t = 0;
      for (const [h, v] of Object.entries(rec)) {
        const k = site(h);
        bySite[k] = (bySite[k] || 0) + v;
        t += v;
      }
      perDay.push([d, t]);
      total += t;
    }
    return { bySite, perDay, total };
  }

  // ---------- drawing ----------
  function rows(box, list, fmt, empty) {
    box.replaceChildren();
    if (!list.length) return box.append(el("p", "empty", empty));
    const max = Math.max(list[0][1], 1);
    list.slice(0, 10).forEach(([h, v]) => {
      const r = el("div", "row2");
      const img = el("img");
      img.alt = "";
      img.src = favicon("https://" + h, 32);
      const bar = el("i");
      bar.style.setProperty("--w", Math.max(3, Math.round((v * 100) / max)) + "%");
      r.append(img, el("span", "", h), el("b", "", fmt(v)), bar);
      box.append(r);
    });
  }
  function chart(points, labelEvery, label, tip, nowIdx) {
    const box = $("chart");
    box.replaceChildren();
    const max = Math.max(...points.map((p) => p[1]), 60);
    points.forEach(([lab, v], i) => {
      const c = el("div", "c" + (i === nowIdx ? " now" : "") + (v ? "" : " z"));
      c.style.setProperty("--h", Math.round((v / max) * 100) + "%");
      c.style.setProperty("--i", i);
      c.title = `${tip(lab)}: ${dur(v)}`;
      const b = el("i");
      c.append(el("em", "", v ? dur(v) : ""), b);
      if (i % labelEvery === 0 || points.length <= 8) c.append(el("span", "", label(lab)));
      box.append(c);
    });
  }

  async function render() {
    localStorage.setItem("owl.rp", period);
    document.querySelectorAll("[data-p]").forEach((b) => {
      b.classList.toggle("on", b.dataset.p === period);
      b.setAttribute("aria-pressed", b.dataset.p === period);
    });
    $("next").disabled = offset >= 0;
    const [s, e] = bounds(period, offset), [ps, pe] = bounds(period, offset - 1);
    const what = period === "day" ? "day" : period === "week" ? "week" : "month";
    $("title").replaceChildren(
      document.createTextNode(
        offset === 0 ? (period === "day" ? "Today" : period === "week" ? "This week" : "This month") : offset === -1 ? (period === "day" ? "Yesterday" : period === "week" ? "Last week" : "Last month") : period === "month" ? s.toLocaleDateString("en", { month: "long", year: "numeric" }) : period === "day" ? s.toLocaleDateString("en", { weekday: "long", day: "numeric", month: "long" }) : "Week of " + s.toLocaleDateString("en", { day: "numeric", month: "long" })
      ),
      el("em", "", ".")
    );
    const last = new Date(e.getTime() - 1);
    $("range").textContent = period === "day" ? s.toLocaleDateString("en", { weekday: "long", day: "numeric", month: "long", year: "numeric" }) : `${s.toLocaleDateString("en", { day: "numeric", month: "long" })} to ${last.toLocaleDateString("en", { day: "numeric", month: "long", year: "numeric" })}`;

    const { days, hours } = await stored();
    const cur = timeIn(days, s, e), prev = timeIn(days, ps, pe);
    const [vis, pvis] = await Promise.all([visits(s, e), visits(ps, pe)]);
    const nDays = Math.max(1, Math.min(daysIn(s, e).length, Math.ceil((Math.min(Date.now(), e.getTime()) - s.getTime()) / DAY)));
    const sitesUsed = new Set([...Object.keys(cur.bySite), ...Object.keys(vis.bySite)]).size;

    const k1 = change(cur.total, prev.total, what), k2 = change(vis.total, pvis.total, what);
    const kp = [
      [dur(cur.total), "Time online", k1],
      [period === "day" ? (cur.total ? fmtHour((hours[dayKey(s)] || []).reduce((bi, v, i, a) => (v > a[bi] ? i : bi), 0)) : "-") : dur(cur.total / nDays), period === "day" ? "Busiest hour" : "Daily average", { t: "" }],
      [vis.total.toLocaleString(), "Page visits", k2],
      [String(sitesUsed), "Sites", { t: "" }],
    ];
    $("kpis").replaceChildren(
      ...kp.map(([b, l, c], i) => {
        const d = el("div");
        d.style.setProperty("--i", i);
        d.append(el("b", "", b), el("span", "", l));
        if (c.t) d.append(el("small", c.up ? "up" : "", c.t));
        return d;
      })
    );

    // chart: hours of the day, or days
    if (period === "day") {
      const hrs = hours[dayKey(s)] || new Array(24).fill(0);
      $("charth").textContent = "Time online by hour";
      chart(hrs.map((v, h) => [h, v]), 3, (h) => (clock12() ? `${h % 12 || 12}${h < 12 ? "a" : "p"}` : p2(h)), fmtHour, offset === 0 ? new Date().getHours() : -1);
    } else {
      $("charth").textContent = "Time online per day";
      const todayIdx = cur.perDay.findIndex(([d]) => dayKey(d) === dayKey());
      chart(cur.perDay, period === "week" ? 1 : 3, (d) => (period === "week" ? d.toLocaleDateString("en", { weekday: "short" }) : String(d.getDate())), (d) => d.toLocaleDateString("en", { weekday: "long", day: "numeric", month: "long" }), todayIdx);
    }
    $("chartn").textContent = cur.total ? "total " + dur(cur.total) : "no time counted yet";

    const byTime = Object.entries(cur.bySite).sort((a, b) => b[1] - a[1]);
    const byVisit = Object.entries(vis.bySite).sort((a, b) => b[1] - a[1]);
    rows($("bytime"), byTime, dur, "No time counted in this period yet. Time is counted from the day the extension was installed.");
    rows($("byvisit"), byVisit, (v) => v.toLocaleString(), "No visits in this period.");

    // spotlight on one site, YouTube first
    const choices = ["youtube.com", ...byTime.map((x) => x[0]), ...byVisit.map((x) => x[0])].filter((v, i, a) => a.indexOf(v) === i).slice(0, 6);
    if (!choices.includes(spotSite)) spotSite = "youtube.com";
    const t = cur.bySite[spotSite] || 0, pt = prev.bySite[spotSite] || 0, v = vis.bySite[spotSite] || 0;
    const name = spotSite === "youtube.com" ? "YouTube" : spotSite;
    const sp = $("spot");
    const h2 = el("h2");
    h2.append(el("span", "", "Spotlight"), el("span", "", name));
    const lines = [];
    if (spotSite === "youtube.com") lines.push(`${vis.videos.size.toLocaleString()} video${vis.videos.size === 1 ? "" : "s"} opened, ${v.toLocaleString()} visits.`);
    else lines.push(`${v.toLocaleString()} visit${v === 1 ? "" : "s"}.`);
    if (cur.total && t) lines.push(`That is ${Math.round((t / cur.total) * 100)}% of your time online${byTime[0] && byTime[0][0] === spotSite ? ", more than any other site" : ""}.`);
    const ch = change(t, pt, what);
    const pick = el("div", "pick");
    for (const c of choices) {
      const b = el("button", "ghost" + (c === spotSite ? " on" : ""), c === "youtube.com" ? "YouTube" : c);
      b.type = "button";
      b.onclick = () => {
        spotSite = c;
        localStorage.setItem("owl.spot", c);
        render();
      };
      pick.append(b);
    }
    sp.replaceChildren(h2, el("div", "big", dur(t)), el("p", "", lines.join(" ")), ...(ch.t ? [el("p", "sub", "Time: " + ch.t + ".")] : []), pick);

    // facts
    const facts = [];
    if (period !== "day") {
      const best = cur.perDay.reduce((b, x) => (x[1] > b[1] ? x : b), [null, 0]);
      if (best[0] && best[1]) facts.push(["Busiest day", `${best[0].toLocaleDateString("en", { weekday: "long", day: "numeric", month: "short" })}, ${dur(best[1])}`]);
      const hr = new Array(24).fill(0);
      for (const d of daysIn(s, e)) (hours[dayKey(d)] || []).forEach((x, i) => (hr[i] += x));
      const bh = hr.reduce((bi, x, i, a) => (x > a[bi] ? i : bi), 0);
      if (hr[bh]) facts.push(["Busiest hour", fmtHour(bh)]);
      const quiet = cur.perDay.filter(([d, x]) => !x && d.getTime() < Date.now()).length;
      if (cur.total) facts.push(["Days without the browser", String(quiet)]);
    }
    if (byTime[0]) facts.push(["Most time", `${byTime[0][0]}, ${dur(byTime[0][1])}`]);
    if (byVisit[0]) facts.push(["Most visits", `${byVisit[0][0]}, ${byVisit[0][1].toLocaleString()}`]);
    const newSites = Object.keys(vis.bySite).filter((k) => !pvis.bySite[k]).length;
    if (vis.total) facts.push([`Sites not visited the ${what} before`, String(newSites)]);
    if (vis.videos.size) facts.push(["YouTube videos opened", vis.videos.size.toLocaleString()]);
    $("facts").replaceChildren(
      ...(facts.length
        ? facts.map(([a, b]) => {
            const r = el("div", "fact");
            r.append(el("span", "", a), el("b", "", b));
            return r;
          })
        : [el("p", "empty", "Facts appear once there is some browsing in this period.")])
    );
  }

  document.querySelectorAll("[data-p]").forEach((b) =>
    b.addEventListener("click", () => {
      period = b.dataset.p;
      offset = 0;
      render();
    })
  );
  $("prev").onclick = () => {
    offset--;
    render();
  };
  $("next").onclick = () => {
    if (offset < 0) offset++;
    render();
  };
  document.addEventListener("keydown", (e) => {
    if (e.key === "ArrowLeft") $("prev").click();
    if (e.key === "ArrowRight" && offset < 0) $("next").click();
  });
  render();
})();
