import { useState, useEffect, useRef } from 'react';

export default function WebSceneMode() {
    const iframeRef = useRef<HTMLIFrameElement>(null);

    useEffect(() => {
        // Drive the iframe animation independently when previewed in this tab
        let frame = 0;
        let animId: number;
        let lastTime = performance.now();

        const loop = () => {
            const now = performance.now();
            if (now - lastTime > 1000 / 30) {
                frame++;
                iframeRef.current?.contentWindow?.postMessage({
                    type: "vidra_frame",
                    frame: frame,
                    time: frame / 30,
                    fps: 30
                }, "*");
                lastTime = now;
            }
            animId = requestAnimationFrame(loop);
        };

        animId = requestAnimationFrame(loop);
        return () => cancelAnimationFrame(animId);
    }, []);

    return (
        <div className="h-full flex flex-col pt-4">
            <div className="mb-4">
                <h2 className="text-xl font-bold mb-2">Web Capture Scene</h2>
                <p className="text-vd-dim text-sm max-w-2xl">
                    This is an ordinary Vite development server serving a standard HTML/JS page (<code>web/chart.html</code>).
                    When rendering to video, VidraEngine mounts this URL in a headless browser, syncs its <code>requestAnimationFrame</code> loop to the video timeline, and captures the frames.
                </p>
            </div>

            <div className="flex-1 bg-[#0d1117] rounded-lg overflow-hidden border border-vd-border shadow-2xl relative">
                <iframe
                    ref={iframeRef}
                    src="/web/chart.html"
                    className="w-full h-full border-0 absolute inset-0"
                    title="Web Scene Preview"
                ></iframe>
            </div>
        </div>
    );
}
