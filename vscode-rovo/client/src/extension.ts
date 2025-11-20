import * as vscode from 'vscode';
import * as child_process from 'child_process';
import { promisify } from 'util';
import * as fs from 'fs/promises';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable
} from 'vscode-languageclient/node';

const execAsync = promisify(child_process.exec);

let client: LanguageClient;

export async function activate(context: vscode.ExtensionContext) {
    const outputChannel = vscode.window.createOutputChannel('Rovo LSP');
    outputChannel.appendLine('Rovo extension activating...');

    // Setup text decorations for Rovo annotations (works alongside rust-analyzer)
    const annotationDecoration = vscode.window.createTextEditorDecorationType({
        color: '#4EC9B0'
    });
    const statusCodeDecoration = vscode.window.createTextEditorDecorationType({
        color: '#B5CEA8'
    });
    const securitySchemeDecoration = vscode.window.createTextEditorDecorationType({
        color: '#9CDCFE'
    });
    const tagValueDecoration = vscode.window.createTextEditorDecorationType({
        color: '#CE9178'
    });

    let decorationDebugLogged = false;

    function updateDecorations(editor: vscode.TextEditor) {
        if (!editor || editor.document.languageId !== 'rust') {
            return;
        }

        const text = editor.document.getText();
        const lines = text.split('\n');

        const annotationRanges: vscode.Range[] = [];
        const statusCodeRanges: vscode.Range[] = [];
        const securitySchemeRanges: vscode.Range[] = [];
        const tagValueRanges: vscode.Range[] = [];

        // Find #[rovo] attributes
        const rovoLines: number[] = [];
        for (let i = 0; i < lines.length; i++) {
            if (lines[i].includes('#[rovo]')) {
                rovoLines.push(i);
            }
        }

        if (rovoLines.length === 0) {
            if (!decorationDebugLogged) {
                outputChannel.appendLine(`No #[rovo] found in ${editor.document.fileName}`);
                decorationDebugLogged = true;
            }
            return;
        }

        if (!decorationDebugLogged) {
            outputChannel.appendLine(`Found ${rovoLines.length} #[rovo] attribute(s)`);
            decorationDebugLogged = true;
        }

        // For each #[rovo], highlight annotations in doc comments above
        for (const rovoLine of rovoLines) {
            let docStartLine = rovoLine - 1;
            while (docStartLine >= 0 && lines[docStartLine].match(/^\s*\/\/\//)) {
                docStartLine--;
            }
            docStartLine++;

            for (let i = docStartLine; i < rovoLine; i++) {
                const line = lines[i];

                // Highlight @annotations
                const annotationRegex = /@(response|tag|security|example|id|hidden|rovo-ignore)\b/g;
                let match;
                while ((match = annotationRegex.exec(line)) !== null) {
                    const startPos = new vscode.Position(i, match.index);
                    const endPos = new vscode.Position(i, match.index + match[0].length);
                    annotationRanges.push(new vscode.Range(startPos, endPos));
                }

                // Highlight tag values
                const tagValueRegex = /@(?:tag|id)\s+(\w+)/g;
                while ((match = tagValueRegex.exec(line)) !== null) {
                    const startPos = new vscode.Position(i, match.index + match[0].indexOf(match[1]));
                    const endPos = new vscode.Position(i, startPos.character + match[1].length);
                    tagValueRanges.push(new vscode.Range(startPos, endPos));
                }

                // Highlight status codes
                const statusCodeRegex = /\b([1-5][0-9]{2})\b/g;
                while ((match = statusCodeRegex.exec(line)) !== null) {
                    const startPos = new vscode.Position(i, match.index);
                    const endPos = new vscode.Position(i, match.index + match[0].length);
                    statusCodeRanges.push(new vscode.Range(startPos, endPos));
                }

                // Highlight security schemes
                const securitySchemeRegex = /\b(bearer|basic|apiKey|oauth2)\b/g;
                while ((match = securitySchemeRegex.exec(line)) !== null) {
                    const startPos = new vscode.Position(i, match.index);
                    const endPos = new vscode.Position(i, match.index + match[0].length);
                    securitySchemeRanges.push(new vscode.Range(startPos, endPos));
                }
            }
        }

        if (!decorationDebugLogged) {
            outputChannel.appendLine(`Decorations: ${annotationRanges.length} annotations, ${statusCodeRanges.length} codes, ${securitySchemeRanges.length} schemes, ${tagValueRanges.length} values`);
            decorationDebugLogged = true;
        }

        editor.setDecorations(annotationDecoration, annotationRanges);
        editor.setDecorations(statusCodeDecoration, statusCodeRanges);
        editor.setDecorations(securitySchemeDecoration, securitySchemeRanges);
        editor.setDecorations(tagValueDecoration, tagValueRanges);
    }

    // Update decorations when editor changes
    context.subscriptions.push(
        vscode.window.onDidChangeActiveTextEditor(editor => {
            if (editor) {
                updateDecorations(editor);
            }
        })
    );

    // Update decorations when document changes
    context.subscriptions.push(
        vscode.workspace.onDidChangeTextDocument(event => {
            const editor = vscode.window.activeTextEditor;
            if (editor && event.document === editor.document) {
                updateDecorations(editor);
            }
        })
    );

    // Reapply decorations periodically to persist over rust-analyzer
    const reapplyInterval = setInterval(() => {
        if (vscode.window.activeTextEditor) {
            updateDecorations(vscode.window.activeTextEditor);
        }
    }, 1000);

    context.subscriptions.push({
        dispose: () => clearInterval(reapplyInterval)
    });

    // Apply decorations on active editor
    if (vscode.window.activeTextEditor) {
        updateDecorations(vscode.window.activeTextEditor);
    }

    try {
        // Get configuration
        const config = vscode.workspace.getConfiguration('rovo');
        const serverPath = config.get<string>('serverPath', 'rovo-lsp');
        const autoInstall = config.get<boolean>('autoInstall', true);

        outputChannel.appendLine(`Server path: ${serverPath}`);
        outputChannel.appendLine(`Auto install: ${autoInstall}`);

        // Check if server is available
        outputChannel.appendLine('Checking if rovo-lsp is available...');
        const actualServerPath = await checkServerAvailable(serverPath);

        if (!actualServerPath) {
            outputChannel.appendLine(`rovo-lsp not found at: ${serverPath}`);

            if (autoInstall) {
                const shouldInstall = await promptInstall();
                if (shouldInstall) {
                    const installed = await installServer(outputChannel);
                    if (!installed) {
                        vscode.window.showErrorMessage(
                            'Failed to install rovo-lsp. Please install manually: cargo install rovo-lsp'
                        );
                        return;
                    }
                    // Re-check server path after installation
                    const installedServerPath = await checkServerAvailable(serverPath);
                    if (!installedServerPath) {
                        vscode.window.showErrorMessage(
                            'rovo-lsp installed but not found in PATH. Please restart VS Code or check your installation.'
                        );
                        return;
                    }
                    outputChannel.appendLine(`Server installed and found at: ${installedServerPath}`);
                    await startLanguageServer(installedServerPath, config, outputChannel, context);
                    return;
                } else {
                    vscode.window.showInformationMessage(
                        'rovo-lsp not installed. Install it with: cargo install rovo-lsp'
                    );
                    return;
                }
            } else {
                vscode.window.showWarningMessage(
                    'rovo-lsp not found. Please install it: cargo install rovo-lsp'
                );
                return;
            }
        } else {
            outputChannel.appendLine(`Server found at: ${actualServerPath}`);
            await startLanguageServer(actualServerPath, config, outputChannel, context);
        }

    } catch (error) {
        outputChannel.appendLine(`Error activating extension: ${error}`);
        vscode.window.showErrorMessage(`Failed to activate Rovo LSP: ${error}`);
    }
}

async function checkServerAvailable(serverPath: string): Promise<string | null> {
    try {
        const command = process.platform === 'win32' ? 'where' : 'which';
        const result = await execAsync(`${command} ${serverPath}`);
        return result.stdout.trim();
    } catch {
        // Not in PATH, try common cargo bin locations
        const cargoBinPaths = [
            `${process.env.HOME}/.cargo/bin/${serverPath}`,
            `/home/${process.env.USER}/.cargo/bin/${serverPath}`,
        ];

        for (const binPath of cargoBinPaths) {
            try {
                await fs.access(binPath, fs.constants.F_OK);
                return binPath;
            } catch {
                continue;
            }
        }

        return null;
    }
}

async function promptInstall(): Promise<boolean> {
    const choice = await vscode.window.showInformationMessage(
        'rovo-lsp is not installed. Would you like to install it now via cargo?',
        'Yes',
        'No'
    );
    return choice === 'Yes';
}

async function installServer(outputChannel: vscode.OutputChannel): Promise<boolean> {
    return vscode.window.withProgress(
        {
            location: vscode.ProgressLocation.Notification,
            title: 'Installing rovo-lsp',
            cancellable: false
        },
        async (progress) => {
            try {
                outputChannel.appendLine('Starting installation: cargo install rovo-lsp');
                progress.report({ message: 'Running cargo install...' });

                // Check if cargo is available
                try {
                    await execAsync('cargo --version');
                } catch {
                    vscode.window.showErrorMessage(
                        'Cargo not found. Please install Rust from https://rustup.rs/'
                    );
                    return false;
                }

                // Install rovo-lsp
                const { stdout, stderr } = await execAsync('cargo install rovo-lsp', {
                    maxBuffer: 10 * 1024 * 1024 // 10MB buffer for cargo output
                });

                outputChannel.appendLine(stdout);
                if (stderr) {
                    outputChannel.appendLine(stderr);
                }

                outputChannel.appendLine('Installation completed successfully');
                vscode.window.showInformationMessage('rovo-lsp installed successfully!');
                return true;

            } catch (error: any) {
                outputChannel.appendLine(`Installation failed: ${error.message}`);
                if (error.stdout) outputChannel.appendLine(error.stdout);
                if (error.stderr) outputChannel.appendLine(error.stderr);
                return false;
            }
        }
    );
}

async function startLanguageServer(
    serverPath: string,
    config: vscode.WorkspaceConfiguration,
    outputChannel: vscode.OutputChannel,
    context: vscode.ExtensionContext
) {
    outputChannel.appendLine(`Starting rovo-lsp server at: ${serverPath}`);

    // Server executable configuration
    const serverExecutable: Executable = {
        command: serverPath,
        args: []
    };

    const serverOptions: ServerOptions = {
        run: serverExecutable,
        debug: serverExecutable
    };

    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'rust' }
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.rs')
        },
        outputChannel: outputChannel,
        initializationOptions: {}
    };

    // Create and start the client
    client = new LanguageClient(
        'rovoLsp',
        'Rovo Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client (this will also launch the server)
    await client.start();

    outputChannel.appendLine('Rovo LSP server started successfully');

    // Register for disposal
    context.subscriptions.push(client);
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
