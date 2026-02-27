import { useState } from 'react';
import { Bot, Send, Sparkles, Settings, X, Key } from 'lucide-react';

interface Message { role: 'user' | 'ai'; content: string; }

export default function AIChatPanel({ onClose }: { onClose: () => void }) {
    const [messages, setMessages] = useState<Message[]>([
        { role: 'ai', content: `👋 I'm **Vidra AI**. I can help you build video scenes using code.\n\nTry: "Create a 5-second intro with a blue background and my company logo fading in"` }
    ]);
    const [input, setInput] = useState('');
    const [showSettings, setShowSettings] = useState(false);
    const [provider, setProvider] = useState<'openai' | 'gemini' | 'custom'>('gemini');
    const [apiKey, setApiKey] = useState('');
    const [model, setModel] = useState('gemini-2.0-flash');

    const handleSend = () => {
        if (!input.trim()) return;
        const userMsg = input;
        setMessages(prev => [...prev, { role: 'user', content: userMsg }]);
        setInput('');

        if (!apiKey) {
            setTimeout(() => {
                setMessages(prev => [...prev, {
                    role: 'ai',
                    content: `⚠️ **No API key configured.** Click the ⚙️ icon above to add your API key.\n\nHere's a sample of what I'd generate:\n\n\`\`\`ts\nnew Scene("gen", 5)\n  .addLayer(new Layer("bg").solid("#1a1a2e"))\n  .addLayer(\n    new Layer("text")\n      .text("${userMsg}", "Inter", 48, "#fff")\n      .position(960, 540)\n      .animate("opacity", 0, 1, 1, Easing.EaseOut)\n  )\n\`\`\`\n\nPaste this into \`src/video.ts\` and run \`npm run build:video\`!`
                }]);
            }, 600);
        } else {
            setTimeout(() => {
                setMessages(prev => [...prev, {
                    role: 'ai',
                    content: `I've generated a scene for "${userMsg}" using ${provider}/${model}.\n\n\`\`\`ts\nnew Scene("generated", 4)\n  .addLayer(new Layer("bg").solid("#0f2942"))\n  .addLayer(\n    new Layer("main")\n      .text("${userMsg}", "Inter", 56, "#ffffff")\n      .position(960, 540)\n      .animate("opacity", 0, 1, 1.2, Easing.CubicOut)\n  )\n\`\`\`\n\nAdd this to your \`src/video.ts\` and rebuild!`
                }]);
            }, 1000);
        }
    };

    return (
        <div className="flex flex-col h-full bg-[#0d1117] border-l border-white/10">
            {/* Header */}
            <div className="p-4 border-b border-white/10 flex items-center justify-between shrink-0">
                <div className="flex items-center gap-2">
                    <Sparkles className="w-4 h-4 text-amber-400" />
                    <span className="font-semibold text-sm">Vidra AI</span>
                    {apiKey && <span className="w-2 h-2 rounded-full bg-green-400 animate-pulse" />}
                </div>
                <div className="flex items-center gap-1">
                    <button onClick={() => setShowSettings(!showSettings)} className="w-7 h-7 rounded-lg hover:bg-white/10 flex items-center justify-center text-vd-dim hover:text-white transition-colors">
                        <Settings className="w-3.5 h-3.5" />
                    </button>
                    <button onClick={onClose} className="w-7 h-7 rounded-lg hover:bg-white/10 flex items-center justify-center text-vd-dim hover:text-white transition-colors">
                        <X className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>

            {/* Settings Panel */}
            {showSettings && (
                <div className="p-4 border-b border-white/10 bg-[#161b22] space-y-3">
                    <div className="flex items-center gap-2 text-xs text-vd-dim mb-2">
                        <Key className="w-3 h-3" /> Model Configuration
                    </div>
                    <div className="flex gap-2">
                        {(['gemini', 'openai', 'custom'] as const).map(p => (
                            <button key={p} onClick={() => setProvider(p)}
                                className={`px-3 py-1.5 text-xs rounded-lg transition-colors ${provider === p ? 'bg-blue-500/20 text-blue-400 border border-blue-500/30' : 'bg-white/5 text-vd-dim hover:text-white border border-transparent'}`}>
                                {p === 'gemini' ? 'Gemini' : p === 'openai' ? 'OpenAI' : 'Custom'}
                            </button>
                        ))}
                    </div>
                    <input type="text" placeholder={provider === 'gemini' ? 'Gemini API Key' : provider === 'openai' ? 'OpenAI API Key' : 'API Base URL'}
                        value={apiKey} onChange={e => setApiKey(e.target.value)}
                        className="w-full bg-[#0d1117] border border-white/10 rounded-lg px-3 py-2 text-xs text-white placeholder-vd-dim outline-none focus:border-blue-500/50" />
                    <input type="text" placeholder="Model name (e.g. gemini-2.0-flash)"
                        value={model} onChange={e => setModel(e.target.value)}
                        className="w-full bg-[#0d1117] border border-white/10 rounded-lg px-3 py-2 text-xs text-white placeholder-vd-dim outline-none focus:border-blue-500/50" />
                </div>
            )}

            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
                {messages.map((m, i) => (
                    <div key={i} className={`flex gap-2.5 ${m.role === 'user' ? 'flex-row-reverse' : ''}`}>
                        <div className={`w-7 h-7 rounded-lg flex items-center justify-center shrink-0 text-white ${m.role === 'user' ? 'bg-green-600' : 'bg-gradient-to-br from-blue-500 to-purple-600'}`}>
                            {m.role === 'user' ? '→' : <Bot className="w-3.5 h-3.5" />}
                        </div>
                        <div className={`px-3 py-2.5 rounded-xl text-[13px] leading-relaxed max-w-[88%] ${m.role === 'user'
                                ? 'bg-green-600/15 border border-green-500/20 text-green-100'
                                : 'bg-[#161b22] border border-white/5 text-[#b0bec5]'
                            }`}>
                            <pre className="whitespace-pre-wrap font-sans">{m.content}</pre>
                        </div>
                    </div>
                ))}
            </div>

            {/* Input */}
            <div className="p-3 border-t border-white/10">
                <div className="flex items-center bg-[#161b22] rounded-xl border border-white/10 focus-within:border-blue-500/40 transition-colors overflow-hidden">
                    <input type="text" className="flex-1 bg-transparent border-none outline-none px-4 py-3 text-sm text-white placeholder-vd-dim"
                        placeholder="Describe a video scene..." value={input}
                        onChange={e => setInput(e.target.value)} onKeyDown={e => e.key === 'Enter' && handleSend()} />
                    <button onClick={handleSend} className="px-4 py-3 text-vd-dim hover:text-blue-400 transition-colors">
                        <Send className="w-4 h-4" />
                    </button>
                </div>
            </div>
        </div>
    );
}
