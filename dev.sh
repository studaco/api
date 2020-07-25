source .env
systemfd --no-pid -s http::$PORT -- cargo watch -x run