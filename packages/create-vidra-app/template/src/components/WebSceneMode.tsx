import { useState, useEffect, useRef, useMemo } from 'react';

/** A pure React animated bar chart — no iframe, no HTML files. */
export default function WebSceneMode() {
    const [time, setTime] = useState(0);
    const rafRef = useRef<number>(0);
    const startRef = useRef(performance.now());

    useEffect(() => {
        const loop = () => {
            setTime((performance.now() - startRef.current) / 1000);
            rafRef.current = requestAnimationFrame(loop);
        };
        rafRef.current = requestAnimationFrame(loop);
        return () => cancelAnimationFrame(rafRef.current);
    }, []);

    const data = useMemo(() => [
        { label: 'Q1', value: 45, color: '#3b82f6' },
        { label: 'Q2', value: 72, color: '#22c55e' },
        { label: 'Q3', value: 58, color: '#a855f7' },
        { label: 'Q4', value: 91, color: '#f97316' },
        { label: 'Q5', value: 68, color: '#ec4899' },
    ], []);

    const maxVal = Math.max(...data.map(d => d.value));

    return (
        <div className="h-full flex flex-col items-center justify-center">
            <div className="w-full max-w-3xl bg-gradient-to-br from-[#0f172a] to-[#1e293b] rounded-2xl p-10 border border-white/10 shadow-2xl shadow-black/40">
                <h2 className="text-2xl font-bold text-white mb-2 text-center">Revenue by Quarter</h2>
                <p className="text-sm text-blue-300/60 text-center mb-10">
                    This is a <span className="text-blue-400 font-semibold">React component</span> animated with <code className="bg-white/5 px-1.5 py-0.5 rounded text-xs">requestAnimationFrame</code> — exactly how Vidra captures web scenes into video frames.
                </p>

                <div className="flex items-end justify-center gap-6 h-64">
                    {data.map((d, i) => {
                        const delay = i * 0.2;
                        const t = Math.max(0, Math.min((time - delay) / 1.2, 1));
                        const eased = 1 - Math.pow(1 - t, 3);
                        const height = eased * (d.value / maxVal) * 200;
                        const labelOpacity = Math.min(1, Math.max(0, (time - delay - 0.5) / 0.4));

                        return (
                            <div key={d.label} className="flex flex-col items-center gap-2 w-16">
                                <span className="text-xs font-bold text-white tabular-nums transition-opacity" style={{ opacity: labelOpacity }}>
                                    ${d.value}k
                                </span>
                                <div className="w-full rounded-t-lg transition-all relative overflow-hidden"
                                    style={{ height: `${height}px`, background: `linear-gradient(to top, ${d.color}88, ${d.color})` }}>
                                    <div className="absolute inset-0 bg-gradient-to-t from-transparent to-white/10" />
                                </div>
                                <span className="text-xs text-slate-400 font-medium">{d.label}</span>
                            </div>
                        );
                    })}
                </div>

                <div className="mt-8 flex items-center justify-center gap-4">
                    <button onClick={() => { startRef.current = performance.now(); setTime(0); }}
                        className="px-5 py-2.5 bg-gradient-to-r from-blue-500 to-blue-700 hover:from-blue-400 hover:to-blue-600 text-white text-sm font-semibold rounded-xl transition-all hover:scale-105 active:scale-95 shadow-lg shadow-blue-500/20">
                        ▶ Replay Animation
                    </button>
                    <span className="text-xs text-slate-500 font-mono tabular-nums">{time.toFixed(1)}s</span>
                </div>
            </div>
        </div>
    );
}
