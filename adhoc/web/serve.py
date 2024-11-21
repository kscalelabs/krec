import http.server
from http.server import SimpleHTTPRequestHandler
from socketserver import TCPServer

class CORSRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        # Add CORS headers
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        # Add required headers for SharedArrayBuffer
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        super().end_headers()

# Set the server to listen on localhost and port 8000
HOST = "127.0.0.1"
PORT = 8001

if __name__ == "__main__":
    with TCPServer((HOST, PORT), CORSRequestHandler) as httpd:
        print(f"Serving on http://{HOST}:{PORT}")
        httpd.serve_forever()

