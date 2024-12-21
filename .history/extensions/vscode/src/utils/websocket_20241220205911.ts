import WebSocket from 'ws';

export class WebSocketClient {
    private ws: WebSocket | undefined;
    private onProgressCallback: (progress: number) => void;

    constructor(url: string, onProgress: (progress: number) => void) {
        this.onProgressCallback = onProgress;
        this.connect(url);
    }

    private connect(url: string) {
        this.ws = new WebSocket(url);

        this.ws.on('open', () => {
            console.log('WebSocket connected');
        });

        this.ws.on('message', (data: string) => {
            try {
                const message = JSON.parse(data);
                if (message.progress !== undefined) {
                    this.onProgressCallback(message.progress);
                }
            } catch (error) {
                console.error('Failed to parse WebSocket message:', error);
            }
        });

        this.ws.on('error', (error: Error) => {
            console.error('WebSocket error:', error);
        });

        this.ws.on('close', () => {
            console.log('WebSocket disconnected');
        });
    }

    public disconnect() {
        if (this.ws) {
            this.ws.close();
            this.ws = undefined;
        }
    }

    public isConnected(): boolean {
        return this.ws?.readyState === WebSocket.OPEN;
    }
} 