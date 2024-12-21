import * as path from 'path';
import Mocha from 'mocha';
import glob from 'glob';
import './mock/moduleResolver';

export function run(): Promise<void> {
    // Create the mocha test
    const mocha = new Mocha({
        ui: 'tdd',
        color: true,
        timeout: 10000,  // Increase timeout for slower machines
        parallel: true,  // Enable parallel test execution
        retries: 1      // Allow one retry for flaky tests
    });

    const testsRoot = path.resolve(__dirname, '.');

    return new Promise<void>((resolve, reject) => {
        glob('**/**.test.js', { cwd: testsRoot }, (err: Error | null, files: string[]) => {
            if (err) {
                return reject(err);
            }

            // Add files to the test suite
            files.forEach((f: string) => mocha.addFile(path.resolve(testsRoot, f)));

            try {
                // Run the mocha test
                mocha.run((failures: number) => {
                    if (failures > 0) {
                        reject(new Error(`${failures} tests failed.`));
                    } else {
                        resolve();
                    }
                });
            } catch (err) {
                console.error(err);
                reject(err);
            }
        });
    });
} 