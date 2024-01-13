import { generateChangeLog } from "automation/changelog.ts";

const version = Deno.args[0];
const changelog = await generateChangeLog({
  versionTo: version,
});
const text = `## Changes

${changelog}

## Install

[Install](https://dprint.dev/install/) and [setup](https://dprint.dev/setup/) dprint.

Then in your project's directory with a dprint.json file, run:

\`\`\`shellsession
dprint config add biome
\`\`\`

## JS Formatting API

  * [JS Formatter](https://github.com/dprint/js-formatter) - Browser/Deno and Node
  * [npm package](https://www.npmjs.com/package/@dprint/biome)
`;

console.log(text);
