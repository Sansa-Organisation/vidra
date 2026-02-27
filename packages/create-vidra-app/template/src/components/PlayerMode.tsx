import { useState, useEffect, useRef } from 'react';
import { VidraEngine } from '@sansavision/vidra-player';

interface PlayerModeProps {
    isActive: boolean;
}

export default function PlayerMode({ isActive }: PlayerModeProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const engineRef = useRef<VidraEngine | null>(null);
    const rafRef = useRef<number>();

    const [isPlaying, setIsPlaying] = useState(false);
    const [currentTime, setCurrentTime] = useState(0);
    const [error, setError] = useState<string | null>(null);
    const [irJSON, setIrJSON] = useState<string>('');

    useEffect(() => {
        // Only initialize if we're the active tab and haven't initialized yet
        if (!isActive || engineRef.current || !canvasRef.current) return;

        let mounted = true;

        async function init() {
            try {
                // Fetch project JSON
                const res = await fetch('/project.json');
                if (!res.ok) throw new Error("Could not find project.json. Did you run 'npm run build:video'?");
                const jsonText = await res.text();
                if (!mounted) return;
                setIrJSON(jsonText);

                // Initialize engine
                const engine = new VidraEngine(canvasRef.current!);
                await engine.init();
                engine.loadIR(jsonText);

                if (!mounted) {
                    // Cleanup if unmounted during init
                    return;
                }

                engineRef.current = engine;
                engine.play();
                setIsPlaying(true);

                // Render loop
                const loop = () => {
                    if (!mounted) return;
                    if (engineRef.current) {
                        engineRef.current.renderCurrentFrame();
                        setCurrentTime(engineRef.current.getCurrentTime());
                    }
                    rafRef.current = requestAnimationFrame(loop);
                };
                rafRef.current = requestAnimationFrame(loop);

            } catch (err: any) {
                if (mounted) setError(err.message);
            }
        }

        init();

        return () => {
            mounted = false;
            if (rafRef.current) cancelAnimationFrame(rafRef.current);
            // Clean up engine if VidraEngine supports cleanup/destroy in the future.
            engineRef.current = null;
        };
    }, [isActive]);

    const togglePlay = () => {
        if (!engineRef.current) return;
        if (isPlaying) {
            engineRef.current.pause();
        } else {
            engineRef.current.play();
        }
        setIsPlaying(!isPlaying);
    };

    const restart = () => {
        if (!engineRef.current) return;
        engineRef.current.seekToFrame(0);
        engineRef.current.play();
        setIsPlaying(true);
    };

    return (
        <div className="max-w-5xl mx-auto flex flex-col gap-6">
            <div className="bg-vd-panel border border-vd-border rounded-lg p-6">
                <h2 className="text-xl font-bold mb-2 text-vd-accent">Vidra WASM Player</h2>
                <p className="text-vd-dim mb-4">
                    This mode uses <code>@sansavision/vidra-player</code> to natively render the project timeline in your browser via WebAssembly and WebGL.
                </p>

                <div className="flex items-center gap-3">
                    <button
                        onClick={togglePlay}
                        disabled={!!error}
                        className="px-4 py-2 bg-vd-accent hover:bg-vd-accent-hover text-white rounded font-medium disabled:opacity-50"
                    >
                        {isPlaying ? 'Pause' : 'Play'}
                    </button>
                    <button
                        onClick={restart}
                        disabled={!!error}
                        className="px-4 py-2 bg-transparent border border-vd-border hover:border-vd-dim rounded font-medium disabled:opacity-50"
                    >
                        Restart
                    </button>
                    <span className="font-mono text-vd-dim ml-2 w-20">
                        {error ? 'Error' : `${currentTime.toFixed(2)}s`}
                    </span>
                    {error && <span className="text-red-400 text-sm ml-2">{error}</span>}
                </div>
            </div>

            <div className="bg-black rounded-lg overflow-hidden border border-vd-border shadow-2xl relative aspect-video flex-shrink-0">
                <canvas
                    ref={canvasRef}
                    id="vidra-canvas"
                    width="1920"
                    height="1080"
                    className="w-full h-full object-contain block"
                ></canvas>
            </div>
        </div>
    );
}
