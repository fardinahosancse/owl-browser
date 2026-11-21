// Owl. Calm Home, background worker. Keeps the numbers the home page shows, all on this computer:
//  - time online per day and per site: the site in the focused tab, while you are not idle;
//  - OwlKick: how many requests the tracker rules blocked, today and in all.
// Nothing is ever sent anywhere.

const IDLE_AFTER = 60; // seconds without input before the clock stops

const today = () => {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
};
const hostOf = (u) => {
  try {
    const x = new URL(u);
    return x.protocol === "http:" || x.protocol === "https:" ? x.hostname.replace(/^www\./, "") : "";
  } catch {
    return "";
  }
};

// ---------- time online ----------
// The worker can be stopped at any moment, so the running stretch lives in storage too.
async function flush(nextHost) {
  const now = Date.now();
  const { cur, usage = {}, daily = {} } = await chrome.storage.local.get(["cur", "usage", "daily"]);
  if (cur && cur.host && cur.since) {
    const secs = Math.min(Math.round((now - cur.since) / 1000), 15 * 60); // a lost event never counts hours
    if (secs > 0) {
      usage[cur.host] = (usage[cur.host] || 0) + secs;
      const d = today();
      daily[d] = (daily[d] || 0) + secs;
      // keep 60 days and 500 sites
      const days = Object.keys(daily).sort();
      while (days.length > 60) delete daily[days.shift()];
      const sites = Object.entries(usage).sort((a, b) => b[1] - a[1]);
      if (sites.length > 500) for (const [h] of sites.slice(500)) delete usage[h];
    }
  }
  const next = nextHost === undefined ? (cur && cur.host) || "" : nextHost;
  await chrome.storage.local.set({ usage, daily, cur: { host: next, since: next ? now : 0 } });
}

async function focusedHost() {
  try {
    const state = await chrome.idle.queryState(IDLE_AFTER);
    if (state !== "active") return "";
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

// ---------- OwlKick ----------
// Matched rules are only kept by the browser for a few minutes, so they are counted every minute.
async function countBlocks() {
  try {
    const { kickSeen = Date.now() - 60_000, kickDay = today(), kickToday = 0, kickTotal = 0 } = await chrome.storage.local.get(["kickSeen", "kickDay", "kickToday", "kickTotal"]);
    const { rulesMatchedInfo } = await chrome.declarativeNetRequest.getMatchedRules({ minTimeStamp: kickSeen + 1 });
    const n = rulesMatchedInfo.length;
    const last = rulesMatchedInfo.reduce((m, r) => Math.max(m, r.timeStamp), kickSeen);
    const d = today();
    await chrome.storage.local.set({
      kickSeen: last,
      kickDay: d,
      kickToday: (kickDay === d ? kickToday : 0) + n,
      kickTotal: kickTotal + n,
    });
  } catch {
    // rate limited or not allowed: try again next minute
  }
}

chrome.alarms.create("tick", { periodInMinutes: 1 });
chrome.alarms.onAlarm.addListener(async (a) => {
  if (a.name !== "tick") return;
  await follow();
  await countBlocks();
});

// the home page asks for fresh numbers when it opens
chrome.runtime.onMessage.addListener((msg, _sender, reply) => {
  if (msg === "refresh") {
    Promise.all([follow(), countBlocks()]).then(() => reply(true));
    return true;
  }
  if (msg && msg.kick !== undefined) {
    chrome.declarativeNetRequest
      .updateEnabledRulesets(msg.kick ? { enableRulesetIds: ["owlkick"] } : { disableRulesetIds: ["owlkick"] })
      .then(() => chrome.storage.local.set({ kickOn: !!msg.kick }))
      .then(() => reply(true));
    return true;
  }
});

// show the number blocked on each tab on the toolbar icon
chrome.runtime.onInstalled.addListener(() => {
  chrome.declarativeNetRequest.setExtensionActionOptions({ displayActionCountAsBadgeText: true }).catch(() => {});
  chrome.storage.local.get("kickOn").then(({ kickOn }) => {
    if (kickOn === undefined) chrome.storage.local.set({ kickOn: true });
  });
});
