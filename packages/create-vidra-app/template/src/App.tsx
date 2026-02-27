import { useState } from 'react';
import PlayerMode from './components/PlayerMode';
import WebSceneMode from './components/WebSceneMode';
import CodeEditorPanel from './components/CodeEditorPanel';
import AIChatPanel from './components/AIChatPanel';
import { MonitorPlay, Code2, Sparkles, LayoutGrid, FileVideo } from 'lucide-react';

function App() {
    const [activeTab, setActiveTab] = useState<'player' | 'code' | 'web'>('player');
    const [showAI, setShowAI] = useState(true);

    return (
        <div className="h-screen w-screen bg-vd-bg text-vd-text flex font-sans overflow-hidden">

            {/* 1. Left Icon Rail (Sleek Sidebar) */}
            <div className="w-16 bg-[#0d1117] border-r border-vd-border flex flex-col items-center py-4 gap-6 shrink-0 z-20 shadow-2xl">
                <div className="w-10 h-10 bg-blue-600 rounded-xl flex items-center justify-center shadow-lg shadow-blue-500/20 mb-4">
                    <FileVideo className="w-6 h-6 text-white" />
                </div>

                <button
                    onClick={() => setActiveTab('player')}
                    className={`w-10 h-10 rounded-lg flex items-center justify-center transition-all ${activeTab === 'player' ? 'bg-[#1f6feb] text-white shadow-md' : 'text-vd-dim hover:text-white hover:bg-white/5'
                        }`}
                    title="Vidra Player"
                >
                    <MonitorPlay className="w-5 h-5" />
                </button>

                <button
                    onClick={() => setActiveTab('code')}
                    className={`w-10 h-10 rounded-lg flex items-center justify-center transition-all ${activeTab === 'code' ? 'bg-[#1f6feb] text-white shadow-md' : 'text-vd-dim hover:text-white hover:bg-white/5'
                        }`}
                    title="Source Code"
                >
                    <Code2 className="w-5 h-5" />
                </button>

                <button
                    onClick={() => setActiveTab('web')}
                    className={`w-10 h-10 rounded-lg flex items-center justify-center transition-all ${activeTab === 'web' ? 'bg-[#1f6feb] text-white shadow-md' : 'text-vd-dim hover:text-white hover:bg-white/5'
                        }`}
                    title="Web Scene Engine"
                >
                    <LayoutGrid className="w-5 h-5" />
                </button>

                <div className="flex-1" />

                <button
                    onClick={() => setShowAI(!showAI)}
                    className={`w-10 h-10 rounded-lg flex items-center justify-center transition-all ${showAI ? 'bg-[#bd561d]/20 text-[#f0883e]' : 'text-vd-dim hover:text-white hover:bg-white/5'
                        }`}
                    title="AI Assistant"
                >
                    <Sparkles className="w-5 h-5" />
                </button>
            </div>

            {/* 2. Main Content Area */}
            <main className="flex-1 flex flex-col relative bg-gradient-to-br from-[#0d1117] to-[#161b22]">

                {/* Header Ribbon */}
                <header className="h-14 border-b border-vd-border px-6 flex items-center justify-between shrink-0 bg-[#0d1117]/50 backdrop-blur-md">
                    <div className="flex items-center gap-3">
                        <span className="font-semibold text-lg text-[#e6edf3]">🎬 {"{{PROJECT_NAME}}"}</span>
                        <span className="px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider bg-blue-500/10 text-blue-400 border border-blue-500/20">
                            Vidra Studio
                        </span>
                    </div>
                    <div className="flex items-center gap-4 text-sm text-vd-dim font-mono">
                        <span>npm run build:video</span>
                    </div>
                </header>

                {/* Tab Content Canvas */}
                <div className="flex-1 overflow-auto p-6">
                    <div className={`h-full ${activeTab === 'player' ? 'block' : 'hidden'}`}>
                        <PlayerMode isActive={activeTab === 'player'} />
                    </div>
                    <div className={`h-full ${activeTab === 'code' ? 'block' : 'hidden'}`}>
                        <CodeEditorPanel />
                    </div>
                    <div className={`h-full ${activeTab === 'web' ? 'block' : 'hidden'}`}>
                        <WebSceneMode />
                    </div>
                </div>

            </main>

            {/* 3. AI Sidebar (Right) */}
            {showAI && (
                <aside className="w-[380px] shrink-0 transform transition-all shadow-[-10px_0_30px_rgba(0,0,0,0.5)] z-30">
                    <AIChatPanel />
                </aside>
            )}

        </div>
    );
}

export default App;
