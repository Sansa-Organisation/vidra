// Scene Graph panel — tree view of scenes/layers (Task 9.6)
import { useProjectStore } from '../hooks/useProject';

const LAYER_ICONS: Record<string, string> = {
    text: '🔤', image: '🖼️', video: '🎥', shape: '◼', web: '🌐',
    audio: '🔊', tts: '🗣️', waveform: '〰️', spritesheet: '🎞️',
};

export function SceneGraphPanel() {
    const { scenes, selectedLayerId, selectLayer } = useProjectStore();

    return (
        <div className="panel-content scene-tree">
            {scenes.map(scene => (
                <div key={scene.id}>
                    <div className="tree-item scene">
                        <span className="icon">🎬</span>
                        {scene.name || scene.id}
                    </div>
                    {scene.layers.map(layer => (
                        <div
                            key={layer.id}
                            className={`tree-item ${selectedLayerId === layer.id ? 'selected' : ''}`}
                            style={{ paddingLeft: '24px' }}
                            onClick={() => selectLayer(layer.id)}
                        >
                            <span className="icon">{LAYER_ICONS[layer.content_type] || '📦'}</span>
                            {layer.label || layer.id}
                        </div>
                    ))}
                </div>
            ))}
            {scenes.length === 0 && (
                <div style={{ padding: '8px', color: 'var(--text-muted)', fontSize: '12px' }}>
                    Open a project to see scenes
                </div>
            )}
        </div>
    );
}
