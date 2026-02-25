// ─── @sansavision/vidra-web-capture/react ────────────────────────────
//
// React hook that wraps VidraCapture and provides reactive state.

import { useState, useEffect, useRef, useCallback } from 'react';
import { VidraCapture, type VidraState } from './index.js';

export interface UseVidraSceneOptions {
    /**
     * How often (in ms) to poll the bridge for updates when in
     * standalone mode. Defaults to 16ms (~60fps).
     * In capture mode the harness drives updates via postMessage,
     * so this value is only used for the real-time fallback.
     */
    pollInterval?: number;
}

/**
 * React hook for making a component Vidra-capturable.
 *
 * When running inside the Vidra capture harness, it returns the
 * current frame/time/fps/vars from the injected `window.__vidra`
 * bridge. When running standalone in a normal browser, it returns
 * sensible real-time defaults so the component renders normally.
 *
 * @example
 * ```tsx
 * import { useVidraScene } from '@sansavision/vidra-web-capture/react';
 *
 * export default function MyScene() {
 *     const { frame, time, fps, vars, capturing, emit } = useVidraScene();
 *     return (
 *         <div style={{ opacity: vars.opacity ?? 1 }}>
 *             {capturing ? `Frame ${frame}` : 'Live preview'}
 *         </div>
 *     );
 * }
 * ```
 */
export function useVidraScene(opts: UseVidraSceneOptions = {}): VidraState {
    const { pollInterval = 16 } = opts;

    const captureRef = useRef<VidraCapture | null>(null);
    if (!captureRef.current) {
        captureRef.current = new VidraCapture();
    }

    const [state, setState] = useState<VidraState>(() => captureRef.current!.getState());

    // Listen for postMessage-based updates from the capture harness
    useEffect(() => {
        const onMessage = (ev: MessageEvent) => {
            if (ev.data?.type === 'vidra_frame') {
                // The harness has advanced a frame — re-read bridge.
                setState(captureRef.current!.getState());
            }
        };

        window.addEventListener('message', onMessage);

        // In standalone mode, poll so that `time` updates for animations.
        let timerId: ReturnType<typeof setInterval> | undefined;
        if (!captureRef.current!.isCapturing()) {
            timerId = setInterval(() => {
                setState(captureRef.current!.getState());
            }, pollInterval);
        }

        return () => {
            window.removeEventListener('message', onMessage);
            if (timerId !== undefined) clearInterval(timerId);
        };
    }, [pollInterval]);

    // Stable emit callback
    const emit = useCallback((key: string, value: unknown) => {
        captureRef.current?.emit(key, value);
    }, []);

    return { ...state, emit };
}

export { VidraCapture, type VidraState, type VidraBridge } from './index.js';
