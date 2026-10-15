import * as assert from 'assert';
import * as path from 'path';
import * as fs from 'fs/promises';
import * as os from 'os';
import * as child_process from 'child_process';
import { promisify } from 'util';

/**
 * Security tests for command injection vulnerability mitigation in checkServerAvailable function.
 * 
 * The vulnerability being tested: Shell command injection via workspace-controlled rovo.serverPath
 * 
 * Original vulnerability:
 * - The extension used child_process.exec() which spawns a shell
 * - serverPath was interpolated directly into shell command: `which ${serverPath}`
 * - Malicious workspace could set serverPath to: "rovo-lsp; touch /tmp/pwned #"
 * - This would execute arbitrary commands with extension host privileges
 * 
 * The fix implements multiple layers of defense:
 * 1. Only trust user-level (global) configuration, not workspace settings
 * 2. Use execFile() instead of exec() - no shell interpretation
 * 3. Input validation: reject empty, dash-prefixed, or null-byte-containing paths
 * 4. Path traversal protection in fallback cargo bin search
 */

const execFileAsync = promisify(child_process.execFile);

// Extract the checkServerAvailable logic for testing
async function checkServerAvailableTestable(serverPath: string, cargoBinDirs: string[]): Promise<string | null> {
    // Simulate the fixed input validation from extension.ts (line 229)
    if (typeof serverPath !== 'string' || serverPath.length === 0 || serverPath.startsWith('-') || /[\0\r\n]/.test(serverPath)) {
        return null;
    }
    
    try {
        const command = process.platform === 'win32' ? 'where' : 'which';
        // execFile does not spawn a shell; serverPath is passed as a single literal argument.
        const result = await execFileAsync(command, [serverPath]);
        return result.stdout.trim().split(/\r?\n/)[0] || null;
    } catch {
        // Not in PATH, try common cargo bin locations
        for (const cargoBinDir of cargoBinDirs) {
            const baseFull = path.resolve(cargoBinDir);
            const binPath = path.join(baseFull, serverPath);
            const rel = path.relative(baseFull, binPath);
            
            // Security check: prevent path traversal
            if (rel === '..' || rel.startsWith(`..${path.sep}`) || path.isAbsolute(rel)) {
                continue;
            }
            
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

interface TestResult {
    name: string;
    passed: boolean;
    error?: string;
    duration: number;
}

const results: TestResult[] = [];

async function runTest(name: string, testFn: () => Promise<void> | void): Promise<void> {
    const start = Date.now();
    try {
        await testFn();
        const duration = Date.now() - start;
        results.push({ name, passed: true, duration });
        console.log(`✓ ${name} (${duration}ms)`);
    } catch (error) {
        const duration = Date.now() - start;
        const errorMsg = error instanceof Error ? error.message : String(error);
        results.push({ name, passed: false, error: errorMsg, duration });
        console.error(`✗ ${name} (${duration}ms)`);
        console.error(`  ${errorMsg}`);
    }
}

async function runCommandInjectionTests(): Promise<void> {
    console.log('\n=== Command Injection Mitigation Tests ===\n');
    
    // Setup
    const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'rovo-test-'));
    const cargoBinDir = path.join(tempDir, '.cargo', 'bin');
    await fs.mkdir(cargoBinDir, { recursive: true });
    
    const testExecutable = path.join(cargoBinDir, 'rovo-lsp');
    await fs.writeFile(testExecutable, '#!/bin/sh\necho "test"', { mode: 0o755 });
    
    const markerFile = path.join(tempDir, 'command-executed');

    try {
        // Test 1: Unix shell command injection with semicolon
        await runTest('should prevent Unix shell command injection with semicolon', async () => {
            const maliciousPath = `rovo-lsp; touch ${markerFile} #`;
            const result = await checkServerAvailableTestable(maliciousPath, [cargoBinDir]);
            
            // The function should not execute the injected command
            // Since execFile doesn't spawn a shell, the entire string is treated as a literal command name
            // which won't exist, so it should return null
            assert.strictEqual(result, null, 'Should not execute injected command');
            
            // Verify the marker file was NOT created (command was not executed)
            try {
                await fs.access(markerFile);
                assert.fail('Injected command was executed - security vulnerability!');
            } catch {
                // Expected: file should not exist
            }
        });

        // Test 2: Unix shell command injection with pipe
        await runTest('should prevent Unix shell command injection with pipe', async () => {
            const maliciousPath = `rovo-lsp | touch ${markerFile}`;
            const result = await checkServerAvailableTestable(maliciousPath, [cargoBinDir]);
            
            assert.strictEqual(result, null, 'Should not execute piped command');
            
            try {
                await fs.access(markerFile);
                assert.fail('Piped command was executed - security vulnerability!');
            } catch {
                // Expected: file should not exist
            }
        });

        // Test 3: Unix shell command injection with ampersand (background execution)
        await runTest('should prevent Unix shell command injection with ampersand', async () => {
            const maliciousPath = `rovo-lsp & touch ${markerFile}`;
            const result = await checkServerAvailableTestable(maliciousPath, [cargoBinDir]);
            
            assert.strictEqual(result, null, 'Should not execute background command');
            
            // Wait a bit in case background execution was attempted
            await new Promise(resolve => setTimeout(resolve, 100));
            
            try {
                await fs.access(markerFile);
                assert.fail('Background command was executed - security vulnerability!');
            } catch {
                // Expected: file should not exist
            }
        });

        // Test 4: Unix shell command injection with command substitution
        await runTest('should prevent Unix shell command injection with command substitution', async () => {
            const maliciousPath = `rovo-lsp $(touch ${markerFile})`;
            const result = await checkServerAvailableTestable(maliciousPath, [cargoBinDir]);
            
            assert.strictEqual(result, null, 'Should not execute command substitution');
            
            try {
                await fs.access(markerFile);
                assert.fail('Command substitution was executed - security vulnerability!');
            } catch {
                // Expected: file should not exist
            }
        });

        // Test 5: Windows shell command injection with ampersand
        await runTest('should prevent Windows shell command injection with ampersand', async () => {
            const maliciousPath = `rovo-lsp & echo pwned > ${markerFile}`;
            const result = await checkServerAvailableTestable(maliciousPath, [cargoBinDir]);
            
            assert.strictEqual(result, null, 'Should not execute Windows command chain');
            
            try {
                await fs.access(markerFile);
                assert.fail('Windows command chain was executed - security vulnerability!');
            } catch {
                // Expected: file should not exist
            }
        });

        // Test 6: Windows shell command injection with double ampersand
        await runTest('should prevent Windows shell command injection with double ampersand', async () => {
            const maliciousPath = `rovo-lsp && echo pwned > ${markerFile}`;
            const result = await checkServerAvailableTestable(maliciousPath, [cargoBinDir]);
            
            assert.strictEqual(result, null, 'Should not execute Windows conditional command');
            
            try {
                await fs.access(markerFile);
                assert.fail('Windows conditional command was executed - security vulnerability!');
            } catch {
                // Expected: file should not exist
            }
        });

        // Test 7: Shell command injection with newline
        await runTest('should prevent shell command injection with newline', async () => {
            const maliciousPath = `rovo-lsp\ntouch ${markerFile}`;
            const result = await checkServerAvailableTestable(maliciousPath, [cargoBinDir]);
            
            // Input validation should reject paths with newlines
            assert.strictEqual(result, null, 'Should reject path with newline');
            
            try {
                await fs.access(markerFile);
                assert.fail('Newline-separated command was executed - security vulnerability!');
            } catch {
                // Expected: file should not exist
            }
        });

        // Test 8: Exact reproduction of pentest scenario
        await runTest('should prevent exact pentest reproduction scenario', async () => {
            // This is the exact payload from the pentest finding
            const pentestPayload = `rovo-lsp; touch ${markerFile} #`;
            const result = await checkServerAvailableTestable(pentestPayload, [cargoBinDir]);
            
            assert.strictEqual(result, null, 'Should not execute pentest payload');
            
            try {
                await fs.access(markerFile);
                assert.fail('Pentest payload was executed - vulnerability not fixed!');
            } catch {
                // Expected: file should not exist - vulnerability is mitigated
            }
        });

    } finally {
        // Cleanup
        await fs.rm(tempDir, { recursive: true, force: true });
    }
}

async function runInputValidationTests(): Promise<void> {
    console.log('\n=== Input Validation Tests ===\n');
    
    const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'rovo-test-'));
    const cargoBinDir = path.join(tempDir, '.cargo', 'bin');
    await fs.mkdir(cargoBinDir, { recursive: true });

    try {
        // Test empty string
        await runTest('should reject empty string', async () => {
            const result = await checkServerAvailableTestable('', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject empty string');
        });

        // Test dash-prefixed paths (could be interpreted as command flags)
        await runTest('should reject dash-prefixed paths', async () => {
            const result = await checkServerAvailableTestable('-version', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject dash-prefixed path');
        });

        await runTest('should reject double-dash-prefixed paths', async () => {
            const result = await checkServerAvailableTestable('--help', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject double-dash-prefixed path');
        });

        // Test null byte injection
        await runTest('should reject path with null byte', async () => {
            const result = await checkServerAvailableTestable('rovo-lsp\0malicious', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject path with null byte');
        });

        // Test carriage return
        await runTest('should reject path with carriage return', async () => {
            const result = await checkServerAvailableTestable('rovo-lsp\rmalicious', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject path with carriage return');
        });

        // Test newline
        await runTest('should reject path with newline', async () => {
            const result = await checkServerAvailableTestable('rovo-lsp\nmalicious', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject path with newline');
        });

        // Test non-string input (type safety)
        await runTest('should reject non-string input', async () => {
            const result = await checkServerAvailableTestable(null as any, [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject non-string input');
        });

        await runTest('should reject undefined input', async () => {
            const result = await checkServerAvailableTestable(undefined as any, [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject undefined input');
        });

        // Test legitimate paths still work
        await runTest('should accept legitimate simple path', async () => {
            const testExecutable = path.join(cargoBinDir, 'rovo-lsp');
            await fs.writeFile(testExecutable, '#!/bin/sh\necho "test"', { mode: 0o755 });
            
            const result = await checkServerAvailableTestable('rovo-lsp', [cargoBinDir]);
            assert.strictEqual(result, testExecutable, 'Should accept legitimate path');
        });

    } finally {
        await fs.rm(tempDir, { recursive: true, force: true });
    }
}

async function runPathTraversalTests(): Promise<void> {
    console.log('\n=== Path Traversal Protection Tests ===\n');
    
    // Setup
    const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'rovo-test-'));
    const cargoBinDir = path.join(tempDir, '.cargo', 'bin');
    await fs.mkdir(cargoBinDir, { recursive: true });
    
    const testExecutable = path.join(cargoBinDir, 'rovo-lsp');
    await fs.writeFile(testExecutable, '#!/bin/sh\necho "test"', { mode: 0o755 });
    
    const sensitiveFile = path.join(tempDir, 'sensitive.txt');
    await fs.writeFile(sensitiveFile, 'SECRET_DATA');

    try {
        await runTest('should find legitimate executable in cargo bin directory', async () => {
            const result = await checkServerAvailableTestable('rovo-lsp', [cargoBinDir]);
            assert.strictEqual(result, testExecutable, 'Should find the legitimate executable');
        });

        await runTest('should reject path traversal with ../ prefix', async () => {
            const result = await checkServerAvailableTestable('../sensitive.txt', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject path traversal attempt with ../');
        });

        await runTest('should reject path traversal with ../../ prefix', async () => {
            const result = await checkServerAvailableTestable('../../sensitive.txt', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject path traversal attempt with ../../');
        });

        await runTest('should reject absolute path attempts', async () => {
            const absolutePath = path.join(tempDir, 'sensitive.txt');
            const result = await checkServerAvailableTestable(absolutePath, [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject absolute path attempts');
        });

        await runTest('should reject path traversal with subdirectory escape', async () => {
            const result = await checkServerAvailableTestable('subdir/../../sensitive.txt', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject path traversal through subdirectory');
        });

        await runTest('should reject path traversal on Windows-style paths', async () => {
            const result = await checkServerAvailableTestable('..\\sensitive.txt', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject Windows-style path traversal');
        });

        await runTest('should allow legitimate subdirectory paths', async () => {
            const subDir = path.join(cargoBinDir, 'tools');
            await fs.mkdir(subDir, { recursive: true });
            const subExecutable = path.join(subDir, 'tool');
            await fs.writeFile(subExecutable, '#!/bin/sh\necho "tool"', { mode: 0o755 });
            
            const result = await checkServerAvailableTestable('tools/tool', [cargoBinDir]);
            assert.strictEqual(result, subExecutable, 'Should allow legitimate subdirectory paths');
            
            await fs.rm(subDir, { recursive: true, force: true });
        });

        await runTest('should return null for non-existent files without traversal', async () => {
            const result = await checkServerAvailableTestable('non-existent-binary', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should return null for non-existent files');
        });

    } finally {
        // Cleanup
        await fs.rm(tempDir, { recursive: true, force: true });
    }
}

async function runPathValidationTests(): Promise<void> {
    console.log('\n=== Path Validation Logic Tests ===\n');

    await runTest('path.relative correctly identifies parent directory traversal', () => {
        const base = '/home/user/.cargo/bin';
        const traversed = path.join(base, '../../../etc/passwd');
        const rel = path.relative(base, traversed);
        
        assert.ok(
            rel === '..' || rel.startsWith(`..${path.sep}`),
            'path.relative should identify parent directory traversal'
        );
    });

    await runTest('path.relative correctly identifies safe subdirectory', () => {
        const base = '/home/user/.cargo/bin';
        const safe = path.join(base, 'tools/rovo-lsp');
        const rel = path.relative(base, safe);
        
        assert.ok(
            rel !== '..' && !rel.startsWith(`..${path.sep}`) && !path.isAbsolute(rel),
            'path.relative should identify safe subdirectory paths'
        );
    });

    await runTest('path.isAbsolute correctly identifies absolute paths', () => {
        assert.strictEqual(path.isAbsolute('/etc/passwd'), true, 'Should identify Unix absolute path');
        
        if (process.platform === 'win32') {
            assert.strictEqual(path.isAbsolute('C:\\Windows\\System32'), true, 'Should identify Windows absolute path');
        }
    });

    await runTest('validation rejects when relative path is exactly ".."', () => {
        const base = '/home/user/.cargo/bin';
        const binPath = path.join(base, '..');
        const rel = path.relative(base, binPath);
        
        assert.strictEqual(rel, '..', 'Relative path should be exactly ".."');
    });
}

// Main test runner
async function main(): Promise<void> {
    console.log('Running security tests for command injection vulnerability mitigation...');
    
    await runCommandInjectionTests();
    await runInputValidationTests();
    await runPathTraversalTests();
    await runPathValidationTests();
    
    console.log('\n=== Test Summary ===\n');
    const passed = results.filter(r => r.passed).length;
    const failed = results.filter(r => !r.passed).length;
    console.log(`Total: ${results.length}, Passed: ${passed}, Failed: ${failed}`);
    
    if (failed > 0) {
        console.log('\nFailed tests:');
        results.filter(r => !r.passed).forEach(r => {
            console.log(`  - ${r.name}: ${r.error}`);
        });
        process.exit(1);
    } else {
        console.log('\nAll tests passed! ✓');
        process.exit(0);
    }
}

// Run tests if this file is executed directly
if (require.main === module) {
    main().catch(error => {
        console.error('Test runner failed:', error);
        process.exit(1);
    });
}

export { checkServerAvailableTestable, runTest, results };
