import WebSocket from 'isomorphic-ws';

interface ProgressMessage {
    progress: number;
}

export class WebSocketClient {
    private ws: WebSocket | undefined;
    private readonly onProgressCallback: (progress: number) => void;

    constructor(url: string, onProgress: (progress: number) => void) {
        this.onProgressCallback = onProgress;
        this.connect(url);
    }

    private connect(url: string): void {
        this.ws = new WebSocket(url);

        this.ws.onopen = () => {
            console.log('WebSocket connected');
        };

        this.ws.onmessage = (event: WebSocket.MessageEvent) => {
            try {
                const message = JSON.parse(event.data.toString()) as ProgressMessage;
                if (typeof message.progress === 'number') {
                    this.onProgressCallback(message.progress);
                }
            } catch (error) {
                console.error('Failed to parse WebSocket message:', error);
            }
        };

        this.ws.onerror = (event: WebSocket.ErrorEvent) => {
            console.error('WebSocket error:', event.error);
        };

        this.ws.onclose = () => {
            console.log('WebSocket disconnected');
        };
    }

    public disconnect(): void {
        if (this.ws) {
            this.ws.close();
            this.ws = undefined;
        }
    }

    public isConnected(): boolean {
        return this.ws?.readyState === WebSocket.OPEN;
    }
} 