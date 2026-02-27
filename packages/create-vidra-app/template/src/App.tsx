import { useState } from 'react';
import { MonitorPlay, Code2, LayoutGrid, Sparkles, FileVideo, BookOpen, ExternalLink } from 'lucide-react';
import PlayerMode from './components/PlayerMode';
import WebSceneMode from './components/WebSceneMode';
import AIChatPanel from './components/AIChatPanel';

type Tab = 'player' | 'examples' | 'web';

function App() {
    const [activeTab, setActiveTab] = useState<Tab>('player');
    const [showAI, setShowAI] = useState(false);

    const tabs: { id: Tab; icon: typeof MonitorPlay; label: string; desc: string }[] = [
        { id: 'player', icon: MonitorPlay, label: 'Player', desc: 'Live WASM preview' },
        { id: 'examples', icon: BookOpen, label: 'Learn', desc: 'Quick start guide' },
        { id: 'web', icon: LayoutGrid, label: 'Web Scene', desc: 'React animation demo' },
    ];

    return (
        <div className="h-screen w-screen bg-[#0a0e17] text-white flex overflow-hidden font-sans">

            {/* ── Left Rail ── */}
            <div className="w-[72px] bg-[#0d1117] border-r border-white/[.06] flex flex-col items-center py-5 gap-2 shrink-0">
                <div className="w-11 h-11 bg-gradient-to-br from-blue-500 to-blue-700 rounded-[14px] flex items-center justify-center shadow-lg shadow-blue-500/20 mb-6">
                    <FileVideo className="w-6 h-6 text-white" />
                </div>

                {tabs.map(t => {
                    const Icon = t.icon;
                    const isActive = activeTab === t.id;
                    return (
                        <button key={t.id} onClick={() => setActiveTab(t.id)} title={t.desc}
                            className={`w-11 h-11 rounded-[12px] flex items-center justify-center transition-all duration-200 ${isActive
                                    ? 'bg-blue-500/15 text-blue-400 shadow-sm ring-1 ring-blue-500/20'
                                    : 'text-slate-500 hover:text-slate-300 hover:bg-white/[.04]'
                                }`}>
                            <Icon className="w-5 h-5" />
                        </button>
                    );
                })}

                <div className="flex-1" />

                <button onClick={() => setShowAI(!showAI)} title="AI Assistant"
                    className={`w-11 h-11 rounded-[12px] flex items-center justify-center transition-all duration-200 ${showAI
                            ? 'bg-amber-500/15 text-amber-400 ring-1 ring-amber-500/20'
                            : 'text-slate-500 hover:text-slate-300 hover:bg-white/[.04]'
                        }`}>
                    <Sparkles className="w-5 h-5" />
                </button>
            </div>

            {/* ── Main Area ── */}
            <main className="flex-1 flex flex-col min-w-0">
                {/* Header */}
                <header className="h-12 border-b border-white/[.06] px-6 flex items-center justify-between shrink-0 bg-[#0d1117]/60 backdrop-blur-sm">
                    <div className="flex items-center gap-3">
                        <span className="font-bold text-[15px]">{"{{PROJECT_NAME}}"}</span>
                        <span className="px-2 py-0.5 text-[10px] font-bold uppercase tracking-widest bg-gradient-to-r from-blue-500/10 to-purple-500/10 text-blue-300 rounded-md border border-blue-500/15">
                            Vidra Studio
                        </span>
                    </div>
                    <div className="flex items-center gap-2 text-xs text-slate-500">
                        <span className="hidden sm:inline font-mono">v0.1</span>
                    </div>
                </header>

                {/* Content */}
                <div className="flex-1 overflow-auto bg-gradient-to-b from-[#0d1117] to-[#0a0e17]">
                    {activeTab === 'player' && <PlayerMode isActive={activeTab === 'player'} />}
                    {activeTab === 'web' && <WebSceneMode />}
                    {activeTab === 'examples' && <LearnPanel />}
                </div>
            </main>

            {/* ── AI Sidebar ── */}
            {showAI && (
                <aside className="w-[360px] shrink-0 shadow-[-8px_0_40px_rgba(0,0,0,0.6)]">
                    <AIChatPanel onClose={() => setShowAI(false)} />
                </aside>
            )}
        </div>
    );
}

/** Getting Started / Learn panel */
function LearnPanel() {
    const examples = [
        { title: 'Edit your video', desc: 'Open src/video.ts in your editor. Change colors, text, animations and rebuild.', cmd: 'npm run build:video' },
        { title: 'Preview in browser', desc: 'The Player tab auto-loads public/project.json and plays it in real-time via WASM.', cmd: 'npm run dev' },
        { title: 'Render to MP4', desc: 'Use the CLI to render your project to a video file at full quality.', cmd: 'npx @sansavision/vidra render video.vidra -o output.mp4' },
        { title: 'Web Scenes', desc: 'Build animated React components and capture them frame-by-frame into your video.', cmd: null },
    ];

    return (
        <div className="max-w-3xl mx-auto p-8 space-y-8">
            <div>
                <h1 className="text-3xl font-bold bg-gradient-to-r from-white to-blue-200 bg-clip-text text-transparent">
                    Welcome to Vidra
                </h1>
                <p className="text-slate-400 mt-2 text-lg">
                    Build stunning programmatic videos with TypeScript, render anywhere.
                </p>
            </div>

            <div className="grid gap-4">
                {examples.map((ex, i) => (
                    <div key={i} className="bg-[#161b22] border border-white/[.06] rounded-xl p-5 hover:border-blue-500/20 transition-colors group">
                        <div className="flex items-start justify-between">
                            <div>
                                <h3 className="font-semibold text-white flex items-center gap-2">
                                    <span className="w-6 h-6 rounded-lg bg-blue-500/10 text-blue-400 text-xs flex items-center justify-center font-bold">{i + 1}</span>
                                    {ex.title}
                                </h3>
                                <p className="text-sm text-slate-400 mt-1.5">{ex.desc}</p>
                            </div>
                        </div>
                        {ex.cmd && (
                            <div className="mt-3 bg-[#0d1117] rounded-lg px-4 py-2.5 font-mono text-xs text-blue-300 border border-white/[.04] select-all">
                                {ex.cmd}
                            </div>
                        )}
                    </div>
                ))}
            </div>

            <div className="flex gap-3">
                <a href="https://github.com/Sansa-Organisation/vidra" target="_blank" rel="noopener"
                    className="flex items-center gap-2 px-5 py-2.5 bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl text-sm font-medium transition-colors">
                    <ExternalLink className="w-4 h-4" /> Documentation
                </a>
                <a href="https://github.com/Sansa-Organisation/vidra" target="_blank" rel="noopener"
                    className="flex items-center gap-2 px-5 py-2.5 bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl text-sm font-medium transition-colors">
                    <Code2 className="w-4 h-4" /> View on GitHub
                </a>
            </div>
        </div>
    );
}

export default App;
