#!/usr/bin/env python3
"""
Mock JSON-RPC 2.0 stdio client simulating Cursor / VS Code MCP connection to QubitCore daemon.
Sends standard MCP initialize and list_roots calls and verifies valid responses.
"""
import sys
import json
import subprocess
import time

def send_request(proc, req_id, method, params=None):
    payload = {
        "jsonrpc": "2.0",
        "id": req_id,
        "method": method,
        "params": params or {}
    }
    line = json.dumps(payload) + "\n"
    proc.stdin.write(line.encode("utf-8"))
    proc.stdin.flush()

def read_response(proc, timeout=5.0):
    start = time.time()
    while time.time() - start < timeout:
        line = proc.stdout.readline()
        if line:
            return json.loads(line.decode("utf-8"))
        time.sleep(0.05)
    return None

def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <path_to_qubit_cli_or_daemon>")
        sys.exit(1)

    cmd = sys.argv[1:]
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    try:
        # 1. Initialize
        print("[>] Sending 'initialize' request...")
        send_request(proc, 1, "initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {"roots": {"listChanged": True}},
            "clientInfo": {"name": "CursorMockClient", "version": "0.1.0"}
        })
        resp = read_response(proc)
        print(f"[<] Response: {resp}")
        assert resp and resp.get("id") == 1, "Initialize failed"

        # 2. List Roots
        print("[>] Sending 'roots/list' request...")
        send_request(proc, 2, "roots/list", {})
        resp = read_response(proc)
        print(f"[<] Response: {resp}")
        assert resp and resp.get("id") == 2, "List roots failed"

        print("[+] Mock client test passed successfully!")
    finally:
        proc.terminate()
        proc.wait()

if __name__ == "__main__":
    main()
