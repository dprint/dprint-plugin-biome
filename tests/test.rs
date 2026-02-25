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

        format_text(file_path, &file_text, &config_result.config)
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
  assert!(err_str.contains("expected `}` but instead the file ends"), "Unexpected error: {}", err_str);
}

#[test]
fn grit_metavariables_js() {
  let mut config = Configuration::default();
  config.grit_metavariables = Some(true);
  let result = format_text(&PathBuf::from("./file.ts"), "const x = µvar_name;\n", &config);
  assert!(result.is_ok());
}

#[test]
fn grit_metavariables_config_resolves() {
  let mut config_map = ConfigKeyMap::new();
  config_map.insert("gritMetavariables".to_string(), ConfigKeyValue::from_bool(true));
  let global_config = GlobalConfiguration::default();
  let result = resolve_config(config_map, &global_config);
  assert!(result.diagnostics.is_empty());
  assert_eq!(result.config.grit_metavariables, Some(true));
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
