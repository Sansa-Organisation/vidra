// Canvas panel — renders frames via WebSocket (Task 9.4)
import { useRef, useEffect, useCallback } from 'react';
import { useProjectStore } from '../hooks/useProject';

interface CanvasPanelProps {
    meta: { width: number; height: number; fps: number; total_frames: number } | null;
    requestFrame: (f: number) => void;
    setFrameCallback: (cb: (data: ArrayBuffer) => void) => void;
}

export function CanvasPanel({ meta, requestFrame, setFrameCallback }: CanvasPanelProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const frame = useProjectStore(s => s.frame);
    const setFrame = useProjectStore(s => s.setFrame);

    useEffect(() => {
        if (!canvasRef.current) return;
        const ctx = canvasRef.current.getContext('2d');
        if (!ctx) return;

        setFrameCallback((data: ArrayBuffer) => {
            const blob = new Blob([data], { type: 'image/jpeg' });
            createImageBitmap(blob).then(bmp => {
                ctx.drawImage(bmp, 0, 0);
            });
        });
    }, [setFrameCallback]);

    useEffect(() => {
        if (meta) requestFrame(frame);
    }, [frame, meta, requestFrame]);

    const onKeyDown = useCallback((e: KeyboardEvent) => {
        if (!meta) return;
        if (e.key === 'ArrowRight') setFrame(Math.min(frame + 1, meta.total_frames - 1));
        if (e.key === 'ArrowLeft') setFrame(Math.max(frame - 1, 0));
    }, [frame, meta, setFrame]);

    useEffect(() => {
        window.addEventListener('keydown', onKeyDown);
        return () => window.removeEventListener('keydown', onKeyDown);
    }, [onKeyDown]);

    return (
        <div className="viewport">
            <canvas
                ref={canvasRef}
                width={meta?.width ?? 1920}
                height={meta?.height ?? 1080}
            />
            <div className="frame-indicator">
                Frame {frame} / {(meta?.total_frames ?? 1) - 1}
                {meta && <> · {meta.width}×{meta.height} @ {meta.fps}fps</>}
            </div>
        </div>
    );
}
