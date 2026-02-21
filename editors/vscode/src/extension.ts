import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    // Determine the path to the vidra LSP binary
    // In dev, assuming it's built and installed via cargo or in the target dir
    // For production, the extension would likely package the binary or download it.
    // Assuming cargo is in path and vidra is installed or we use the local debug build for now.

    const isWindows = process.platform === 'win32';
    // Use cargo run if we just want to execute it locally.
    // Or assume people have `vidra` in their PATH.
    // Let's assume it's in the PATH for now, users install via `cargo install --path crates/vidra-cli`
    const command = isWindows ? 'vidra.exe' : 'cargo';
    const args = ['run', '--bin', 'vidra', '--', 'lsp'];

    const serverOptions: ServerOptions = {
        run: { command: command, args: args, options: { cwd: workspace.workspaceFolders?.[0].uri.fsPath } },
        debug: { command: command, args: args, options: { cwd: workspace.workspaceFolders?.[0].uri.fsPath } }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'vidra' }],
        synchronize: {
            // Notify the server about file changes to '.vidra' files
            fileEvents: workspace.createFileSystemWatcher('**/*.vidra')
        }
    };

    // Create the language client and start the client.
    client = new LanguageClient(
        'vidraLanguageServer',
        'Vidra Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
