import { useState, useEffect, useRef } from 'react';
import { Play, Pause, RotateCcw, AlertCircle, Volume2 } from 'lucide-react';
import { VidraEngine } from '@sansavision/vidra-player';

interface PlayerModeProps { isActive: boolean; }

export default function PlayerMode({ isActive }: PlayerModeProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const engineRef = useRef<VidraEngine | null>(null);
    const rafRef = useRef<number>(0);

    const [isPlaying, setIsPlaying] = useState(false);
    const [currentTime, setCurrentTime] = useState(0);
    const [duration, setDuration] = useState(1);
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        if (!isActive || engineRef.current || !canvasRef.current) return;
        let mounted = true;

        async function init() {
            try {
                const res = await fetch('/project.json');
                if (!res.ok) throw new Error("Missing project.json — run 'npm run build:video' first");
                const jsonText = await res.text();
                if (!mounted) return;

                const engine = new VidraEngine(canvasRef.current!);
                await engine.init();
                const info = engine.loadIR(jsonText);
                if (!mounted) return;

                engineRef.current = engine;
                setDuration(info.totalDuration || 11);
                setLoading(false);

                // Auto-play immediately so users see animation
                engine.play();
                setIsPlaying(true);

                const loop = () => {
                    if (!mounted) return;
                    if (engineRef.current) {
                        setCurrentTime(engineRef.current.getCurrentTime());
                    }
                    rafRef.current = requestAnimationFrame(loop);
                };
                rafRef.current = requestAnimationFrame(loop);
            } catch (err: any) {
                if (mounted) { setError(err.message); setLoading(false); }
            }
        }
        init();
        return () => { mounted = false; cancelAnimationFrame(rafRef.current); };
    }, [isActive]);

    const togglePlay = () => {
        if (!engineRef.current) return;
        if (isPlaying) engineRef.current.pause(); else engineRef.current.play();
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

    const progress = duration > 0 ? (currentTime / duration) * 100 : 0;

    return (
        <div className="h-full flex flex-col items-center justify-center relative">
            {/* Canvas Container */}
            <div className="w-full max-w-4xl relative group">
                <div className="relative rounded-2xl overflow-hidden shadow-2xl shadow-black/50 ring-1 ring-white/10 bg-black aspect-video">
                    {loading && (
                        <div className="absolute inset-0 flex items-center justify-center bg-[#0d1117] z-10">
                            <div className="flex flex-col items-center gap-3">
                                <div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
                                <p className="text-sm text-vd-dim">Loading WASM Engine...</p>
                            </div>
                        </div>
                    )}
                    {error && (
                        <div className="absolute inset-0 flex flex-col items-center justify-center bg-[#0d1117] z-10 p-8">
                            <AlertCircle className="w-12 h-12 text-red-400 mb-4" />
                            <p className="text-red-300 text-sm font-mono text-center">{error}</p>
                        </div>
                    )}
                    <canvas ref={canvasRef} width="1920" height="1080" className="w-full h-full object-contain block" />
                </div>

                {/* Controls Bar — attached directly below canvas */}
                <div className="mt-4 bg-[#161b22]/80 backdrop-blur-xl border border-white/10 rounded-2xl p-3 flex items-center gap-3">
                    <button onClick={togglePlay} disabled={!!error || loading}
                        className="w-11 h-11 rounded-xl bg-gradient-to-br from-blue-500 to-blue-700 hover:from-blue-400 hover:to-blue-600 flex items-center justify-center text-white transition-all hover:scale-105 active:scale-95 disabled:opacity-40 shadow-lg shadow-blue-500/25">
                        {isPlaying ? <Pause className="w-5 h-5 fill-white" /> : <Play className="w-5 h-5 ml-0.5 fill-white" />}
                    </button>

                    <button onClick={restart} disabled={!!error || loading}
                        className="w-9 h-9 rounded-lg bg-white/5 hover:bg-white/10 flex items-center justify-center text-vd-dim hover:text-white transition-all disabled:opacity-40">
                        <RotateCcw className="w-4 h-4" />
                    </button>

                    {/* Timeline */}
                    <div className="flex-1 flex items-center gap-3 mx-2">
                        <span className="font-mono text-[11px] text-blue-400 w-10 text-right tabular-nums">{currentTime.toFixed(1)}s</span>
                        <div className="flex-1 relative h-2 bg-white/5 rounded-full overflow-hidden cursor-pointer group/timeline">
                            <div className="absolute inset-y-0 left-0 bg-gradient-to-r from-blue-500 to-blue-400 rounded-full transition-all"
                                style={{ width: `${progress}%` }} />
                            <input type="range" min="0" max={duration} step="0.03" value={currentTime} onChange={handleSeek}
                                className="absolute inset-0 w-full opacity-0 cursor-pointer" />
                        </div>
                        <span className="font-mono text-[11px] text-vd-dim w-10 tabular-nums">{duration.toFixed(1)}s</span>
                    </div>

                    <Volume2 className="w-4 h-4 text-vd-dim" />
                </div>
            </div>
        </div>
    );
}
