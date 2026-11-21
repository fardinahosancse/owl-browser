// Owl. Calm Home, background worker. Counts time online, all on this computer:
// the site in the focused tab, while you are not idle, per day and per site. Nothing is sent anywhere.
//
// storage.local:
//   days  { "YYYY-MM-DD": { host: seconds } }   kept 400 days, for the reports
//   daily { "YYYY-MM-DD": seconds }             total per day
//   hours { "YYYY-MM-DD": [24 x seconds] }      when in the day, kept 400 days
//   usage { host: seconds }                     all time, for the home page
//   cur   { host, since }                       the stretch that is running now

const IDLE_AFTER = 60; // seconds without input before the clock stops
const KEEP_DAYS = 400;

const dayKey = (d = new Date()) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
const hostOf = (u) => {
  try {
    const x = new URL(u);
    return x.protocol === "http:" || x.protocol === "https:" ? x.hostname.replace(/^www\./, "") : "";
  } catch {
    return "";
  }
};

// The worker can be stopped at any moment, so the running stretch lives in storage too.
let busy = Promise.resolve();
function flush(nextHost) {
  busy = busy.then(() => doFlush(nextHost)).catch(() => {});
  return busy;
}
async function doFlush(nextHost) {
  const now = Date.now();
  const st = await chrome.storage.local.get(["cur", "usage", "daily", "days", "hours"]);
  const usage = st.usage || {}, daily = st.daily || {}, days = st.days || {}, hours = st.hours || {};
  const cur = st.cur;
  if (cur && cur.host && cur.since) {
    // a lost event never counts hours: at most 15 minutes per stretch
    let from = Math.max(cur.since, now - 15 * 60_000);
    while (from < now) {
      // split at the hour so the per-hour and per-day numbers are right across midnight
      const d = new Date(from);
      const next = new Date(d.getFullYear(), d.getMonth(), d.getDate(), d.getHours() + 1).getTime();
      const to = Math.min(now, next);
      const secs = Math.round((to - from) / 1000);
      if (secs > 0) {
        const k = dayKey(d);
        usage[cur.host] = (usage[cur.host] || 0) + secs;
        daily[k] = (daily[k] || 0) + secs;
        (days[k] = days[k] || {})[cur.host] = (days[k][cur.host] || 0) + secs;
        (hours[k] = hours[k] || new Array(24).fill(0))[d.getHours()] += secs;
      }
      from = to;
    }
    for (const o of [daily, days, hours]) {
      const ks = Object.keys(o).sort();
      while (ks.length > KEEP_DAYS) delete o[ks.shift()];
    }
    const sites = Object.entries(usage).sort((a, b) => b[1] - a[1]);
    if (sites.length > 1000) for (const [h] of sites.slice(1000)) delete usage[h];
  }
  const next = nextHost === undefined ? (cur && cur.host) || "" : nextHost;
  await chrome.storage.local.set({ usage, daily, days, hours, cur: { host: next, since: next ? now : 0 } });
}

async function focusedHost() {
  try {
    if ((await chrome.idle.queryState(IDLE_AFTER)) !== "active") return "";
    const [tab] = await chrome.tabs.query({ active: true, lastFocusedWindow: true });
    const win = tab && (await chrome.windows.get(tab.windowId));
    return tab && win && win.focused ? hostOf(tab.url || "") : "";
  } catch {
    return "";
  }
}

const follow = async () => flush(await focusedHost());
chrome.tabs.onActivated.addListener(follow);
chrome.tabs.onUpdated.addListener((_id, info, tab) => {
  if (tab.active && (info.url || info.status === "complete")) follow();
});
chrome.windows.onFocusChanged.addListener(follow);
chrome.idle.setDetectionInterval(IDLE_AFTER);
chrome.idle.onStateChanged.addListener(follow);

chrome.alarms.create("tick", { periodInMinutes: 1 });
chrome.alarms.onAlarm.addListener((a) => {
  if (a.name === "tick") follow();
});

// pages ask for fresh numbers when they open
chrome.runtime.onMessage.addListener((msg, _sender, reply) => {
  if (msg === "refresh") {
    follow().then(() => reply(true));
    return true;
  }
});

// 1.0 blocked trackers; 1.1 does not, so its old counters are removed once
chrome.runtime.onInstalled.addListener(() => {
  chrome.storage.local.remove(["kickSeen", "kickDay", "kickToday", "kickTotal", "kickOn"]);
});
