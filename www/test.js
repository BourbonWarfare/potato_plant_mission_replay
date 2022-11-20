async function testPost() {
    const test_request = {
        lobby_name: 'my-lobby',
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
