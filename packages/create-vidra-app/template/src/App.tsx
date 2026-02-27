import { useState } from 'react';
import PlayerMode from './components/PlayerMode';
import RenderMode from './components/RenderMode';
import WebSceneMode from './components/WebSceneMode';

function App() {
    const [activeTab, setActiveTab] = useState<'player' | 'sdk' | 'web'>('player');

    return (
        <div className="min-h-screen bg-vd-bg text-vd-text flex flex-col font-sans">
            {/* Header */}
            <header className="border-b border-vd-border p-4 flex flex-col gap-4">
                <div>
                    <h1 className="text-xl font-bold flex items-center gap-2">
                        🎬 {"{{PROJECT_NAME}}"}
                    </h1>
                    <p className="text-sm text-vd-dim mt-1">Vidra Interactive Development Environment</p>
                </div>

                {/* Navigation Tabs */}
                <nav className="flex gap-4 border-b border-vd-border pb-[-1px]">
                    <button
                        onClick={() => setActiveTab('player')}
                        className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${activeTab === 'player'
                            ? 'border-vd-accent text-vd-accent'
                            : 'border-transparent text-vd-dim hover:text-vd-text'
                            }`}
                    >
                        ▶️ Player Mode
                    </button>
                    <button
                        onClick={() => setActiveTab('sdk')}
                        className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${activeTab === 'sdk'
                            ? 'border-vd-accent text-vd-accent'
                            : 'border-transparent text-vd-dim hover:text-vd-text'
                            }`}
                    >
                        ⚙️ SDK / Render IR
                    </button>
                    <button
                        onClick={() => setActiveTab('web')}
                        className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${activeTab === 'web'
                            ? 'border-vd-accent text-vd-accent'
                            : 'border-transparent text-vd-dim hover:text-vd-text'
                            }`}
                    >
                        🌐 Web Scene
                    </button>
                </nav>
            </header>

            {/* Main Content Area */}
            <main className="flex-1 p-6 relative">
                <div className={activeTab === 'player' ? 'block' : 'hidden'}>
                    <PlayerMode isActive={activeTab === 'player'} />
                </div>

                <div className={activeTab === 'sdk' ? 'block' : 'hidden'}>
                    <RenderMode />
                </div>

                <div className={activeTab === 'web' ? 'block' : 'hidden'}>
                    <WebSceneMode />
                </div>
            </main>
        </div>
    );
}

export default App;
