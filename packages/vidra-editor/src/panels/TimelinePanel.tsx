// Timeline panel (Task 9.5)
import { useProjectStore } from '../hooks/useProject';
import type { ProjectMeta } from '../hooks/useBackend';

interface TimelinePanelProps { meta: ProjectMeta | null }

export function TimelinePanel({ meta }: TimelinePanelProps) {
    const { scenes, frame, setFrame, playing, setPlaying, selectedLayerId, selectLayer } = useProjectStore();
    const totalFrames = meta?.total_frames ?? 1;

    return (
        <div className="timeline-panel">
            <div className="timeline-header">
                <button className="play-btn" onClick={() => setPlaying(!playing)}>
                    {playing ? '⏸' : '▶'}
                </button>
                <span>Frame {frame} / {totalFrames - 1}</span>
                <input
                    type="range" min={0} max={totalFrames - 1} value={frame}
                    onChange={e => setFrame(Number(e.target.value))}
                    style={{ flex: 1, accentColor: 'var(--accent-blue)' }}
                />
            </div>
            <div className="timeline-tracks">
                {scenes.map(scene => (
                    <div key={scene.id}>
                        <div className="timeline-row">
                            <div className="row-label" style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
                                🎬 {scene.name || scene.id}
                            </div>
                        </div>
                        {scene.layers.map(layer => (
                            <div className="timeline-row" key={layer.id}>
                                <div className="row-label">{layer.label || layer.id}</div>
                                <div
                                    className={`row-bar ${selectedLayerId === layer.id ? 'selected' : ''}`}
                                    style={{ width: `${Math.max(60, (scene.duration_frames / totalFrames) * 400)}px` }}
                                    onClick={() => selectLayer(layer.id)}
                                />
                            </div>
                        ))}
                    </div>
                ))}
                {scenes.length === 0 && (
                    <div style={{ padding: '12px', color: 'var(--text-muted)', fontSize: '12px' }}>
                        No scenes loaded
                    </div>
                )}
            </div>
        </div>
    );
}
