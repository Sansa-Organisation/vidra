// useAI — typed wrapper around POST /api/ai/chat (Task 10.2)
import { useState, useCallback } from 'react';

export interface AiMessage {
    role: 'user' | 'assistant' | 'system';
    content: string;
}

export function useAI(rest: (method: string, path: string, body?: unknown) => Promise<unknown>) {
    const [messages, setMessages] = useState<AiMessage[]>([]);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const sendMessage = useCallback(async (content: string, model: string = 'gpt-4') => {
        const newMsg: AiMessage = { role: 'user', content };
        const newHistory = [...messages, newMsg];
        setMessages(newHistory);
        setLoading(true);
        setError(null);

        try {
            const res = await rest('POST', '/api/ai/chat', {
                messages: newHistory,
                model
            }) as { response?: string, error?: string };

            if (res.error) throw new Error(res.error);
            if (res.response) {
                setMessages([...newHistory, { role: 'assistant', content: res.response }]);
            }
        } catch (e) {
            setError(String(e));
        } finally {
            setLoading(false);
        }
    }, [messages, rest]);

    const clearChat = useCallback(() => {
        setMessages([]);
        setError(null);
    }, []);

    return { messages, loading, error, sendMessage, clearChat };
}
