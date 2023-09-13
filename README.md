# Http_server demo

This is a demonstration of a basic HTTP server that utilizes a RPC-based mini_redis as its backend. Here's an architectural overview of the setup:
![](images/README/2023-09-13-12-36-38.png#pic)

# Usage

First build and run the mini_redis:
```
cd mini_redis
cargo build
./target/debug/mini_redis
```

Then run the http-server:
```
cd ..
cargo run
```

The server will run at 127.0.0.1:3000 by default. Use a browser to visit it. Here are some examples:
![](images/README/2023-09-13-12-37-16.png#pic)

![](images/README/2023-09-13-12-37-34.png#pic)

![](images/README/2023-09-13-12-39-15.png#pic)

![](images/README/2023-09-13-12-42-05.png#pic)

WARNING: the mini_redis is ran over a filter which will block all requests include "Genshin", let's try it!

![](images/README/2023-09-13-12-43-06.png#pic)

Also, you can use the requsest cli to visit the server:

```
cargo run --bin request -- get/set/del/ping [args]
```
 
examples:
![](images/README/2023-09-13-19-08-12.png#pic)