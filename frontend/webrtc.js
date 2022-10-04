/* eslint-env browser */

let pc = new RTCPeerConnection({
    iceServers: [
        {
            urls: 'stun:stun.l.google.com:19302'
        }
    ]
})

pc.ontrack = function (event) {
    var el = document.createElement(event.track.kind)
    el.srcObject = event.streams[0]
    el.autoplay = true
    el.controls = true

    document.getElementById('remoteVideos').appendChild(el)
}

pc.oniceconnectionstatechange = e => console.log(pc.iceConnectionState)
pc.onicecandidate = event => {
    if (event.candidate === null) {
        let local_descr = btoa(JSON.stringify(pc.localDescription))
        fetch("/api/view", {
            method: "post",
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },

            //make sure to serialize your JSON body
            body: JSON.stringify({
                session_description: local_descr,
                id: 0
            })
        })
            .then((response) => response.json())
            .then((data) => {
                console.log(data);
                try {
                    pc.setRemoteDescription(new RTCSessionDescription(JSON.parse(atob(data.session_description))))
                } catch (e) {
                    alert(e)
                }

            });
    }
}

// Offer to receive 1 audio, and 2 video tracks
pc.addTransceiver('audio', { 'direction': 'recvonly' })
pc.addTransceiver('video', { 'direction': 'recvonly' })
pc.addTransceiver('video', { 'direction': 'recvonly' })
pc.createOffer().then(d => pc.setLocalDescription(d)).catch(console.log)

window.startSession = () => {
    let sd = document.getElementById('remoteSessionDescription').value
    if (sd === '') {
        return alert('Session Description must not be empty')
    }

    try {
        pc.setRemoteDescription(new RTCSessionDescription(JSON.parse(atob(sd))))
    } catch (e) {
        alert(e)
    }
}
