import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { cpSync, mkdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const root = fileURLToPath(new URL('../', import.meta.url));
const output = join(root, 'apps/desktop/public/ocr');
const modules = join(root, 'apps/desktop/node_modules');
for (const name of ['tesseract.js', 'tesseract.js-core'])
  assert.equal(JSON.parse(readFileSync(join(modules, name, 'package.json'), 'utf8')).version, '7.0.0', `Wrong ${name} version`);
const model = join(root, 'resources/tesseract-fra/fra.traineddata');
assert.equal(createHash('sha256').update(readFileSync(model)).digest('hex'),
  'ced037562e8c80c13122dece28dd477d399af80911a28791a66a63ac1e3445ca', 'French OCR model is invalid');
mkdirSync(output, { recursive: true });
cpSync(model, join(output, 'fra.traineddata'));
cpSync(join(modules, 'tesseract.js/dist/worker.min.js'), join(output, 'worker.min.js'));
cpSync(join(modules, 'tesseract.js/dist/worker.min.js.LICENSE.txt'), join(output, 'worker.min.js.LICENSE.txt'));
for (const variant of ['', '-lstm', '-simd', '-simd-lstm', '-relaxedsimd', '-relaxedsimd-lstm']) {
  const name = `tesseract-core${variant}.wasm.js`;
  cpSync(join(modules, 'tesseract.js-core', name), join(output, name));
}
cpSync(join(root, 'resources/tesseract-fra/licenses'), join(output, 'notices'), { recursive: true });
cpSync(join(root, 'resources/tesseract-fra/README.md'), join(output, 'README.md'));
