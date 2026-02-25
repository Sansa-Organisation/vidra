// useMcp — typed wrapper around POST /api/mcp/invoke (Task 10.5)
import { useState, useCallback } from 'react';

export function useMcp(rest: (method: string, path: string, body?: unknown) => Promise<unknown>) {
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [result, setResult] = useState<string | null>(null);

    const invoke = useCallback(async (name: string, args: Record<string, unknown> = {}) => {
        setLoading(true);
        setError(null);
        try {
            const res = await rest('POST', '/api/mcp/invoke', { name, arguments: args }) as { result: string };
            setResult(res.result);
            return res.result;
        } catch (e) {
            const msg = String(e);
            setError(msg);
            return null;
        } finally {
            setLoading(false);
        }
    }, [rest]);

    return { invoke, loading, error, result };
}
