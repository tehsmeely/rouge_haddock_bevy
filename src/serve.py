# Python 3 server example
from http.server import BaseHTTPRequestHandler, HTTPServer
import time

hostName = "localhost"
serverPort = 8080



class MyServer(BaseHTTPRequestHandler):
    def do_GET(self):
        print("Got request for path:", self.path)

        fname = None
        if self.path == "/":
            fname = "index.html"
        elif self.path[:5] == "/out/":
            fname = self.path[1:]

        if fname is not None:
            print("Serving file", fname)
            self.send_response(200)

            ext = fname.split('.')[-1]
            if ext == "js":
                content_type = "text/javascript"
            elif ext == "html":
                content_type = "text/html"
            elif ext == "wasm":
                content_type = "application/wasm"
            else:
                print("WARN: unknown ext for content type: {}".format(ext))
                content_type = "text/plain"

            self.send_header("Content-type", content_type)
            self.end_headers()

            with open(fname, "rb") as f:
                self.wfile.write(f.read())
        else:
            self.send_response(404)


if __name__ == "__main__":
    webServer = HTTPServer((hostName, serverPort), MyServer)
    print("Server started http://%s:%s" % (hostName, serverPort))

    try:
        webServer.serve_forever()
    except KeyboardInterrupt:
        pass

    webServer.server_close()
    print("Server stopped.")