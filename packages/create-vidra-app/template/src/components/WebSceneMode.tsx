export default function WebSceneMode() {
    return (
        <div className="max-w-5xl mx-auto flex flex-col gap-6 h-[calc(100vh-140px)]">
            <div className="bg-vd-panel border border-vd-border rounded-lg p-6">
                <h2 className="text-xl font-bold mb-2">Web Capture Scene</h2>
                <p className="text-vd-dim mb-2">
                    This is an ordinary Vite development server serving a standard HTML/JS page (<code>web/chart.html</code>).
                </p>
                <p className="text-vd-dim">
                    When rendering to video, VidraEngine mounts this URL in an automated headless browser,
                    syncs its <code>requestAnimationFrame</code> loop to the video timeline, and captures the frames!
                </p>
            </div>

            <div className="flex-1 bg-white rounded-lg overflow-hidden border border-vd-border shadow-2xl relative">
                <iframe
                    src="/web/chart.html"
                    className="w-full h-full border-0 absolute inset-0"
                    title="Web Scene Preview"
                ></iframe>
            </div>
        </div>
    );
}
