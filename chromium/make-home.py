"""Turns Owl.'s classic home page (written by the ignored test `dump_home_for_chromium`) into the
standalone home page the Chromium build opens: the same markup and design, with the font embedded,
no app connection, and Owl.'s own pages pointed at Chromium's built-in ones.

    OWL_CEF_HOME=chromium/home.raw.html cargo test --release dump_home_for_chromium -- --ignored
    python chromium/make-home.py
"""
import base64, os, re

here = os.path.dirname(os.path.abspath(__file__))
raw = open(os.path.join(here, "home.raw.html"), encoding="utf-8").read()
font = base64.b64encode(open(os.path.join(here, "..", "assets", "fonts", "SpaceGrotesk-latin.woff2"), "rb").read()).decode()

html = re.sub(r'<meta http-equiv="Content-Security-Policy"[^>]*>', "", raw, count=1)
html = re.sub(r" nonce=[0-9a-f]+", "", html)
html = html.replace("url(/font/space-grotesk.woff2)", f"url(data:font/woff2;base64,{font})")
# Owl.'s own pages become Chromium's built-in ones in this build
for owl, chrome in {
    "owl://settings": "chrome://settings",
    "owl://downloads": "chrome://downloads",
    "owl://passwords": "chrome://password-manager",
    "owl://history": "chrome://history",
    "owl://bookmarks": "chrome://bookmarks",
    "owl://about": "chrome://version",
}.items():
    html = html.replace(owl, chrome)
# there is no Owl. app behind this page: messages to it are dropped
html = html.replace("<head>", "<head>", 1)
html = html.replace("<meta charset=utf-8>", "<meta charset=utf-8><script>window.ipc={postMessage:function(){}}</script>", 1)
open(os.path.join(here, "home.html"), "w", encoding="utf-8", newline="").write(html)
print("home.html", len(html), "bytes")
