// Canvas panel — renders frames via WebSocket (Task 9.4)
import { useRef, useEffect, useCallback, useState } from 'react';
import { useProjectStore } from '../hooks/useProject';

interface CanvasPanelProps {
    meta: { width: number; height: number; fps: number; total_frames: number } | null;
    requestFrame: (f: number) => void;
    setFrameCallback: (cb: (data: ArrayBuffer) => void) => void;
}

export function CanvasPanel({ meta, requestFrame, setFrameCallback }: CanvasPanelProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const viewportRef = useRef<HTMLDivElement>(null);
    const frame = useProjectStore(s => s.frame);
    const setFrame = useProjectStore(s => s.setFrame);

    // Zoom & Pan state
    const [scale, setScale] = useState(1);
    const [pan, setPan] = useState({ x: 0, y: 0 });
    const isDragging = useRef(false);

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
        // Ignore if typing in an input or textarea
        if (document.activeElement?.tagName === 'INPUT' || document.activeElement?.tagName === 'TEXTAREA') return;

        if (e.key === 'ArrowRight') setFrame(Math.min(frame + 1, meta.total_frames - 1));
        if (e.key === 'ArrowLeft') setFrame(Math.max(frame - 1, 0));
        if (e.key === '0' && (e.ctrlKey || e.metaKey)) {
            // Reset view
            setScale(1);
            setPan({ x: 0, y: 0 });
            e.preventDefault();
        }
    }, [frame, meta, setFrame]);

    useEffect(() => {
        window.addEventListener('keydown', onKeyDown);
        return () => window.removeEventListener('keydown', onKeyDown);
    }, [onKeyDown]);

    // Handle Pan & Zoom
    const handleWheel = (e: React.WheelEvent) => {
        if (e.ctrlKey || e.metaKey) {
            // Zoom
            e.preventDefault();
            e.stopPropagation();

            const zoomSensitivity = 0.001;
            const delta = -e.deltaY * zoomSensitivity;
            let newScale = scale * Math.exp(delta);
            newScale = Math.min(Math.max(newScale, 0.1), 10);

            setScale(newScale);
        } else {
            // Pan
            setPan(p => ({
                x: p.x - e.deltaX,
                y: p.y - e.deltaY,
            }));
        }
    };

    const handlePointerDown = (e: React.PointerEvent) => {
        if (e.button === 1 || (e.button === 0 && e.altKey)) { // Middle click or Alt+Left click
            isDragging.current = true;
            if (viewportRef.current) viewportRef.current.setPointerCapture(e.pointerId);
            e.preventDefault();
        }
    };

    const handlePointerMove = (e: React.PointerEvent) => {
        if (isDragging.current) {
            setPan(p => ({
                x: p.x + e.movementX,
                y: p.y + e.movementY,
            }));
        }
    };

    const handlePointerUp = (e: React.PointerEvent) => {
        isDragging.current = false;
        if (viewportRef.current && viewportRef.current.hasPointerCapture(e.pointerId)) {
            viewportRef.current.releasePointerCapture(e.pointerId);
        }
    };

    return (
        <div
            className="viewport"
            ref={viewportRef}
            onWheel={handleWheel}
            onPointerDown={handlePointerDown}
            onPointerMove={handlePointerMove}
            onPointerUp={handlePointerUp}
            style={{ overflow: 'hidden', touchAction: 'none' }}
        >
            <div style={{
                transform: `translate(${pan.x}px, ${pan.y}px) scale(${scale})`,
                transformOrigin: '0 0',
                willChange: 'transform',
                transition: isDragging.current ? 'none' : 'transform 0.05s ease-out',
                position: 'absolute',
                top: '50%',
                left: '50%',
                // Adjust for center
                marginLeft: meta ? -(meta.width * scale) / 2 / scale : 0,
                marginTop: meta ? -(meta.height * scale) / 2 / scale : 0,
            }}>
                <canvas
                    ref={canvasRef}
                    width={meta?.width ?? 1920}
                    height={meta?.height ?? 1080}
                    style={{
                        boxShadow: '0 8px 32px rgba(0,0,0,0.5)',
                        backgroundColor: '#000',
                    }}
                />
            </div>
            <div className="frame-indicator">
                Frame {frame} / {(meta?.total_frames ?? 1) - 1}
                {meta && <> · {meta.width}×{meta.height} @ {meta.fps}fps</>}
                {' '}- {(scale * 100).toFixed(0)}%
            </div>
        </div>
    );
}
