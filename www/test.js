async function testPost() {
    const test_request = {
        lobby_id: document.getElementById("lobby_id").value,
        mission_id: 'foobar'
    };
    console.log(test_request);

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
    let lobby_id = document.getElementById("lobby_id").value;
	const socket = new WebSocket("ws://localhost:3000?lobby-id=" + lobby_id);
	socket.onopen = () => { console.log("foo"); };
	socket.onmessage = (ev) => { console.log(ev.data); }
}
