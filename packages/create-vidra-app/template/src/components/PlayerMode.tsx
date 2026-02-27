import { useState, useEffect, useRef } from 'react';
import { Play, Pause, RotateCcw, MonitorPlay, AlertCircle } from 'lucide-react';
import { VidraEngine } from '@sansavision/vidra-player';

interface PlayerModeProps { isActive: boolean; }

export default function PlayerMode({ isActive }: PlayerModeProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const engineRef = useRef<VidraEngine | null>(null);

    const [isPlaying, setIsPlaying] = useState(false);
    const [currentTime, setCurrentTime] = useState(0);
    const [duration, setDuration] = useState(1);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!isActive || engineRef.current || !canvasRef.current) return;
        let mounted = true;

        async function init() {
            try {
                const res = await fetch('/project.json');
                if (!res.ok) throw new Error("Missing project.json. Run 'npm run build:video'");
                const jsonText = await res.text();
                if (!mounted) return;

                const engine = new VidraEngine(canvasRef.current!);

                // Use events instead of a manual requestAnimationFrame loop
                // engine.events = {
                //   onFrame: () => setCurrentTime(engine.getCurrentTime()),
                //   onError: (err) => setError(err)
                // };

                await engine.init();
                const info = engine.loadIR(jsonText);
                setDuration(info.totalDuration || 1);

                if (!mounted) return;
                engineRef.current = engine;
                engine.play();
                setIsPlaying(true);

                // Temp loop until events are fully wired
                const loop = () => {
                    if (!mounted) return;
                    if (engineRef.current) {
                        setCurrentTime(engineRef.current.getCurrentTime());
                    }
                    requestAnimationFrame(loop);
                };
                requestAnimationFrame(loop);

            } catch (err: any) {
                if (mounted) setError(err.message);
            }
        }
        init();

        return () => {
            mounted = false;
            engineRef.current?.pause();
            engineRef.current = null;
        };
    }, [isActive]);

    const togglePlay = () => {
        if (!engineRef.current) return;
        if (isPlaying) engineRef.current.pause();
        else engineRef.current.play();
        setIsPlaying(!isPlaying);
    };

    const restart = () => {
        if (!engineRef.current) return;
        engineRef.current.seekToFrame(0);
        engineRef.current.play();
        setIsPlaying(true);
    };

    const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (!engineRef.current) return;
        const t = parseFloat(e.target.value);
        engineRef.current.seekToTime(t);
        setCurrentTime(t);
    };

    return (
        <div className="h-full flex flex-col justify-center max-w-5xl mx-auto pt-4 relative">

            {/* Player Frame */}
            <div className="bg-black rounded-lg overflow-hidden border border-[#30363d] shadow-2xl relative aspect-video flex-shrink-0 group ring-1 ring-white/5">
                {error ? (
                    <div className="absolute inset-0 flex flex-col items-center justify-center text-red-400 bg-[#0d1117] p-8 text-center ring-1 ring-red-500/20">
                        <AlertCircle className="w-12 h-12 mb-4 text-red-500/80" />
                        <p className="font-mono text-sm">{error}</p>
                    </div>
                ) : (
                    <canvas
                        ref={canvasRef}
                        width="1920"
                        height="1080"
                        className="w-full h-full object-contain block"
                    />
                )}
            </div>

            {/* Sleek Controls */}
            <div className="mt-6 bg-[#161b22] border border-[#30363d] rounded-xl p-4 flex items-center gap-4 shadow-lg">
                <button
                    onClick={togglePlay}
                    disabled={!!error}
                    className="w-10 h-10 rounded-full bg-blue-600 hover:bg-blue-500 flex items-center justify-center text-white transition-all hover:scale-105 active:scale-95 disabled:opacity-50"
                >
                    {isPlaying ? <Pause className="w-5 h-5 fill-current" /> : <Play className="w-5 h-5 ml-1 fill-current" />}
                </button>

                <button
                    onClick={restart}
                    disabled={!!error}
                    className="w-10 h-10 rounded-full bg-[#0d1117] border border-[#30363d] hover:border-[#8b949e] flex items-center justify-center text-vd-text transition-all disabled:opacity-50"
                >
                    <RotateCcw className="w-4 h-4" />
                </button>

                <div className="flex-1 flex items-center gap-3">
                    <span className="font-mono text-xs text-vd-dim w-12 text-right">
                        {(currentTime).toFixed(1)}s
                    </span>

                    <input
                        type="range"
                        min="0"
                        max={duration}
                        step="0.01"
                        value={currentTime}
                        onChange={handleSeek}
                        className="flex-1 h-2 bg-[#0d1117] rounded-lg appearance-none cursor-pointer accent-blue-500"
                    />

                    <span className="font-mono text-xs text-vd-dim w-12">
                        {(duration).toFixed(1)}s
                    </span>
                </div>
            </div>
        </div>
    );
}
