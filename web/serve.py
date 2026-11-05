#!/usr/bin/env python3
"""Static file server for the wasm build with COOP/COEP headers.

SharedArrayBuffer (engine worker) and wasm-bindgen-rayon both require these
headers; plain `python -m http.server` cannot set them.

Usage: python web/serve.py [port]
"""
import http.server
import sys

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8123


class Handler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        # Dev server: never let the browser cache wasm/js/worker scripts.
        self.send_header("Cache-Control", "no-store")
        super().end_headers()


if __name__ == "__main__":
    print(f"serving ./web at http://localhost:{PORT} (COOP/COEP enabled)")
    http.server.ThreadingHTTPServer(
        ("127.0.0.1", PORT), Handler
    ).serve_forever()
