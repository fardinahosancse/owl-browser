# Small HTTP server for scripts/smoke-test.ps1: serves a test page, a PNG and a large file with
# byte-range support (so Owl's fast download engine is used). Prints the large file's SHA-256.
import hashlib, http.server, re, struct, sys, zlib

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8799
BIG = bytes((i * 31 + i // 4096) % 251 for i in range(6 * 1024 * 1024))
print(hashlib.sha256(BIG).hexdigest(), flush=True)

def png(w=64, h=48):
    raw = b"".join(b"\x00" + bytes(c for x in range(w) for c in ((x * 4) % 256, (y * 5) % 256, 120)) for y in range(h))
    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")

PNG = png()
PAGE = b"""<!doctype html><title>Smoke test</title><h1>Owl smoke test</h1><img src="/pic.png" width=64 height=48>
<script>document.cookie='owltest=1; max-age=3600';try{localStorage.setItem('owltest','1')}catch(e){}</script>"""

class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass
    def send(self, code, ctype, body, extra=()):
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        for k, v in extra:
            self.send_header(k, v)
        self.end_headers()
        try:
            self.wfile.write(body)
        except Exception:
            pass
    def do_GET(self):
        if self.path.startswith("/page.html"):
            return self.send(200, "text/html", PAGE)
        if self.path.startswith("/pic.png"):
            return self.send(200, "image/png", PNG)
        if self.path.startswith("/big.bin"):
            n = len(BIG)
            m = re.match(r"bytes=(\d+)-(\d*)", self.headers.get("Range") or "")
            if m:
                a = int(m[1]); b = min(int(m[2]) if m[2] else n - 1, n - 1)
                return self.send(206, "application/octet-stream", BIG[a:b + 1], [("Content-Range", f"bytes {a}-{b}/{n}"), ("Accept-Ranges", "bytes")])
            return self.send(200, "application/octet-stream", BIG, [("Accept-Ranges", "bytes"), ("Content-Disposition", 'attachment; filename="big.bin"')])
        self.send(404, "text/plain", b"not found")

http.server.ThreadingHTTPServer(("127.0.0.1", PORT), H).serve_forever()
