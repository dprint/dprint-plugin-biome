# dprint-plugin-biome

[![](https://img.shields.io/crates/v/dprint-plugin-biome.svg)](https://crates.io/crates/dprint-plugin-biome) [![CI](https://github.com/dprint/dprint-plugin-biome/workflows/CI/badge.svg)](https://github.com/dprint/dprint-plugin-biome/actions?query=workflow%3ACI)

Adapter for [Biome](https://github.com/biomejs/biome) for use as a formatting plugin in [dprint](https://github.com/dprint/dprint).

## Install

[Install](https://dprint.dev/install/) and [setup](https://dprint.dev/setup/) dprint.

Then in your project's directory with a dprint.json file, run:

```shellsession
dprint config add biome
```

## Configuration

To add configuration, specify a `"biome"` key in your dprint.json:

```jsonc
{
  "biome": {
    "indentStyle": "space",
    "lineWidth": 100,
    "indentWidth": 4
  },
  "plugins": [
    // ...etc...
  ]
}
```

For an overview of the config, see https://dprint.dev/plugins/biome/config/

## JS Formatting API

- [JS Formatter](https://github.com/dprint/js-formatter) - Browser/Deno and Node
- [npm package](https://www.npmjs.com/package/@dprint/biome)

## Versioning

This repo automatically upgrades to the latest version of Biome once a day. You can check which version of Biome is being used by looking at the `tag` property in the `biome_js_formatter` entry in the Cargo.toml file in this repo:

https://github.com/dprint/dprint-plugin-biome/blob/main/Cargo.toml

At the moment, the version of this plugin does not reflect the version of Biome. This is just in case there are any small bug fixes that need to be made as this plugin is quite new. After a while I'll try to match the versions.
