#!/usr/bin/env node
// Prova CLI entry point.
// Pure ESM, zero deps — uses only Node built-ins.

import { run } from '../src/index.mjs';

run(process.argv.slice(2)).catch(err => {
  console.error('\x1b[31merror:\x1b[0m', err.message || err);
  if (process.env.PROVA_DEBUG) console.error(err.stack);
  process.exit(1);
});
