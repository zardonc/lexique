import { strict as assert } from 'node:assert';
import { readFile } from 'node:fs/promises';

const artifact = new URL('../target/wasm32-unknown-unknown/release/lexique_core.wasm', import.meta.url);
const { instance } = await WebAssembly.instantiate(await readFile(artifact));
assert.equal(instance.exports.schema_version(), 10, 'WASM must expose the current shared schema version');
console.log('Compiled WASM loaded and executed successfully. This checks the scaffold ABI, not FSRS.');
