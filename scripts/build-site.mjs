// Builds the landing page in site/: takes the newest released version from CHANGELOG.md and writes it
// into site/index.html (the scrolling banner at the top and the "New in" section) and site/release.json.
// Vercel runs this on every deploy; it needs only Node and no packages.
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = "https://github.com/fardinahosancse/owl-browser";
const md = readFileSync(join(root, "CHANGELOG.md"), "utf8").replace(/\r\n/g, "\n");

// first "## [x.y.z] - date" section (Unreleased is skipped: only shipped versions appear on the site)
const m = md.match(/^## \[(\d+\.\d+\.\d+[^\]]*)\] - (\d{4}-\d{2}-\d{2})\n([\s\S]*?)(?=^## \[|\n\[[^\]]+\]: |$(?![\s\S]))/m);
if (!m) {
  console.log("No released version in CHANGELOG.md; leaving the page as it is.");
  process.exit(0);
}
const [, version, date, body] = m;

const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
const plain = (s) => s.replace(/\*\*(.+?)\*\*/g, "$1").replace(/`([^`]+)`/g, "$1").replace(/\[([^\]]+)\]\([^)]+\)/g, "$1").trim();

const sections = [];
let cur = null;
for (const line of body.split("\n")) {
  const h = line.match(/^### (.+)/);
  if (h) {
    cur = { title: h[1].trim(), items: [] };
    sections.push(cur);
    continue;
  }
  const b = line.match(/^- (.+)/);
  if (b && cur) cur.items.push(b[1]);
  else if (cur && cur.items.length && /^\s{2,}\S/.test(line)) cur.items[cur.items.length - 1] += " " + line.trim();
}

// an item "**Name**: text" becomes {name, text}; otherwise the first clause is the name
const split = (raw) => {
  const bold = raw.match(/^\*\*(.+?)\*\*:?\s*(.*)$/);
  if (bold) return { name: plain(bold[1]), text: plain(bold[2]) };
  const t = plain(raw);
  const cut = t.search(/[:.;]/);
  return cut > 0 && cut < 70 ? { name: t.slice(0, cut), text: t.slice(cut + 1).trim() } : { name: t, text: "" };
};
const cap = (x) => ({ name: x.name, text: x.text ? x.text.charAt(0).toUpperCase() + x.text.slice(1) : "" });
const added0 = (sections.find((s) => /^added$/i.test(s.title)) || sections[0] || { items: [] }).items.map(split);
const added = added0.map(cap);
const pretty = new Date(date + "T12:00:00Z").toLocaleDateString("en-GB", { day: "numeric", month: "long", year: "numeric", timeZone: "UTC" });
const releaseUrl = `${repo}/releases/tag/v${version}`;

// ---- scrolling banner ----
const highlights = added.slice(0, 8).map((x) => x.name);
const tickText = [`Owl ${version} is out`, pretty, ...highlights].map(esc).join(" <i>/</i> ");
const banner = `<a class="tick" href="${releaseUrl}" aria-label="Owl ${esc(version)}, released ${esc(pretty)}. Read the release notes."><span class="tk-clip"><span class="tk-in"><span>${tickText} <i>/</i> <b>Release notes</b></span><span aria-hidden="true">${tickText} <i>/</i> <b>Release notes</b></span></span></span></a>`;

// ---- "New in" section ----
const rows = added
  .slice(0, 9)
  .map((x) => `<li class="rv"><h3>${esc(x.name)}</h3>${x.text ? `<p>${esc(x.text.length > 260 ? x.text.slice(0, 257).replace(/\s\S*$/, "") + "..." : x.text)}</p>` : ""}</li>`)
  .join("\n    ");
const other = sections
  .filter((s) => !/^added$/i.test(s.title) && !/known issues/i.test(s.title))
  .map((s) => `${s.items.length} ${s.title.toLowerCase()}`)
  .join(", ");
const known = (sections.find((s) => /known issues/i.test(s.title)) || { items: [] }).items.length;
const news = `<section class="news" id="new" aria-labelledby="h-new">
  <div class="nh rv"><h2 id="h-new">New in ${esc(version)}</h2><p>Released ${esc(pretty)}.${other ? ` Also ${esc(other)}.` : ""}${known ? ` ${known} known issues are listed in the notes.` : ""}</p><a class="link" href="${releaseUrl}">Full release notes</a></div>
  <ul class="nl">
    ${rows}
  </ul>
</section>`;

const page = join(root, "site", "index.html");
let html = readFileSync(page, "utf8");
const put = (name, content) => {
  const re = new RegExp(`(<!-- release:${name} -->)[\\s\\S]*?(<!-- /release:${name} -->)`);
  if (!re.test(html)) throw new Error(`marker release:${name} missing in site/index.html`);
  html = html.replace(re, `$1\n${content}\n$2`);
};
put("banner", banner);
put("news", news);
html = html.replace(/(<meta name="owl-version" content=")[^"]*(")/, `$1${version}$2`);
html = html.replace(/(<!-- release:version -->)[^<]*(<!-- \/release:version -->)/g, `$1${esc(version)}$2`);
writeFileSync(page, html);
writeFileSync(join(root, "site", "release.json"), JSON.stringify({ version, date, url: releaseUrl, added, sections }, null, 2));
console.log(`Landing page updated to Owl ${version} (${date}), ${added.length} highlights.`);
