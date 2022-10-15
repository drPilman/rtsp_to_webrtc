# RTCP TO Webrtc


```bash
./prod.sh
```

```bash
curl -d '{"url":"rtsp://wowzaec2demo.streamlock.net/vod/mp4:BigBuckBunny_115k.mp4", "token":"123888"}' -H "Content-Type: application/json" -X POST http://localhost:8080/api/add_source
```

open localhost:8080


### stop 
```bash
curl -d '{"id":0 , "token":"123888"}' -H "Content-Type: application/json" -X POST http://localhost:8080/api/stop_source
```
