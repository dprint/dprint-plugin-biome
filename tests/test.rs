extern crate dprint_development;
extern crate dprint_plugin_biome;

use std::path::PathBuf;
use std::sync::Arc;

use dprint_core::configuration::*;
use dprint_development::*;
use dprint_plugin_biome::configuration::Configuration;
use dprint_plugin_biome::configuration::resolve_config;
use dprint_plugin_biome::*;
use pretty_assertions::assert_eq;

#[test]
fn test_specs() {
  let global_config = GlobalConfiguration::default();

  run_specs(
    &PathBuf::from("./tests/specs"),
    &ParseSpecOptions {
      default_file_name: "file.ts",
    },
    &RunSpecsOptions {
      fix_failures: false,
      format_twice: true,
    },
    {
      let global_config = global_config.clone();
      Arc::new(move |file_path, file_text, spec_config| {
        let spec_config: ConfigKeyMap = serde_json::from_value(spec_config.clone().into()).unwrap();
        let config_result = resolve_config(spec_config, &global_config);
        ensure_no_diagnostics(&config_result.diagnostics);

        format_text(file_path, file_text, &config_result.config).map_err(anyhow::Error::from)
      })
    },
    Arc::new(move |_file_path, _file_text, _spec_config| panic!("Plugin does not support dprint-core tracing.")),
  )
}

#[test]
fn should_fail_on_parse_error_js() {
  let config = Configuration::default();
  let err = format_text(&PathBuf::from("./file.ts"), "const t string = 5;", &config).unwrap_err();
  assert_eq!(
    err.to_string(),
    r#"parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Const declarations must have an initialized value.
  
  i This variable needs to be initialized.
  


parse ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  × Expected a semicolon or an implicit semicolon after a statement, but found none
  
  i An explicit or implicit semicolon is expected here...
  
  i ...Which is required to end this statement
  

"#
  );
}

#[test]
fn should_fail_on_parse_error_json() {
  let config = Configuration::default();
  let err = format_text(&PathBuf::from("./file.json"), "{", &config).unwrap_err();
  let err_str = err.to_string();
  assert!(
    err_str.contains("expected `}` but instead the file ends"),
    "Unexpected error: {}",
    err_str
  );
}

#[test]
fn should_not_format_disabled_languages() {
  let mut config_map = ConfigKeyMap::new();
  for key in ["typescript.enabled", "jsonc.enabled"] {
    config_map.insert(key.to_string(), ConfigKeyValue::from_bool(false));
  }
  let result = resolve_config(config_map, &GlobalConfiguration::default());
  assert!(result.diagnostics.is_empty());
  for (file_name, text, expected) in [
    ("file.ts", "a", None),
    ("file.tsx", "a", None),
    ("file.mts", "a", None),
    ("file.jsonc", "[1,2]", None),
    ("file.js", "a", Some("a;\n")),
    ("file.jsx", "a", Some("a;\n")),
    ("file.json", "[1,2]", Some("[1, 2]\n")),
  ] {
    let output = format_text(&PathBuf::from(file_name), text, &result.config).unwrap();
    assert_eq!(output.as_deref(), expected, "{file_name}");
  }

  let mut config_map = ConfigKeyMap::new();
  for key in ["javascript.enabled", "json.enabled"] {
    config_map.insert(key.to_string(), ConfigKeyValue::from_bool(false));
  }
  let result = resolve_config(config_map, &GlobalConfiguration::default());
  assert!(result.diagnostics.is_empty());
  for (file_name, text, expected) in [
    ("file.js", "a", None),
    ("file.cjs", "a", None),
    ("file.json", "[1,2]", None),
    ("file.ts", "a", Some("a;\n")),
    ("file.jsonc", "[1,2]", Some("[1, 2]\n")),
  ] {
    let output = format_text(&PathBuf::from(file_name), text, &result.config).unwrap();
    assert_eq!(output.as_deref(), expected, "{file_name}");
  }
}

#[test]
fn language_enabled_defaults() {
  use dprint_plugin_biome::configuration::Language;

  let config = Configuration::default();
  let enabled = Language::ALL
    .into_iter()
    .filter(|language| config.is_language_enabled(*language))
    .collect::<Vec<_>>();
  assert_eq!(
    enabled,
    vec![
      Language::TypeScript,
      Language::JavaScript,
      Language::Json,
      Language::Jsonc
    ]
  );
  let config = Configuration {
    css_enabled: Some(true),
    graphql_enabled: Some(true),
    ..Default::default()
  };
  assert!(
    Language::ALL
      .into_iter()
      .all(|language| config.is_language_enabled(language))
  );
  for language in Language::ALL {
    for ext in language.extensions() {
      assert_eq!(Language::from_extension(ext), Some(language), "{ext}");
    }
  }
}

#[test]
fn css_modules_config_resolves() {
  let mut config_map = ConfigKeyMap::new();
  config_map.insert("css.cssModules".to_string(), ConfigKeyValue::from_bool(true));
  let global_config = GlobalConfiguration::default();
  let result = resolve_config(config_map, &global_config);
  assert!(result.diagnostics.is_empty());
  assert_eq!(result.config.css_css_modules, Some(true));
}
