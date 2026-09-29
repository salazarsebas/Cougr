#!/usr/bin/env node
import { readFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

import { replayFromFixture, type MatchFixtureFile } from './replay.ts';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  let fixturePath = resolve(__dirname, '../fixtures/turn_based_match.json');
  let stepByStep = false;

  for (let i = 0; i < args.length; i += 1) {
    if (args[i] === '--fixture' && args[i + 1]) {
      fixturePath = resolve(process.cwd(), args[i + 1] as string);
      i += 1;
    } else if (args[i] === '--step-by-step') {
      stepByStep = true;
    } else if (args[i] === '--help' || args[i] === '-h') {
      console.log('Usage: cougr-spectator [options]');
      console.log('');
      console.log('Options:');
      console.log('  --fixture <path>   Path to match events fixture JSON (default: fixtures/turn_based_match.json)');
      console.log('  --step-by-step     Step through each timeline frame');
      console.log('  --help, -h         Show this help message');
      process.exit(0);
    }
  }

  console.log(`Loading match fixture from: ${fixturePath}`);
  const raw = readFileSync(fixturePath, 'utf8');
  const fixture = JSON.parse(raw) as MatchFixtureFile;

  const timeline = await replayFromFixture(fixture);

  if (stepByStep) {
    for (let i = 0; i < timeline.length; i += 1) {
      timeline.goToStep(i);
      console.log(timeline.formatStateLog());
      console.log('');
    }
  } else {
    console.log(timeline.formatFullLog());
  }
}

main().catch((err) => {
  console.error('Replay failed:', err);
  process.exit(1);
});
