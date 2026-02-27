import { useState, useEffect } from 'react';

export default function RenderMode() {
    const [irJSON, setIrJSON] = useState<string>('Loading project.json...');

    useEffect(() => {
        fetch('/project.json')
            .then(res => {
                if (!res.ok) throw new Error("Could not find project.json. Did you run 'npm run build:video'?");
                return res.text();
            })
            .then(text => {
                try {
                    setIrJSON(JSON.stringify(JSON.parse(text), null, 2));
                } catch {
                    setIrJSON(text);
                }
            })
            .catch(err => setIrJSON(err.message));
    }, []);

    return (
        <div className="max-w-5xl mx-auto flex flex-col gap-6">
            <div className="bg-vd-panel border border-vd-border rounded-lg p-6">
                <h2 className="text-xl font-bold mb-2">Build & Render</h2>
                <p className="text-vd-dim mb-4">
                    This mode shows the JSON Intermediate Representation (IR) generated programmatically by the SDK in <code>src/video.ts</code>.
                </p>

                <div className="bg-vd-bg p-4 rounded border border-vd-border mb-4">
                    <p className="text-sm text-vd-dim mb-2">Build IR to public/project.json:</p>
                    <code className="text-vd-accent block">npm run build:video</code>

                    <p className="text-sm text-vd-dim mt-4 mb-2">Render raw .vidra to MP4 (requires vidra CLI):</p>
                    <code className="text-vd-accent block">vidra render video.vidra -o output.mp4</code>
                </div>
            </div>

            <div className="bg-[#0d1117] border border-vd-border rounded-lg p-4 overflow-auto max-h-[60vh]">
                <h3 className="text-sm font-bold text-vd-dim mb-2 uppercase tracking-wider">public/project.json</h3>
                <pre className="text-xs text-green-400 font-mono">
                    {irJSON}
                </pre>
            </div>
        </div>
    );
}
