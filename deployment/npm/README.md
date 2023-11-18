# @dprint/biome

npm distribution of [dprint-plugin-biome](https://github.com/dprint/dprint-plugin-biome) which is an adapter plugin for [Biome](https://github.com/biomejs/biome).

Use this with [@dprint/formatter](https://github.com/dprint/js-formatter) or just use @dprint/formatter and download the [dprint-plugin-biome Wasm file](https://github.com/dprint/dprint-plugin-biome/releases).

## Example

```ts
import { getBuffer } from "@dprint/biome";
import { createFromBuffer } from "@dprint/formatter";

const formatter = createFromBuffer(getBuffer());

console.log(
  formatter.formatText("test.js", "console.log(  5  )"),
);
```
