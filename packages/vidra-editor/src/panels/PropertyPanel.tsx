// Property Inspector panel (Task 9.7)
import { useProjectStore } from '../hooks/useProject';

interface Props {
    onPatch: (layerId: string, props: Record<string, unknown>) => void;
}

export function PropertyPanel({ onPatch }: Props) {
    const { selectedLayerId, scenes } = useProjectStore();

    const layer = scenes.flatMap(s => s.layers).find(l => l.id === selectedLayerId);

    if (!layer) {
        return (
            <div className="panel-content" style={{ color: 'var(--text-muted)', fontSize: '12px', padding: '12px' }}>
                Select a layer to view its properties.
            </div>
        );
    }

    return (
        <div className="panel-content">
            <div className="prop-group">
                <div className="prop-group-title">Layer</div>
                <div className="prop-row">
                    <span className="prop-label">ID</span>
                    <input className="prop-input" value={layer.id} readOnly />
                </div>
                <div className="prop-row">
                    <span className="prop-label">Type</span>
                    <input className="prop-input" value={layer.content_type} readOnly />
                </div>
            </div>

            <div className="prop-group">
                <div className="prop-group-title">Transform</div>
                <div className="prop-row">
                    <span className="prop-label">X</span>
                    <input className="prop-input" type="number" defaultValue={0}
                        onBlur={e => onPatch(layer.id, { x: Number(e.target.value) })} />
                </div>
                <div className="prop-row">
                    <span className="prop-label">Y</span>
                    <input className="prop-input" type="number" defaultValue={0}
                        onBlur={e => onPatch(layer.id, { y: Number(e.target.value) })} />
                </div>
                <div className="prop-row">
                    <span className="prop-label">Opacity</span>
                    <input className="prop-slider" type="range" min={0} max={1} step={0.01} defaultValue={1}
                        onChange={e => onPatch(layer.id, { opacity: Number(e.target.value) })} />
                </div>
                <div className="prop-row">
                    <span className="prop-label">Scale</span>
                    <input className="prop-input" type="number" step={0.1} defaultValue={1}
                        onBlur={e => onPatch(layer.id, { scale: Number(e.target.value) })} />
                </div>
                <div className="prop-row">
                    <span className="prop-label">Rotation</span>
                    <input className="prop-input" type="number" step={1} defaultValue={0}
                        onBlur={e => onPatch(layer.id, { rotation: Number(e.target.value) })} />
                </div>
            </div>
        </div>
    );
}
