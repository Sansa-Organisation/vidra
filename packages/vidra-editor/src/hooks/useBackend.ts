// useBackend — WebSocket connection + REST client for the editor backend (Task 9.2)
import { useState, useEffect, useRef, useCallback } from 'react';

export interface ProjectMeta {
    width: number;
    height: number;
    fps: number;
    total_frames: number;
}

export function useBackend(port = 3001) {
    const [connected, setConnected] = useState(false);
    const [meta, setMeta] = useState<ProjectMeta | null>(null);
    const [error, setError] = useState<string | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const onFrameRef = useRef<((data: ArrayBuffer) => void) | null>(null);

    const baseUrl = `http://127.0.0.1:${port}`;

    useEffect(() => {
        let reconnectTimer: ReturnType<typeof setTimeout>;
        function connect() {
            const ws = new WebSocket(`ws://127.0.0.1:${port}/ws`);
            ws.binaryType = 'arraybuffer';
            wsRef.current = ws;

            ws.onopen = () => { setConnected(true); setError(null); };
            ws.onclose = () => {
                setConnected(false);
                reconnectTimer = setTimeout(connect, 2000);
            };
            ws.onmessage = (ev) => {
                if (typeof ev.data === 'string') {
                    const msg = JSON.parse(ev.data);
                    if (msg.type === 'METADATA') {
                        setMeta(msg as ProjectMeta);
                        setError(null);
                    } else if (msg.type === 'ERROR') {
                        setError(msg.message);
                    }
                } else {
                    onFrameRef.current?.(ev.data);
                }
            };
        }
        connect();
        return () => {
            clearTimeout(reconnectTimer);
            wsRef.current?.close();
        };
    }, [port]);

    const requestFrame = useCallback((frame: number) => {
        if (wsRef.current?.readyState === 1) {
            wsRef.current.send(JSON.stringify({ type: 'REQUEST_FRAME', frame }));
        }
    }, []);

    const setFrameCallback = useCallback((cb: (data: ArrayBuffer) => void) => {
        onFrameRef.current = cb;
    }, []);

    const rest = useCallback(async (method: string, path: string, body?: unknown) => {
        const res = await fetch(`${baseUrl}${path}`, {
            method,
            headers: body ? { 'Content-Type': 'application/json' } : undefined,
            body: body ? JSON.stringify(body) : undefined,
        });
        return res.json();
    }, [baseUrl]);

    return { connected, meta, error, requestFrame, setFrameCallback, rest, setError };
}
