async function testPost() {
    const test_request = {
        lobby_id: 'my-lobby',
        mission_id: 'foobar'
    };

    const response = await fetch('http://localhost:3000/create_lobby', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(test_request)
    });
    return response.json()
}

async function testWebSocket() {
	const socket = new WebSocket("ws://localhost:3000?lobby-id=my-lobby");
	socket.onopen = () => { console.log("foo"); };
	socket.onmessage = (ev) => { console.log(ev.data); }
}
