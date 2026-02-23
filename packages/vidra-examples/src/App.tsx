import React, { useEffect, useRef, useState, useCallback } from 'react';
import Editor from '@monaco-editor/react';
import { VidraEngine, Project, Scene, Layer, Easing, hex, rgba } from '@sansavision/vidra-player';
import { GoogleGenAI } from '@google/genai';
import OpenAI from 'openai';
import { VIDRA_SPEC_VIDRASCRIPT, VIDRA_SPEC_SDK, DOCS_DATA, PREMADE_ASSETS } from './vidra-spec';

// @ts-ignore
import demoVidraRaw from './examples/demo.vidra?raw';
// @ts-ignore
import demoSdkRaw from './examples/demo.ts?raw';

const defaultScripts: Record<string, string> = {
    vidrascript: demoVidraRaw,
    sdk: demoSdkRaw
};

// ─── Monaco VidraScript Language Registration ─────────────────────
const handleEditorWillMount = (monaco: any) => {
    monaco.languages.register({ id: 'vidrascript' });
    monaco.languages.setMonarchTokensProvider('vidrascript', {
        keywords: [
            'if', 'else', 'let', 'export', 'import', 'project', 'scene', 'layer',
            'component', 'layout_rules', 'override', 'solid', 'image', 'video',
            'audio', 'text', 'tts', 'position', 'animation', 'from', 'to',
            'duration', 'ease', 'easing', 'font', 'size', 'color', 'linear',
            'easeInOut', 'easeOut', 'easeIn', 'easeOutBack', 'cubicIn', 'cubicOut',
            'cubicInOut', 'opacity', 'scale', 'rotation', 'effect', 'preset', 'delay', 'mask'
        ],
        typeKeywords: ['String', 'Number', 'Duration', 'Color', 'Image', 'Video', 'Font', 'AssetId'],
        tokenizer: {
            root: [
                [/[a-zA-Z_]\w*(?=\()/, 'entity.name.function'],
                [/[a-zA-Z_]\w*:/, 'type.identifier'],
                [/[a-zA-Z_]\w*/, { cases: { '@keywords': 'keyword', '@typeKeywords': 'type', '@default': 'identifier' } }],
                [/\/\/.*$/, 'comment'],
                [/"([^"\\]|\\.)*"/, 'string'],
                [/#([0-9a-fA-F]{6,8})/, 'constant.other.color'],
                [/\d+(\.\d+)?(s|ms|f)?\b/, 'number'],
            ]
        }
    });
};

// ─── Helpers ──────────────────────────────────────────────────────
function formatTime(seconds: number) {
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 100);
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`;
}

// ─── Types ────────────────────────────────────────────────────────
type SidebarTab = 'code' | 'creative' | 'chat' | 'docs';
type ChatMessage = { role: 'user' | 'assistant'; content: string };

type AIProvider = 'gemini' | 'openai';
interface AIModelPreset {
    id: string;
    name: string;
    provider: AIProvider;
    model: string;
}

const AI_MODEL_PRESETS: AIModelPreset[] = [
    { id: 'gemini-flash', name: 'Gemini 2.0 Flash', provider: 'gemini', model: 'gemini-2.0-flash' },
    { id: 'gemini-pro', name: 'Gemini 2.5 Pro', provider: 'gemini', model: 'gemini-2.5-pro-preview-05-06' },
    { id: 'gpt-4o', name: 'GPT-4o', provider: 'openai', model: 'gpt-4o' },
    { id: 'gpt-4o-mini', name: 'GPT-4o Mini', provider: 'openai', model: 'gpt-4o-mini' },
    { id: 'custom', name: '✏️ Custom Model', provider: 'gemini', model: '' },
];

interface CreativeScene {
    id: string;
    name: string;
    duration: number;
    layers: CreativeLayer[];
}

interface CreativeLayer {
    id: string;
    type: 'solid' | 'text' | 'image';
    color?: string;
    text?: string;
    font?: string;
    fontSize?: number;
    textColor?: string;
    assetId?: string;
    x: number;
    y: number;
    opacity: number;
    animations: CreativeAnim[];
}

interface CreativeAnim {
    property: string;
    from: number;
    to: number;
    duration: number;
    easing: string;
}

// ─── Main App ─────────────────────────────────────────────────────
export default function App() {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const engineRef = useRef<VidraEngine | null>(null);

    const [sidebarTab, setSidebarTab] = useState<SidebarTab>('code');
    const [mode, setMode] = useState<'vidrascript' | 'sdk'>('vidrascript');
    const [code, setCode] = useState(defaultScripts.vidrascript);
    const [liveMode, setLiveMode] = useState(false);
    const liveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    const [engineState, setEngineState] = useState('idle');
    const [status, setStatus] = useState('Engine Loading');
    const [assets, setAssets] = useState<{ id: string; name: string }[]>([]);
    const [currentTimeStr, setCurrentTimeStr] = useState('00:00.00');
    const [totalTimeStr, setTotalTimeStr] = useState('00:00.00');
    const [progress, setProgress] = useState(0);
    const [metaInfo, setMetaInfo] = useState('Resolution: -- | FPS: --');
    const [compileError, setCompileError] = useState<string | null>(null);

    // Resizer
    const [sidebarWidth, setSidebarWidth] = useState(40);
    const [isResizing, setIsResizing] = useState(false);

    // Chat
    const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
    const [chatInput, setChatInput] = useState('');
    const [chatLang, setChatLang] = useState<'vidrascript' | 'sdk'>('vidrascript');
    const [chatLoading, setChatLoading] = useState(false);
    const chatEndRef = useRef<HTMLDivElement>(null);

    // AI Model Selection & API Keys
    const [selectedModelId, setSelectedModelId] = useState(() => localStorage.getItem('vidra_ai_model') || 'gemini-flash');
    const [customProvider, setCustomProvider] = useState<AIProvider>(() => (localStorage.getItem('vidra_custom_provider') as AIProvider) || 'gemini');
    const [customModelName, setCustomModelName] = useState(() => localStorage.getItem('vidra_custom_model') || '');
    const [apiKeys, setApiKeys] = useState<Record<AIProvider, string>>(() => ({
        gemini: localStorage.getItem('vidra_key_gemini') || '',
        openai: localStorage.getItem('vidra_key_openai') || '',
    }));
    const [showKeyModal, setShowKeyModal] = useState(false);

    // Creative
    const [scenes, setScenes] = useState<CreativeScene[]>([{
        id: 's1', name: 'Scene 1', duration: 3,
        layers: [
            { id: 'bg', type: 'solid', color: '#1a1a2e', x: 960, y: 540, opacity: 1, animations: [] },
            {
                id: 'title', type: 'text', text: 'Hello Vidra!', font: 'Inter', fontSize: 120,
                textColor: '#ffffff', x: 960, y: 540, opacity: 1,
                animations: [{ property: 'opacity', from: 0, to: 1, duration: 1.5, easing: 'easeOut' }]
            }
        ]
    }]);
    const [activeScene, setActiveScene] = useState(0);
    const [activeLayer, setActiveLayer] = useState<string | null>(null);
    const [creativeCodePreview, setCreativeCodePreview] = useState<string | null>(null);

    // ─── Resizer Effect ───────────────────────────────────────────
    useEffect(() => {
        const onMove = (e: MouseEvent) => {
            if (!isResizing) return;
            setSidebarWidth(Math.max(20, Math.min(80, (e.clientX / window.innerWidth) * 100)));
        };
        const onUp = () => setIsResizing(false);
        if (isResizing) {
            document.body.style.cursor = 'col-resize';
            document.body.style.userSelect = 'none';
            window.addEventListener('mousemove', onMove);
            window.addEventListener('mouseup', onUp);
        } else {
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
        }
        return () => { window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); };
    }, [isResizing]);

    // ─── Engine Init ──────────────────────────────────────────────
    useEffect(() => {
        if (!canvasRef.current || engineRef.current) return;
        const engine = new VidraEngine(canvasRef.current, {
            onReady: () => { setStatus('Engine Ready'); setEngineState('ready'); },
            onStateChange: (state) => setEngineState(state),
            onFrame: (frame) => {
                const info = engine.getProjectInfo();
                if (info) {
                    setCurrentTimeStr(formatTime(frame / info.fps));
                    setProgress((frame / Math.max(1, info.totalFrames - 1)) * 100);
                }
            },
            onError: (err) => { console.error(err); setCompileError(String(err)); }
        });
        engineRef.current = engine;
        engine.init().then(() => {
            console.log('Vidra WASM Engine initialized');
            // Load premade assets
            PREMADE_ASSETS.forEach(a => engine.loadImageAsset(a.id, a.url).catch(console.error));
            setAssets(PREMADE_ASSETS.map(a => ({ id: a.id, name: a.name })));
        }).catch(err => { setStatus('Failed to load WASM'); console.error(err); });
    }, []);

    // ─── Compile Logic ────────────────────────────────────────────
    const handleCompile = useCallback((sourceCode?: string, sourceMode?: string) => {
        const engine = engineRef.current;
        if (!engine) return;
        const c = sourceCode ?? code;
        const m = sourceMode ?? mode;
        setCompileError(null);

        try {
            let info;
            if (m === 'vidrascript') {
                info = engine.loadSource(c);
            } else {
                let cleanCode = c
                    .replace(/import\s+.*?['"].*?['"];?/g, '')
                    .replace(/export\s+/g, '');
                cleanCode += `\nreturn createDemoProject();`;
                const buildFn = new Function('Project', 'Scene', 'Layer', 'Easing', 'hex', 'rgba', cleanCode);
                const project = buildFn(Project, Scene, Layer, Easing, hex, rgba);
                info = engine.loadProject(project);
            }
            if (info) {
                setMetaInfo(`${info.width}×${info.height} · ${info.fps}fps · ${info.sceneCount} scene${info.sceneCount > 1 ? 's' : ''}`);
                setTotalTimeStr(formatTime(info.totalDuration));
                setProgress(0);
                setCurrentTimeStr('00:00.00');
                setStatus('Compiled');
            }
        } catch (err: any) {
            console.error(err);
            setCompileError(err.message || String(err));
        }
    }, [code, mode]);

    // ─── Live Mode Debounced ──────────────────────────────────────
    useEffect(() => {
        if (!liveMode) return;
        if (liveTimerRef.current) clearTimeout(liveTimerRef.current);
        liveTimerRef.current = setTimeout(() => handleCompile(), 800);
        return () => { if (liveTimerRef.current) clearTimeout(liveTimerRef.current); };
    }, [code, liveMode, handleCompile]);

    // ─── Assets ───────────────────────────────────────────────────
    const handleFiles = async (files: FileList | File[]) => {
        const engine = engineRef.current;
        if (!engine) return;
        for (const file of Array.from(files)) {
            const id = file.name.split('.')[0].replace(/[^a-zA-Z0-9_-]/g, '_');
            const url = URL.createObjectURL(file);
            setAssets(prev => [...prev, { id, name: file.name }]);
            if (file.type.startsWith('image/')) {
                try { await engine.loadImageAsset(id, url); } catch (e) { console.error(e); }
            }
        }
    };

    // ─── Seek ─────────────────────────────────────────────────────
    const handleSeek = (e: React.MouseEvent<HTMLDivElement>) => {
        const engine = engineRef.current;
        if (!engine) return;
        const info = engine.getProjectInfo();
        if (!info) return;
        const rect = e.currentTarget.getBoundingClientRect();
        const percent = Math.max(0, e.clientX - rect.left) / rect.width;
        engine.seekToFrame(Math.floor(percent * info.totalFrames));
    };

    // ─── Resolve active AI model ─────────────────────────────────
    const getActiveModel = useCallback((): { provider: AIProvider; model: string } => {
        if (selectedModelId === 'custom') {
            return { provider: customProvider, model: customModelName };
        }
        const preset = AI_MODEL_PRESETS.find(m => m.id === selectedModelId);
        return preset ? { provider: preset.provider, model: preset.model } : { provider: 'gemini', model: 'gemini-2.0-flash' };
    }, [selectedModelId, customProvider, customModelName]);

    const handleModelChange = (id: string) => {
        setSelectedModelId(id);
        localStorage.setItem('vidra_ai_model', id);
    };

    const updateApiKey = (provider: AIProvider, key: string) => {
        setApiKeys(prev => ({ ...prev, [provider]: key }));
        localStorage.setItem(`vidra_key_${provider}`, key);
    };

    // ─── Chat ─────────────────────────────────────────────────────
    const sendChat = async () => {
        const { provider, model: modelName } = getActiveModel();
        const key = apiKeys[provider];
        if (!chatInput.trim()) return;
        if (!key.trim()) {
            setShowKeyModal(true);
            return;
        }
        if (!modelName.trim()) {
            setChatMessages(prev => [...prev, { role: 'assistant', content: 'Please select a model or enter a custom model name.' }]);
            return;
        }
        const userMsg: ChatMessage = { role: 'user', content: chatInput };
        setChatMessages(prev => [...prev, userMsg]);
        setChatInput('');
        setChatLoading(true);

        try {
            const spec = chatLang === 'vidrascript' ? VIDRA_SPEC_VIDRASCRIPT : VIDRA_SPEC_SDK;
            const history = chatMessages.map(m => `${m.role}: ${m.content}`).join('\n');
            const fullPrompt = `${spec}\n\nConversation so far:\n${history}\n\nuser: ${chatInput}\n\nGenerate the code now:`;
            let text = '';

            if (provider === 'gemini') {
                const ai = new GoogleGenAI({ apiKey: key });
                const response = await ai.models.generateContent({ model: modelName, contents: fullPrompt });
                text = response.text || 'Sorry, I could not generate code.';
            } else {
                // OpenAI-compatible
                const client = new OpenAI({ apiKey: key, dangerouslyAllowBrowser: true });
                const completion = await client.chat.completions.create({
                    model: modelName,
                    messages: [
                        { role: 'system', content: spec },
                        ...chatMessages.map(m => ({ role: m.role as 'user' | 'assistant', content: m.content })),
                        { role: 'user', content: chatInput + '\n\nGenerate the code now:' }
                    ],
                });
                text = completion.choices?.[0]?.message?.content || 'Sorry, I could not generate code.';
            }

            setChatMessages(prev => [...prev, { role: 'assistant', content: text }]);
        } catch (err: any) {
            setChatMessages(prev => [...prev, { role: 'assistant', content: `Error: ${err.message}` }]);
        } finally {
            setChatLoading(false);
            setTimeout(() => chatEndRef.current?.scrollIntoView({ behavior: 'smooth' }), 100);
        }
    };

    const applyAiCode = (code: string) => {
        // Clean markdown fences if any
        let clean = code.replace(/```[\w]*\n?/g, '').replace(/```$/g, '').trim();
        setCode(clean);
        setMode(chatLang);
        setSidebarTab('code');
        setTimeout(() => handleCompile(clean, chatLang), 200);
    };

    // ─── Creative → Code ──────────────────────────────────────────
    const creativeToVidra = (): string => {
        let out = 'project(1920, 1080, 60) {\n';
        scenes.forEach(sc => {
            out += `    scene("${sc.name}", ${sc.duration}s) {\n`;
            sc.layers.forEach(l => {
                out += `        layer("${l.id}") {\n`;
                if (l.type === 'solid') out += `            solid(${l.color})\n`;
                else if (l.type === 'text') out += `            text("${l.text}", font: "${l.font}", size: ${l.fontSize}, color: ${l.textColor})\n`;
                else if (l.type === 'image') out += `            image("${l.assetId}")\n`;
                out += `            position(${l.x}, ${l.y})\n`;
                l.animations.forEach(a => {
                    out += `            animation(${a.property}, from: ${a.from}, to: ${a.to}, duration: ${a.duration}s, easing: ${a.easing})\n`;
                });
                out += `        }\n`;
            });
            out += `    }\n`;
        });
        out += '}\n';
        return out;
    };

    const compileCreative = () => {
        const vidraCode = creativeToVidra();
        setCode(vidraCode);
        setMode('vidrascript');
        handleCompile(vidraCode, 'vidrascript');
    };

    // ─── Creative Layer Helpers ───────────────────────────────────
    const addLayer = (type: 'solid' | 'text' | 'image') => {
        const sc = scenes[activeScene];
        const newLayer: CreativeLayer = {
            id: `layer_${Date.now()}`, type, x: 960, y: 540, opacity: 1, animations: [],
            ...(type === 'solid' ? { color: '#3b82f6' } : {}),
            ...(type === 'text' ? { text: 'New Text', font: 'Inter', fontSize: 64, textColor: '#ffffff' } : {}),
            ...(type === 'image' ? { assetId: assets[0]?.id || '' } : {})
        };
        const updated = [...scenes];
        updated[activeScene] = { ...sc, layers: [...sc.layers, newLayer] };
        setScenes(updated);
        setActiveLayer(newLayer.id);
    };

    const updateLayer = (layerId: string, patch: Partial<CreativeLayer>) => {
        const updated = scenes.map((sc, i) => i !== activeScene ? sc : {
            ...sc,
            layers: sc.layers.map(l => l.id === layerId ? { ...l, ...patch } : l)
        });
        setScenes(updated);
    };

    const addScene = () => {
        setScenes(prev => [...prev, {
            id: `s${prev.length + 1}`, name: `Scene ${prev.length + 1}`, duration: 3,
            layers: [{ id: `bg_${Date.now()}`, type: 'solid', color: '#18181b', x: 960, y: 540, opacity: 1, animations: [] }]
        }]);
    };

    const addAnimation = (layerId: string) => {
        const updated = scenes.map((sc, i) => i !== activeScene ? sc : {
            ...sc,
            layers: sc.layers.map(l => l.id === layerId ? {
                ...l, animations: [...l.animations, { property: 'opacity', from: 0, to: 1, duration: 1, easing: 'easeOut' }]
            } : l)
        });
        setScenes(updated);
    };

    const updateAnim = (layerId: string, animIdx: number, patch: Partial<CreativeAnim>) => {
        const updated = scenes.map((sc, i) => i !== activeScene ? sc : {
            ...sc,
            layers: sc.layers.map(l => l.id === layerId ? {
                ...l,
                animations: l.animations.map((a, j) => j === animIdx ? { ...a, ...patch } : a)
            } : l)
        });
        setScenes(updated);
    };

    const deleteLayer = (layerId: string) => {
        const updated = scenes.map((sc, i) => i !== activeScene ? sc : {
            ...sc, layers: sc.layers.filter(l => l.id !== layerId)
        });
        setScenes(updated);
        setActiveLayer(null);
    };

    const selectedLayer = scenes[activeScene]?.layers.find(l => l.id === activeLayer);

    // ─── Render ───────────────────────────────────────────────────
    return (
        <>
            <header className="studio-header">
                <div className="logo">
                    <div className="vidra-icon"></div>
                    <h1>Vidra Studio</h1>
                    <span className="version-badge">v0.1.5</span>
                </div>
                <div className="header-actions">
                    {sidebarTab === 'code' && (
                        <label className="live-toggle" title="Auto-compile on edit (debounced 800ms)">
                            <input type="checkbox" checked={liveMode} onChange={e => setLiveMode(e.target.checked)} />
                            <span className="toggle-slider"></span>
                            <span className="toggle-label">Live</span>
                        </label>
                    )}
                    <button onClick={() => handleCompile()} disabled={engineState === 'idle'} className="btn primary">
                        ✨ Compile
                    </button>
                </div>
            </header>

            <main className="studio-main">
                {/* ─── Sidebar ────────────────────────────────── */}
                <aside className="sidebar" style={{ width: `${sidebarWidth}%` }}>
                    <nav className="sidebar-tabs">
                        {(['code', 'creative', 'chat', 'docs'] as SidebarTab[]).map(tab => (
                            <button key={tab} className={`stab ${sidebarTab === tab ? 'active' : ''}`}
                                onClick={() => setSidebarTab(tab)}>
                                {tab === 'code' && '⌨️'}
                                {tab === 'creative' && '🎨'}
                                {tab === 'chat' && '🤖'}
                                {tab === 'docs' && '📚'}
                                <span>{tab.charAt(0).toUpperCase() + tab.slice(1)}</span>
                            </button>
                        ))}
                    </nav>

                    <div className="sidebar-content">
                        {/* ─── CODE TAB ────────────────────────── */}
                        {sidebarTab === 'code' && (
                            <>
                                <div className="panel editor-panel">
                                    <div className="panel-header">
                                        <h2>Editor</h2>
                                        <div className="mode-tabs">
                                            <button className={`tab-btn ${mode === 'vidrascript' ? 'active' : ''}`}
                                                onClick={() => { setMode('vidrascript'); setCode(defaultScripts.vidrascript); }}>
                                                VidraScript
                                            </button>
                                            <button className={`tab-btn ${mode === 'sdk' ? 'active' : ''}`}
                                                onClick={() => { setMode('sdk'); setCode(defaultScripts.sdk); }}>
                                                JS SDK
                                            </button>
                                        </div>
                                    </div>
                                    <div className="editor-container">
                                        <Editor
                                            beforeMount={handleEditorWillMount}
                                            height="100%"
                                            theme="vs-dark"
                                            language={mode === 'vidrascript' ? 'vidrascript' : 'typescript'}
                                            value={code}
                                            onChange={(value) => setCode(value || '')}
                                            options={{ automaticLayout: true, minimap: { enabled: false }, scrollBeyondLastLine: false, fontSize: 14, fontFamily: 'JetBrains Mono, monospace', padding: { top: 16 } }}
                                        />
                                    </div>
                                    {compileError && <div className="compile-error">⚠️ {compileError}</div>}
                                </div>
                                <div className="panel">
                                    <div className="panel-header"><h2>Assets</h2></div>
                                    <div className="assets-container">
                                        <div className="drop-zone"
                                            onDragOver={e => e.preventDefault()}
                                            onDrop={e => { e.preventDefault(); e.dataTransfer.files && handleFiles(e.dataTransfer.files); }}>
                                            <p>Drop images here</p>
                                            <input type="file" accept="image/*" multiple style={{ display: 'none' }} id="fileInput"
                                                onChange={e => e.target.files && handleFiles(e.target.files)} />
                                            <label htmlFor="fileInput" className="btn accent" style={{ cursor: 'pointer' }}>Browse Files</label>
                                        </div>
                                        <ul className="asset-list">
                                            {assets.map((asset, i) => (
                                                <li key={i} className="asset-item">
                                                    <span className="asset-name">{asset.name}</span>
                                                    <span className="asset-id">{asset.id}</span>
                                                </li>
                                            ))}
                                        </ul>
                                    </div>
                                </div>
                            </>
                        )}

                        {/* ─── CREATIVE TAB ────────────────────── */}
                        {sidebarTab === 'creative' && (
                            <div className="creative-panel">
                                <div className="panel-header">
                                    <h2>Visual Designer</h2>
                                    <button className="btn primary" onClick={compileCreative}>▶ Preview</button>
                                </div>

                                {/* Scene bar */}
                                <div className="scene-bar">
                                    {scenes.map((sc, i) => (
                                        <button key={sc.id}
                                            className={`scene-chip ${i === activeScene ? 'active' : ''}`}
                                            onClick={() => { setActiveScene(i); setActiveLayer(null); }}>
                                            {sc.name}
                                            <span className="scene-dur">{sc.duration}s</span>
                                        </button>
                                    ))}
                                    <button className="scene-chip add" onClick={addScene}>+ Scene</button>
                                </div>

                                {/* Scene settings */}
                                <div className="creative-section">
                                    <label className="creative-label">Scene Duration (s)</label>
                                    <input type="number" className="creative-input" value={scenes[activeScene]?.duration || 3}
                                        onChange={e => {
                                            const updated = [...scenes];
                                            updated[activeScene] = { ...updated[activeScene], duration: parseFloat(e.target.value) || 1 };
                                            setScenes(updated);
                                        }} />
                                </div>

                                {/* Layers */}
                                <div className="creative-section">
                                    <div className="section-header">
                                        <label className="creative-label">Layers</label>
                                        <div className="add-layer-btns">
                                            <button className="btn-sm" onClick={() => addLayer('solid')}>+ Solid</button>
                                            <button className="btn-sm" onClick={() => addLayer('text')}>+ Text</button>
                                            <button className="btn-sm" onClick={() => addLayer('image')}>+ Image</button>
                                        </div>
                                    </div>
                                    <div className="layer-list">
                                        {scenes[activeScene]?.layers.map(l => (
                                            <div key={l.id}
                                                className={`layer-chip ${activeLayer === l.id ? 'active' : ''}`}
                                                onClick={() => setActiveLayer(l.id)}>
                                                <span className="layer-icon">
                                                    {l.type === 'solid' ? '🟦' : l.type === 'text' ? '📝' : '🖼️'}
                                                </span>
                                                <span className="layer-name">{l.id}</span>
                                                <span className="layer-type">{l.type}</span>
                                            </div>
                                        ))}
                                    </div>
                                </div>

                                {/* Layer inspector */}
                                {selectedLayer && (
                                    <div className="creative-section inspector">
                                        <div className="section-header">
                                            <label className="creative-label">Inspector: {selectedLayer.id}</label>
                                            <button className="btn-sm danger" onClick={() => deleteLayer(selectedLayer.id)}>Delete</button>
                                        </div>

                                        {selectedLayer.type === 'solid' && (
                                            <div className="prop-row">
                                                <label>Color</label>
                                                <input type="color" value={selectedLayer.color} onChange={e => updateLayer(selectedLayer.id, { color: e.target.value })} />
                                            </div>
                                        )}

                                        {selectedLayer.type === 'text' && (
                                            <>
                                                <div className="prop-row">
                                                    <label>Text</label>
                                                    <input type="text" value={selectedLayer.text}
                                                        onChange={e => updateLayer(selectedLayer.id, { text: e.target.value })} />
                                                </div>
                                                <div className="prop-row">
                                                    <label>Font Size</label>
                                                    <input type="number" value={selectedLayer.fontSize}
                                                        onChange={e => updateLayer(selectedLayer.id, { fontSize: parseInt(e.target.value) || 48 })} />
                                                </div>
                                                <div className="prop-row">
                                                    <label>Color</label>
                                                    <input type="color" value={selectedLayer.textColor}
                                                        onChange={e => updateLayer(selectedLayer.id, { textColor: e.target.value })} />
                                                </div>
                                            </>
                                        )}

                                        {selectedLayer.type === 'image' && (
                                            <div className="prop-row">
                                                <label>Asset</label>
                                                <select value={selectedLayer.assetId}
                                                    onChange={e => updateLayer(selectedLayer.id, { assetId: e.target.value })}>
                                                    {assets.map(a => <option key={a.id} value={a.id}>{a.name}</option>)}
                                                </select>
                                            </div>
                                        )}

                                        <div className="prop-row">
                                            <label>Position X</label>
                                            <input type="number" value={selectedLayer.x}
                                                onChange={e => updateLayer(selectedLayer.id, { x: parseInt(e.target.value) || 0 })} />
                                        </div>
                                        <div className="prop-row">
                                            <label>Position Y</label>
                                            <input type="number" value={selectedLayer.y}
                                                onChange={e => updateLayer(selectedLayer.id, { y: parseInt(e.target.value) || 0 })} />
                                        </div>

                                        {/* Animations */}
                                        <div className="section-header" style={{ marginTop: 16 }}>
                                            <label className="creative-label">Animations</label>
                                            <button className="btn-sm" onClick={() => addAnimation(selectedLayer.id)}>+ Add</button>
                                        </div>
                                        {selectedLayer.animations.map((a, j) => (
                                            <div key={j} className="anim-card">
                                                <div className="prop-row">
                                                    <label>Property</label>
                                                    <select value={a.property} onChange={e => updateAnim(selectedLayer.id, j, { property: e.target.value })}>
                                                        {DOCS_DATA.animations.map(an => <option key={an.property} value={an.property}>{an.property}</option>)}
                                                    </select>
                                                </div>
                                                <div className="prop-row compact">
                                                    <div><label>From</label><input type="number" step="0.1" value={a.from} onChange={e => updateAnim(selectedLayer.id, j, { from: parseFloat(e.target.value) })} /></div>
                                                    <div><label>To</label><input type="number" step="0.1" value={a.to} onChange={e => updateAnim(selectedLayer.id, j, { to: parseFloat(e.target.value) })} /></div>
                                                </div>
                                                <div className="prop-row compact">
                                                    <div><label>Duration</label><input type="number" step="0.1" value={a.duration} onChange={e => updateAnim(selectedLayer.id, j, { duration: parseFloat(e.target.value) })} /></div>
                                                    <div>
                                                        <label>Easing</label>
                                                        <select value={a.easing} onChange={e => updateAnim(selectedLayer.id, j, { easing: e.target.value })}>
                                                            {DOCS_DATA.easings.map(ea => <option key={ea.name} value={ea.name}>{ea.name}</option>)}
                                                        </select>
                                                    </div>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                )}

                                <div className="creative-section">
                                    <button className="btn" style={{ width: '100%' }} onClick={() => {
                                        setCreativeCodePreview(prev => prev ? null : creativeToVidra());
                                    }}>
                                        {creativeCodePreview ? '🔼 Hide Code' : '📋 View Generated Code'}
                                    </button>
                                    {creativeCodePreview && (
                                        <div className="code-preview-block">
                                            <pre>{creativeCodePreview}</pre>
                                            <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
                                                <button className="btn-sm" onClick={() => navigator.clipboard.writeText(creativeCodePreview)}>📋 Copy</button>
                                                <button className="btn-sm accent" onClick={() => {
                                                    setCode(creativeCodePreview);
                                                    setMode('vidrascript');
                                                    setSidebarTab('code');
                                                }}>Open in Editor</button>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}

                        {/* ─── CHAT TAB ────────────────────────── */}
                        {sidebarTab === 'chat' && (
                            <div className="chat-panel">
                                <div className="panel-header">
                                    <h2>AI Assistant</h2>
                                    <button className="btn-sm" onClick={() => setShowKeyModal(true)} title="API Keys">🔑</button>
                                </div>

                                {/* Model selector & language toggle */}
                                <div className="chat-config-bar">
                                    <select className="model-select" value={selectedModelId} onChange={e => handleModelChange(e.target.value)}>
                                        {AI_MODEL_PRESETS.map(m => (
                                            <option key={m.id} value={m.id}>{m.name}</option>
                                        ))}
                                    </select>
                                    <div className="mode-tabs compact">
                                        <button className={`tab-btn ${chatLang === 'vidrascript' ? 'active' : ''}`}
                                            onClick={() => setChatLang('vidrascript')}>Vidra</button>
                                        <button className={`tab-btn ${chatLang === 'sdk' ? 'active' : ''}`}
                                            onClick={() => setChatLang('sdk')}>TS</button>
                                    </div>
                                </div>

                                {/* Custom model config */}
                                {selectedModelId === 'custom' && (
                                    <div className="custom-model-config">
                                        <div className="prop-row">
                                            <label>Provider</label>
                                            <select value={customProvider} onChange={e => { setCustomProvider(e.target.value as AIProvider); localStorage.setItem('vidra_custom_provider', e.target.value); }}>
                                                <option value="gemini">Gemini</option>
                                                <option value="openai">OpenAI</option>
                                            </select>
                                        </div>
                                        <div className="prop-row">
                                            <label>Model Name</label>
                                            <input type="text" placeholder="e.g. gpt-4-turbo" value={customModelName}
                                                onChange={e => { setCustomModelName(e.target.value); localStorage.setItem('vidra_custom_model', e.target.value); }}
                                                className="creative-input" />
                                        </div>
                                    </div>
                                )}

                                {/* Missing API key hint */}
                                {(() => { const { provider } = getActiveModel(); return !apiKeys[provider]; })() && (
                                    <div className="api-key-hint">
                                        <span>⚠️ No API key set for <strong>{getActiveModel().provider === 'gemini' ? 'Gemini' : 'OpenAI'}</strong></span>
                                        <button className="btn-sm accent" onClick={() => setShowKeyModal(true)}>Add Key</button>
                                    </div>
                                )}

                                <div className="chat-messages">
                                    {chatMessages.length === 0 && (
                                        <div className="chat-empty">
                                            <div className="chat-empty-icon">🤖</div>
                                            <p>Ask me to create a video!</p>
                                            <div className="chat-suggestions">
                                                {[
                                                    'Create a startup pitch intro with animated text',
                                                    'Make a countdown timer from 5 to 1',
                                                    'Design a product showcase with 3 scenes'
                                                ].map((s, i) => (
                                                    <button key={i} className="suggestion-chip" onClick={() => { setChatInput(s); }}>{s}</button>
                                                ))}
                                            </div>
                                        </div>
                                    )}
                                    {chatMessages.map((msg, i) => (
                                        <div key={i} className={`chat-msg ${msg.role}`}>
                                            <div className="msg-avatar">{msg.role === 'user' ? '👤' : '✨'}</div>
                                            <div className="msg-body">
                                                {msg.role === 'assistant' ? (
                                                    <>
                                                        <pre className="msg-code">{msg.content}</pre>
                                                        <div className="msg-actions">
                                                            <button className="btn-sm" onClick={() => applyAiCode(msg.content)}>▶ Apply & Preview</button>
                                                            <button className="btn-sm" onClick={() => navigator.clipboard.writeText(msg.content)}>📋 Copy</button>
                                                        </div>
                                                    </>
                                                ) : <p>{msg.content}</p>}
                                            </div>
                                        </div>
                                    ))}
                                    {chatLoading && <div className="chat-msg assistant"><div className="msg-avatar">✨</div><div className="msg-body typing"><span></span><span></span><span></span></div></div>}
                                    <div ref={chatEndRef} />
                                </div>

                                <div className="chat-input-bar">
                                    <input type="text" placeholder="Describe a video to create..."
                                        value={chatInput}
                                        onChange={e => setChatInput(e.target.value)}
                                        onKeyDown={e => e.key === 'Enter' && sendChat()}
                                        disabled={chatLoading} />
                                    <button className="btn primary" onClick={sendChat} disabled={chatLoading || !chatInput.trim()}>
                                        {chatLoading ? '...' : '→'}
                                    </button>
                                </div>
                            </div>
                        )}

                        {/* ─── DOCS TAB ────────────────────────── */}
                        {sidebarTab === 'docs' && (
                            <div className="docs-panel">
                                <div className="panel-header"><h2>Reference</h2></div>

                                <details className="doc-section" open>
                                    <summary>🎬 Layer Types</summary>
                                    <div className="doc-grid">
                                        {DOCS_DATA.layerTypes.map(l => (
                                            <div key={l.name} className="doc-card">
                                                <strong>{l.name}</strong>
                                                <code>{l.syntax}</code>
                                                <span className="doc-desc">{l.desc}</span>
                                            </div>
                                        ))}
                                    </div>
                                </details>

                                <details className="doc-section" open>
                                    <summary>🎞️ Animatable Properties</summary>
                                    <div className="doc-grid">
                                        {DOCS_DATA.animations.map(a => (
                                            <div key={a.property} className="doc-card">
                                                <strong>{a.property}</strong>
                                                <span className="doc-desc">{a.desc}</span>
                                            </div>
                                        ))}
                                    </div>
                                </details>

                                <details className="doc-section" open>
                                    <summary>⏱️ Easing Functions</summary>
                                    <div className="doc-grid easing-grid">
                                        {DOCS_DATA.easings.map(e => (
                                            <div key={e.name} className="doc-card mini">
                                                <strong>{e.name}</strong>
                                                <span className="doc-desc">{e.desc}</span>
                                            </div>
                                        ))}
                                    </div>
                                </details>

                                <details className="doc-section">
                                    <summary>🎨 Colors</summary>
                                    <div className="doc-grid">
                                        {DOCS_DATA.colors.map(c => (
                                            <div key={c.name} className="doc-card">
                                                <strong>{c.name}</strong>
                                                <code>{c.syntax}</code>
                                                <span className="doc-desc">e.g. {c.example}</span>
                                            </div>
                                        ))}
                                    </div>
                                </details>

                                <details className="doc-section">
                                    <summary>✨ Effects</summary>
                                    <div className="doc-grid">
                                        {DOCS_DATA.effects.map(e => (
                                            <div key={e.name} className="doc-card">
                                                <strong>{e.name}</strong>
                                                <code>{e.syntax}</code>
                                                <span className="doc-desc">{e.desc}</span>
                                            </div>
                                        ))}
                                    </div>
                                </details>
                            </div>
                        )}
                    </div>
                </aside>

                <div className={`resizer ${isResizing ? 'resizing' : ''}`}
                    onMouseDown={e => { e.preventDefault(); setIsResizing(true); }} />

                {/* ─── Player ─────────────────────────────────── */}
                <section className="player-section" style={{ flex: 1 }}>
                    <div className="player-wrapper">
                        <div className="player-container">
                            <canvas ref={canvasRef} id="vidraCanvas" width="1280" height="720"></canvas>
                        </div>

                        <div className="player-controls">
                            <button onClick={() => engineRef.current?.play()} disabled={engineState === 'loading' || engineState === 'idle'} className="btn icon-btn">
                                <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
                            </button>
                            <button onClick={() => engineRef.current?.pause()} disabled={engineState !== 'playing'} className="btn icon-btn">
                                <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" /></svg>
                            </button>
                            <button onClick={() => engineRef.current?.stop()} disabled={engineState === 'loading' || engineState === 'idle'} className="btn icon-btn">
                                <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor"><path d="M6 6h12v12H6z" /></svg>
                            </button>
                            <div className="time-display"><span>{currentTimeStr}</span> / <span>{totalTimeStr}</span></div>
                            <div className="progress-bar" onClick={handleSeek}>
                                <div className="progress-fill" style={{ width: `${progress}%` }}></div>
                            </div>
                        </div>

                        <div className="status-bar">
                            <div className={`status-badge ${engineState === 'ready' || engineState === 'playing' || engineState === 'paused' ? 'ready' : ''}`}>
                                {liveMode && sidebarTab === 'code' && <span className="live-dot"></span>}
                                {status}
                            </div>
                            <div className="meta-info">{metaInfo}</div>
                        </div>
                    </div>
                </section>
            </main>

            {/* ─── API Key Modal ──────────────────────────── */}
            {showKeyModal && (
                <div className="modal-overlay" onClick={() => setShowKeyModal(false)}>
                    <div className="modal-content" onClick={e => e.stopPropagation()}>
                        <div className="modal-header">
                            <h2>🔑 API Keys</h2>
                            <button className="btn icon-btn" onClick={() => setShowKeyModal(false)}>✕</button>
                        </div>
                        <div className="modal-body">
                            <div className="key-section">
                                <div className="key-section-header">
                                    <span className="provider-badge gemini">Gemini</span>
                                    <a href="https://aistudio.google.com/apikey" target="_blank" rel="noreferrer" className="key-link">Get Key →</a>
                                </div>
                                <input type="password" placeholder="Gemini API Key..." className="creative-input"
                                    value={apiKeys.gemini}
                                    onChange={e => updateApiKey('gemini', e.target.value)} />
                            </div>
                            <div className="key-section">
                                <div className="key-section-header">
                                    <span className="provider-badge openai">OpenAI</span>
                                    <a href="https://platform.openai.com/api-keys" target="_blank" rel="noreferrer" className="key-link">Get Key →</a>
                                </div>
                                <input type="password" placeholder="OpenAI API Key..." className="creative-input"
                                    value={apiKeys.openai}
                                    onChange={e => updateApiKey('openai', e.target.value)} />
                            </div>
                            <p className="key-note">Keys are stored in your browser's localStorage only. Never sent to our servers.</p>
                        </div>
                    </div>
                </div>
            )}
        </>
    );
}
