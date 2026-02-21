"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
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
    const serverOptions = {
        run: { command: command, args: args, options: { cwd: vscode_1.workspace.workspaceFolders?.[0].uri.fsPath } },
        debug: { command: command, args: args, options: { cwd: vscode_1.workspace.workspaceFolders?.[0].uri.fsPath } }
    };
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'vidra' }],
        synchronize: {
            // Notify the server about file changes to '.vidra' files
            fileEvents: vscode_1.workspace.createFileSystemWatcher('**/*.vidra')
        }
    };
    // Create the language client and start the client.
    client = new node_1.LanguageClient('vidraLanguageServer', 'Vidra Language Server', serverOptions, clientOptions);
    client.start();
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
//# sourceMappingURL=extension.js.map