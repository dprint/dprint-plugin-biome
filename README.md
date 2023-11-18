# dprint-plugin-biome

[![](https://img.shields.io/crates/v/dprint-plugin-biome.svg)](https://crates.io/crates/dprint-plugin-biome) [![CI](https://github.com/dprint/dprint-plugin-biome/workflows/CI/badge.svg)](https://github.com/dprint/dprint-plugin-biome/actions?query=workflow%3ACI)

Adapter for [Biome](https://github.com/biomejs/biome) for use as a formatting plugin in [dprint](https://github.com/dprint/dprint).

## Install

[Install](https://dprint.dev/install/) and [setup](https://dprint.dev/setup/) dprint.

Then in your project's directory with a dprint.json file, run:

```shellsession
dprint config add biome
```

### Configuration

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
