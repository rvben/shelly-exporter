#!/usr/bin/env python3
"""Mock Shelly device server for testing the exporter"""

from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import time

class ShellyMockHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/rpc/Shelly.GetDeviceInfo":
            # Gen2 device info
            response = {
                "name": "Mock Shelly Plus 1PM",
                "id": "shellyplus1pm-a8032abc1234",
                "mac": "A8:03:2A:BC:12:34",
                "model": "SNSW-001P16EU",
                "generation": 2,
                "fw_id": "20231107-164514/1.0.8-g8c7bb8d",
                "ver": "1.0.8",
                "app": "Plus1PM",
                "auth_en": False,
                "auth_domain": None
            }
            self.send_json_response(response)
            
        elif self.path == "/rpc/Shelly.GetStatus":
            # Gen2 status
            response = {
                "switch:0": {
                    "id": 0,
                    "source": "manual",
                    "output": True,
                    "apower": 125.7,
                    "voltage": 229.8,
                    "current": 0.547,
                    "freq": 50.0,
                    "pf": 0.99,
                    "aenergy": {
                        "total": 2845.123,
                        "by_minute": [2084.333, 2085.667, 2087.000],
                        "minute_ts": int(time.time())
                    },
                    "temperature": {
                        "tC": 42.3,
                        "tF": 108.1
                    }
                },
                "sys": {
                    "mac": "A8:03:2A:BC:12:34",
                    "restart_required": False,
                    "time": "23:45",
                    "unixtime": int(time.time()),
                    "uptime": 86400,
                    "ram_size": 262144,
                    "ram_free": 157890,
                    "fs_size": 524288,
                    "fs_free": 380234,
                    "cfg_rev": 12,
                    "available_updates": None
                },
                "wifi": {
                    "sta_ip": "192.168.1.100",
                    "status": "got ip",
                    "ssid": "TestNetwork",
                    "rssi": -67
                }
            }
            self.send_json_response(response)
            
        elif self.path == "/settings":
            # Gen1 fallback
            self.send_error(404)
            
        else:
            self.send_error(404)
    
    def send_json_response(self, data):
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())
    
    def log_message(self, format, *args):
        """Suppress log messages"""
        pass

if __name__ == "__main__":
    server = HTTPServer(('localhost', 8888), ShellyMockHandler)
    print("Mock Shelly device running on http://localhost:8888")
    print("Press Ctrl-C to stop")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.shutdown()