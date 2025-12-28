// import type { ServerMsg } from './bindings/ServerMsg';

function handleMessage(msg) {
    switch (msg.type) {
        case 'NewSnapshot':
            console.log(msg.data);
            break;
    }
}

const ws = new WebSocket('ws://ws');
ws.onopen = () => console.log('✅ Connected');
ws.onmessage = (event) => handleMessage(JSON.parse(event.data));
ws.onerror = () => console.log('❌ Error');
ws.onclose = () => console.log('🔌 Disconnected');