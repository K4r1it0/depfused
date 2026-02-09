#!/usr/bin/env python3
"""Serve all test labs on different ports for depfused testing."""

import http.server
import socketserver
import threading
import os
import sys
import signal

LABS = {
    "webpack5-react": {"port": 9001, "dir": "webpack5-react/dist"},
    "vite-vue":       {"port": 9002, "dir": "vite-vue/dist"},
    "parcel-react":   {"port": 9003, "dir": "parcel-react/dist"},
    "esbuild-app":    {"port": 9004, "dir": "esbuild-app/dist/browser"},
    "rollup-library": {"port": 9005, "dir": "rollup-library/dist/browser"},
    "swc-app":        {"port": 9006, "dir": "swc-app/dist/browser"},
    "angular-app":    {"port": 9007, "dir": "angular-app/dist/angular-app/browser"},
    "obfuscated":     {"port": 9008, "dir": "obfuscated/dist"},
    "nextjs-app":     {"port": 9009, "dir": "nextjs-app/.next/static"},
}

BASE = os.path.dirname(os.path.abspath(__file__))
servers = []

class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, format, *args):
        pass  # Suppress logs

def serve(name, port, directory):
    full_dir = os.path.join(BASE, directory)
    if not os.path.exists(full_dir):
        print(f"  SKIP {name}: {full_dir} not found")
        return
    handler = lambda *args, **kwargs: QuietHandler(*args, directory=full_dir, **kwargs)
    with socketserver.TCPServer(("", port), handler) as httpd:
        httpd.allow_reuse_address = True
        servers.append(httpd)
        print(f"  {name:20s} -> http://localhost:{port}  ({directory})")
        httpd.serve_forever()

def main():
    print("Starting test lab servers...")
    threads = []
    for name, cfg in LABS.items():
        t = threading.Thread(target=serve, args=(name, cfg["port"], cfg["dir"]), daemon=True)
        t.start()
        threads.append(t)

    print(f"\n  All {len(LABS)} labs serving. Press Ctrl+C to stop.\n")

    try:
        signal.pause()
    except KeyboardInterrupt:
        print("\nShutting down...")
        for s in servers:
            s.shutdown()

if __name__ == "__main__":
    main()
