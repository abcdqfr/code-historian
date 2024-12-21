import WebSocket from 'ws';

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

        this.ws.on('open', () => {
            console.log('WebSocket connected');
        });

        this.ws.on('message', (data: WebSocket.Data) => {
            try {
                const message = JSON.parse(data.toString()) as ProgressMessage;
                if (typeof message.progress === 'number') {
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