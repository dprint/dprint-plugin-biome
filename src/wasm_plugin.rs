use super::configuration::Configuration;
use super::configuration::resolve_config;

use dprint_core::configuration::ConfigKeyMap;
use dprint_core::configuration::GlobalConfiguration;
use dprint_core::generate_plugin_code;
use dprint_core::plugins::CheckConfigUpdatesMessage;
use dprint_core::plugins::ConfigChange;
use dprint_core::plugins::FileMatchingInfo;
use dprint_core::plugins::FormatResult;
use dprint_core::plugins::PluginInfo;
use dprint_core::plugins::PluginResolveConfigurationResult;
use dprint_core::plugins::SyncFormatRequest;
use dprint_core::plugins::SyncHostFormatRequest;
use dprint_core::plugins::SyncPluginHandler;

struct BiomePluginHandler;

impl SyncPluginHandler<Configuration> for BiomePluginHandler {
  fn resolve_config(
    &mut self,
    config: ConfigKeyMap,
    global_config: &GlobalConfiguration,
  ) -> PluginResolveConfigurationResult<Configuration> {
    let result = resolve_config(config, global_config);
    let mut file_extensions = vec![
      "ts".to_string(),
      "tsx".to_string(),
      "cts".to_string(),
      "mts".to_string(),
      "js".to_string(),
      "jsx".to_string(),
      "cjs".to_string(),
      "mjs".to_string(),
      "json".to_string(),
      "jsonc".to_string(),
    ];
    if result.config.css_enabled == Some(true) {
      file_extensions.push("css".to_string());
    }
    if result.config.graphql_enabled == Some(true) {
      file_extensions.push("graphql".to_string());
    }
    PluginResolveConfigurationResult {
      config: result.config,
      diagnostics: result.diagnostics,
      file_matching: FileMatchingInfo {
        file_extensions,
        file_names: vec![],
      },
    }
  }

  fn check_config_updates(&self, _message: CheckConfigUpdatesMessage) -> anyhow::Result<Vec<ConfigChange>> {
    Ok(Vec::new())
  }

  fn plugin_info(&mut self) -> PluginInfo {
    let version = env!("CARGO_PKG_VERSION").to_string();
    PluginInfo {
      name: env!("CARGO_PKG_NAME").to_string(),
      version: version.clone(),
      config_key: "biome".to_string(),
      help_url: "https://dprint.dev/plugins/biome".to_string(),
      config_schema_url: format!(
        "https://plugins.dprint.dev/dprint/dprint-plugin-biome/{}/schema.json",
        version
      ),
      update_url: Some("https://plugins.dprint.dev/dprint/dprint-plugin-biome/latest.json".to_string()),
    }
  }

  fn license_text(&mut self) -> String {
    std::str::from_utf8(include_bytes!("../LICENSE")).unwrap().into()
  }

  fn format(
    &mut self,
    request: SyncFormatRequest<Configuration>,
    _format_with_host: impl FnMut(SyncHostFormatRequest) -> FormatResult,
  ) -> FormatResult {
    if request.range.is_some() {
      return Ok(None); // not implemented
    }

    let text = String::from_utf8_lossy(&request.file_bytes);
    let maybe_text = super::format_text(request.file_path, &text, request.config)?;
    Ok(maybe_text.map(|t| t.into_bytes()))
  }
}

generate_plugin_code!(BiomePluginHandler, BiomePluginHandler);
