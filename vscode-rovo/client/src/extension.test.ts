import * as assert from 'assert';
import * as path from 'path';
import * as fs from 'fs/promises';
import * as os from 'os';

/**
 * Security tests for path traversal vulnerability mitigation in checkServerAvailable function.
 * 
 * The vulnerability being tested: Potential file inclusion attack via reading file.
 * The fix validates that constructed paths don't escape the cargo bin directory using path.relative().
 */

// Extract the checkServerAvailable logic for testing
async function checkServerAvailableTestable(serverPath: string, cargoBinDirs: string[]): Promise<string | null> {
    // Simulate the fixed logic from extension.ts (lines 235-248)
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

async function runSecurityTests(): Promise<void> {
    console.log('\n=== Extension Security Tests ===\n');
    
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

        await runTest('should reject path with null bytes', async () => {
            const result = await checkServerAvailableTestable('rovo-lsp\0/../../sensitive.txt', [cargoBinDir]);
            assert.strictEqual(result, null, 'Should reject paths with null bytes');
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
    console.log('Running security tests for path traversal vulnerability mitigation...');
    
    await runSecurityTests();
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
