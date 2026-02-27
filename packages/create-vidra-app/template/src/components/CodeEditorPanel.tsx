import { useState, useEffect } from 'react';
import { Code, Terminal } from 'lucide-react';

export default function CodeEditorPanel() {
    const [code, setCode] = useState<string>('// Loading src/video.ts...');

    useEffect(() => {
        fetch('/src/video.ts')
            .then(r => r.text())
            .then(text => setCode(text))
            .catch(() => setCode('/* Run `npm run dev` to see your code here */'));
    }, []);

    return (
        <div className="h-full flex flex-col pt-4">
            <div className="mb-4">
                <h2 className="text-xl font-bold flex items-center gap-2">
                    <Code className="text-vd-accent" /> Source Code
                </h2>
                <p className="text-vd-dim text-sm max-w-2xl mt-1">
                    This is your project's main entry point, <code className="text-[#d2a8ff]">src/video.ts</code>. Editing this file in your IDE and running the build script turns this Typescript code into a `.vidra` intermediate representation.
                </p>
            </div>

            <div className="flex-1 bg-[#0d1117] rounded-lg border border-[#30363d] shadow-2xl flex flex-col overflow-hidden">
                <div className="bg-[#161b22] px-4 py-2 border-b border-[#30363d] flex gap-2">
                    <span className="text-xs font-mono text-[#8b949e]">video.ts</span>
                </div>
                <div className="p-4 overflow-auto flex-1 font-mono text-[13px] text-[#e6edf3] whitespace-pre leading-relaxed select-text font-light">
                    {code}
                </div>
            </div>
        </div>
    );
}
