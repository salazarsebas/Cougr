import test from 'node:test';
import assert from 'node:assert';
import { createServer } from '../src/index.js';

test('server creates successfully', async (t) => {
  const app = createServer();
  assert.ok(app);
});
