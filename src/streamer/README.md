# How to make streamer running locally

1. You need to install GStreamer on your PC
   1. Window - https://gstreamer.freedesktop.org/data/pkg/windows/
   2. Linux - these libraries has to part of future Dockerfile container with Rust app!
```
apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
      gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
      gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
      gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev \
      gstreamer1.0-tools 
```
2. Start docker.
3. Run `docker-compose up` in source directory of the project.
4. Then you start GStreamer pipeline using. Don't forget to modify source file and rtmp server target.
```
gst-launch-1.0 -e filesrc location=video_resources/video2.mp4 ! decodebin name=d ! queue ! videoconvert ! \
x264enc bitrate=1000 tune=zerolatency ! video/x-h264 ! h264parse ! video/x-h264 ! flvmux name=mux streamable=true \
! queue ! rtmpsink location='rtmp://localhost/hls/stream-<id>' \
d. ! queue ! audioconvert ! audioresample ! audio/x-raw,rate=48000 ! voaacenc bitrate=96000 ! audio/mpeg \
! aacparse ! audio/mpeg, mpegversion=4 ! mux.
```

5. It starts sending RTMP stream to Nginx, where it's converted to HLS stream, which can be then fetched using HTTP protocol
by some JavaScript player.
6. For testing, you can use e.g. - https://livepush.io/hlsplayer/index.html, as stream link insert e.g.
`http://localhost:8080/hls/stream-<id>.m3u8`.
7. If you want to check that Nginx creates `.m3u8` files, you can open `http://localhost:8080/hls` - after each refresh,
size of `*.ts` files should change.