async function testPost() {
    const response = await fetch('http://localhost:3000/create_lobby', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ test: 42 })
    });
    return response.json()
}
