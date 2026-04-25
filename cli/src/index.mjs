// Prova CLI dispatcher.
//
// Commands:
//   prova auth              — sign in (email magic-token), saves to ~/.prova/config.json
//   prova put <file>        — upload a file, prints piece-cid + retrieval URL
//   prova get <cid>         — download a file by cid (-> stdout or -o file)
//   prova ls                — list your files
//   prova whoami            — show signed-in identity + quota usage
//   prova logout            — clear local creds
//   prova help              — show usage

import { putCmd } from './cmd/put.mjs';
import { getCmd } from './cmd/get.mjs';
import { authCmd } from './cmd/auth.mjs';
import { lsCmd } from './cmd/ls.mjs';
import { whoamiCmd } from './cmd/whoami.mjs';
import { logoutCmd } from './cmd/logout.mjs';

const VERSION = '0.1.0';

const HELP = `
\x1b[1mProva CLI\x1b[0m \x1b[2mv${VERSION}\x1b[0m

  prova \x1b[36mauth\x1b[0m                 Sign in with your email
  prova \x1b[36mput\x1b[0m \x1b[2m<file>\x1b[0m          Upload a file, get a piece-cid
  prova \x1b[36mget\x1b[0m \x1b[2m<cid> [-o out]\x1b[0m  Retrieve a file by cid
  prova \x1b[36mls\x1b[0m                   List your files
  prova \x1b[36mwhoami\x1b[0m               Show signed-in identity + usage
  prova \x1b[36mlogout\x1b[0m               Forget your credentials
  prova \x1b[36mhelp\x1b[0m                 This help

  prova \x1b[36m--version\x1b[0m

\x1b[2mDocs: https://prova-network.pages.dev/\x1b[0m
`;

export async function run(argv) {
  const [cmd, ...rest] = argv;

  if (!cmd || cmd === 'help' || cmd === '--help' || cmd === '-h') {
    process.stdout.write(HELP);
    return;
  }
  if (cmd === '--version' || cmd === '-v') {
    console.log(VERSION);
    return;
  }

  switch (cmd) {
    case 'auth':    return authCmd(rest);
    case 'put':     return putCmd(rest);
    case 'get':     return getCmd(rest);
    case 'ls':      return lsCmd(rest);
    case 'whoami':  return whoamiCmd(rest);
    case 'logout':  return logoutCmd(rest);
    default:
      console.error(`\x1b[31munknown command:\x1b[0m ${cmd}`);
      process.stdout.write(HELP);
      process.exit(2);
  }
}
