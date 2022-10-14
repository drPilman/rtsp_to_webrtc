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

    document.getElementById('remoteVideo').appendChild(el)
}

pc.oniceconnectionstatechange = e => console.log(pc.iceConnectionState)
pc.onicecandidate = event => {
    if (event.candidate === null) {

        let local_descr = btoa(JSON.stringify(pc.localDescription))

        const params = new Proxy(new URLSearchParams(window.location.search), {
            get: (searchParams, prop) => searchParams.get(prop),
        });
        // Get the value of "id" in eg "https://example.com/?id=some_value"
        let value = parseInt(params.id);

        fetch("/api/view", {
            method: "post",
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },

            //make sure to serialize your JSON body
            body: JSON.stringify({
                session_description: local_descr,
                id: value
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
