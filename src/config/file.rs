//! The `trippy.toml` config file, with the same sections and keys as trippy.
//!
//! The file is looked up where trippy looks: the current directory, the home
//! directory, the config directory and `config directory/trippy`, under the
//! names `trippy.toml` and `.trippy.toml`; the first one found is used.

use crate::config::{
    AddressFamilyConfig, AddressMode, AsMode, DnsResolveMethodConfig, GeoIpMode, IcmpExtensionMode,
    MultipathStrategyConfig, ProtocolConfig,
};
use anyhow::Context;
use etcetera::BaseStrategy;
use serde::Deserialize;
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

const DEFAULT_CONFIG_FILE: &str = "trippy.toml";
const DEFAULT_HIDDEN_CONFIG_FILE: &str = ".trippy.toml";

/// Reads the config file from the default locations, if there is one.
pub fn read_default_config_file() -> anyhow::Result<Option<ConfigFile>> {
    if let Some(file) = read_files("")? {
        return Ok(Some(file));
    }
    let base = etcetera::choose_base_strategy()?;
    if let Some(file) = read_files(base.home_dir())? {
        Ok(Some(file))
    } else if let Some(file) = read_files(base.config_dir())? {
        Ok(Some(file))
    } else if let Some(file) = read_files(base.config_dir().join("trippy"))? {
        Ok(Some(file))
    } else {
        Ok(None)
    }
}

/// Reads the config file at a path.
pub fn read_config_file<P: AsRef<Path>>(path: P) -> anyhow::Result<ConfigFile> {
    let text = std::fs::read_to_string(path.as_ref())
        .with_context(|| format!("config file not found: {}", path.as_ref().display()))?;
    parse(text.trim_start_matches('\u{feff}'))
        .with_context(|| format!("invalid config file: {}", path.as_ref().display()))
}

/// Parses the contents of a config file.
pub fn parse(text: &str) -> anyhow::Result<ConfigFile> {
    Ok(toml::from_str(text)?)
}

fn read_files<P: AsRef<Path>>(dir: P) -> anyhow::Result<Option<ConfigFile>> {
    if let Some(file) = read_file(dir.as_ref(), DEFAULT_CONFIG_FILE)? {
        Ok(Some(file))
    } else if let Some(file) = read_file(dir.as_ref(), DEFAULT_HIDDEN_CONFIG_FILE)? {
        Ok(Some(file))
    } else {
        Ok(None)
    }
}

fn read_file<P: AsRef<Path>>(dir: P, file: &str) -> anyhow::Result<Option<ConfigFile>> {
    let path = dir.as_ref().join(file);
    if path.exists() {
        Ok(Some(read_config_file(path)?))
    } else {
        Ok(None)
    }
}

/// The whole file; every section is optional.
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConfigFile {
    pub trippy: Option<ConfigTrippy>,
    pub strategy: Option<ConfigStrategy>,
    pub theme_colors: Option<ConfigThemeColors>,
    pub bindings: Option<ConfigBindings>,
    pub tui: Option<ConfigTui>,
    pub dns: Option<ConfigDns>,
    pub report: Option<ConfigReport>,
}

/// The `[trippy]` section. Only `unprivileged` applies to the window; the
/// output mode and logging keys belong to `trip` and are accepted and ignored.
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConfigTrippy {
    pub mode: Option<String>,
    pub unprivileged: Option<bool>,
    pub log_format: Option<String>,
    pub log_filter: Option<String>,
    pub log_span_events: Option<String>,
}

/// The `[strategy]` section.
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConfigStrategy {
    pub protocol: Option<ProtocolConfig>,
    pub addr_family: Option<AddressFamilyConfig>,
    pub target_port: Option<u16>,
    pub source_port: Option<u16>,
    #[serde(default, deserialize_with = "addr_deser")]
    pub source_address: Option<IpAddr>,
    pub interface: Option<String>,
    #[serde(default, deserialize_with = "humantime_deser")]
    pub min_round_duration: Option<Duration>,
    #[serde(default, deserialize_with = "humantime_deser")]
    pub max_round_duration: Option<Duration>,
    pub initial_sequence: Option<u16>,
    pub multipath_strategy: Option<MultipathStrategyConfig>,
    #[serde(default, deserialize_with = "humantime_deser")]
    pub grace_duration: Option<Duration>,
    pub max_inflight: Option<u8>,
    pub first_ttl: Option<u8>,
    pub max_ttl: Option<u8>,
    pub packet_size: Option<u16>,
    pub payload_pattern: Option<u8>,
    pub tos: Option<u8>,
    pub icmp_extensions: Option<bool>,
    #[serde(default, deserialize_with = "humantime_deser")]
    pub read_timeout: Option<Duration>,
    pub max_samples: Option<usize>,
    pub max_flows: Option<usize>,
}

/// The `[dns]` section.
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConfigDns {
    pub dns_resolve_method: Option<DnsResolveMethodConfig>,
    pub dns_resolve_all: Option<bool>,
    pub dns_lookup_as_info: Option<bool>,
    #[serde(default, deserialize_with = "humantime_deser")]
    pub dns_timeout: Option<Duration>,
    #[serde(default, deserialize_with = "humantime_deser")]
    pub dns_ttl: Option<Duration>,
}

/// The `[report]` section; it belongs to `trip` and is accepted and ignored.
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConfigReport {
    pub report_cycles: Option<usize>,
}

/// The `[tui]` section.
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConfigTui {
    pub tui_preserve_screen: Option<bool>,
    #[serde(default, deserialize_with = "humantime_deser")]
    pub tui_refresh_rate: Option<Duration>,
    pub tui_privacy_max_ttl: Option<u8>,
    pub tui_address_mode: Option<AddressMode>,
    pub tui_as_mode: Option<AsMode>,
    pub tui_icmp_extension_mode: Option<IcmpExtensionMode>,
    pub tui_geoip_mode: Option<GeoIpMode>,
    pub tui_max_addrs: Option<u8>,
    pub geoip_mmdb_file: Option<String>,
    pub tui_custom_columns: Option<String>,
    pub tui_locale: Option<String>,
    pub tui_timezone: Option<String>,
    #[serde(rename = "tui-max-samples")]
    pub deprecated_tui_max_samples: Option<usize>,
    #[serde(rename = "tui-max-flows")]
    pub deprecated_tui_max_flows: Option<usize>,
}

/// The `[theme-colors]` section: trippy's theme items, as colour names.
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConfigThemeColors {
    pub bg_color: Option<String>,
    pub border_color: Option<String>,
    pub text_color: Option<String>,
    pub tab_text_color: Option<String>,
    pub hops_table_header_bg_color: Option<String>,
    pub hops_table_header_text_color: Option<String>,
    pub hops_table_row_active_text_color: Option<String>,
    pub hops_table_row_inactive_text_color: Option<String>,
    pub hops_table_row_active_selected_bg_color: Option<String>,
    pub hops_table_row_active_selected_text_color: Option<String>,
    pub hops_table_row_inactive_selected_bg_color: Option<String>,
    pub hops_table_row_inactive_selected_text_color: Option<String>,
    pub hops_chart_selected_color: Option<String>,
    pub hops_chart_unselected_color: Option<String>,
    pub hops_chart_axis_color: Option<String>,
    pub frequency_chart_bar_color: Option<String>,
    pub frequency_chart_text_color: Option<String>,
    pub flows_chart_bar_selected_color: Option<String>,
    pub flows_chart_bar_unselected_color: Option<String>,
    pub flows_chart_text_current_color: Option<String>,
    pub flows_chart_text_non_current_color: Option<String>,
    pub samples_chart_color: Option<String>,
    pub samples_chart_lost_color: Option<String>,
    pub help_dialog_bg_color: Option<String>,
    pub help_dialog_text_color: Option<String>,
    pub settings_dialog_bg_color: Option<String>,
    pub settings_tab_text_color: Option<String>,
    pub settings_table_header_text_color: Option<String>,
    pub settings_table_header_bg_color: Option<String>,
    pub settings_table_row_text_color: Option<String>,
    pub map_world_color: Option<String>,
    pub map_radius_color: Option<String>,
    pub map_selected_color: Option<String>,
    pub map_info_panel_border_color: Option<String>,
    pub map_info_panel_bg_color: Option<String>,
    pub map_info_panel_text_color: Option<String>,
    pub info_bar_bg_color: Option<String>,
    pub info_bar_text_color: Option<String>,
}

impl ConfigThemeColors {
    /// The items that are set, as `(item, colour)` pairs in trippy's names.
    pub fn items(&self) -> Vec<(&'static str, String)> {
        let pairs = [
            ("bg-color", &self.bg_color),
            ("border-color", &self.border_color),
            ("text-color", &self.text_color),
            ("tab-text-color", &self.tab_text_color),
            (
                "hops-table-header-bg-color",
                &self.hops_table_header_bg_color,
            ),
            (
                "hops-table-header-text-color",
                &self.hops_table_header_text_color,
            ),
            (
                "hops-table-row-active-text-color",
                &self.hops_table_row_active_text_color,
            ),
            (
                "hops-table-row-inactive-text-color",
                &self.hops_table_row_inactive_text_color,
            ),
            (
                "hops-table-row-active-selected-bg-color",
                &self.hops_table_row_active_selected_bg_color,
            ),
            (
                "hops-table-row-active-selected-text-color",
                &self.hops_table_row_active_selected_text_color,
            ),
            (
                "hops-table-row-inactive-selected-bg-color",
                &self.hops_table_row_inactive_selected_bg_color,
            ),
            (
                "hops-table-row-inactive-selected-text-color",
                &self.hops_table_row_inactive_selected_text_color,
            ),
            ("hops-chart-selected-color", &self.hops_chart_selected_color),
            (
                "hops-chart-unselected-color",
                &self.hops_chart_unselected_color,
            ),
            ("hops-chart-axis-color", &self.hops_chart_axis_color),
            ("frequency-chart-bar-color", &self.frequency_chart_bar_color),
            (
                "frequency-chart-text-color",
                &self.frequency_chart_text_color,
            ),
            (
                "flows-chart-bar-selected-color",
                &self.flows_chart_bar_selected_color,
            ),
            (
                "flows-chart-bar-unselected-color",
                &self.flows_chart_bar_unselected_color,
            ),
            (
                "flows-chart-text-current-color",
                &self.flows_chart_text_current_color,
            ),
            (
                "flows-chart-text-non-current-color",
                &self.flows_chart_text_non_current_color,
            ),
            ("samples-chart-color", &self.samples_chart_color),
            ("samples-chart-lost-color", &self.samples_chart_lost_color),
            ("help-dialog-bg-color", &self.help_dialog_bg_color),
            ("help-dialog-text-color", &self.help_dialog_text_color),
            ("settings-dialog-bg-color", &self.settings_dialog_bg_color),
            ("settings-tab-text-color", &self.settings_tab_text_color),
            (
                "settings-table-header-text-color",
                &self.settings_table_header_text_color,
            ),
            (
                "settings-table-header-bg-color",
                &self.settings_table_header_bg_color,
            ),
            (
                "settings-table-row-text-color",
                &self.settings_table_row_text_color,
            ),
            ("map-world-color", &self.map_world_color),
            ("map-radius-color", &self.map_radius_color),
            ("map-selected-color", &self.map_selected_color),
            (
                "map-info-panel-border-color",
                &self.map_info_panel_border_color,
            ),
            ("map-info-panel-bg-color", &self.map_info_panel_bg_color),
            ("map-info-panel-text-color", &self.map_info_panel_text_color),
            ("info-bar-bg-color", &self.info_bar_bg_color),
            ("info-bar-text-color", &self.info_bar_text_color),
        ];
        pairs
            .into_iter()
            .filter_map(|(item, value)| value.as_ref().map(|v| (item, v.clone())))
            .collect()
    }
}

/// The `[bindings]` section: trippy's commands, as key names.
#[derive(Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ConfigBindings {
    pub toggle_help: Option<String>,
    pub toggle_help_alt: Option<String>,
    pub toggle_settings: Option<String>,
    pub toggle_settings_tui: Option<String>,
    pub toggle_settings_trace: Option<String>,
    pub toggle_settings_dns: Option<String>,
    pub toggle_settings_geoip: Option<String>,
    pub toggle_settings_bindings: Option<String>,
    pub toggle_settings_theme: Option<String>,
    pub toggle_settings_columns: Option<String>,
    pub previous_hop: Option<String>,
    pub next_hop: Option<String>,
    pub previous_trace: Option<String>,
    pub next_trace: Option<String>,
    pub previous_hop_address: Option<String>,
    pub next_hop_address: Option<String>,
    pub address_mode_ip: Option<String>,
    pub address_mode_host: Option<String>,
    pub address_mode_both: Option<String>,
    pub toggle_freeze: Option<String>,
    pub toggle_chart: Option<String>,
    pub toggle_flows: Option<String>,
    #[serde(rename = "toggle-privacy")]
    pub deprecated_toggle_privacy: Option<String>,
    pub expand_privacy: Option<String>,
    pub contract_privacy: Option<String>,
    pub toggle_map: Option<String>,
    pub expand_hosts: Option<String>,
    pub contract_hosts: Option<String>,
    pub expand_hosts_max: Option<String>,
    pub contract_hosts_min: Option<String>,
    pub chart_zoom_in: Option<String>,
    pub chart_zoom_out: Option<String>,
    pub clear_trace_data: Option<String>,
    pub clear_dns_cache: Option<String>,
    pub clear_selection: Option<String>,
    pub toggle_as_info: Option<String>,
    pub toggle_hop_details: Option<String>,
    pub quit: Option<String>,
    pub quit_preserve_screen: Option<String>,
}

impl ConfigBindings {
    /// The commands that are bound, as `(command, key)` pairs in trippy's names.
    pub fn items(&self) -> Vec<(&'static str, String)> {
        let pairs = [
            ("toggle-help", &self.toggle_help),
            ("toggle-help-alt", &self.toggle_help_alt),
            ("toggle-settings", &self.toggle_settings),
            ("toggle-settings-tui", &self.toggle_settings_tui),
            ("toggle-settings-trace", &self.toggle_settings_trace),
            ("toggle-settings-dns", &self.toggle_settings_dns),
            ("toggle-settings-geoip", &self.toggle_settings_geoip),
            ("toggle-settings-bindings", &self.toggle_settings_bindings),
            ("toggle-settings-theme", &self.toggle_settings_theme),
            ("toggle-settings-columns", &self.toggle_settings_columns),
            ("previous-hop", &self.previous_hop),
            ("next-hop", &self.next_hop),
            ("previous-trace", &self.previous_trace),
            ("next-trace", &self.next_trace),
            ("previous-hop-address", &self.previous_hop_address),
            ("next-hop-address", &self.next_hop_address),
            ("address-mode-ip", &self.address_mode_ip),
            ("address-mode-host", &self.address_mode_host),
            ("address-mode-both", &self.address_mode_both),
            ("toggle-freeze", &self.toggle_freeze),
            ("toggle-chart", &self.toggle_chart),
            ("toggle-flows", &self.toggle_flows),
            ("expand-privacy", &self.expand_privacy),
            ("contract-privacy", &self.contract_privacy),
            ("toggle-map", &self.toggle_map),
            ("expand-hosts", &self.expand_hosts),
            ("contract-hosts", &self.contract_hosts),
            ("expand-hosts-max", &self.expand_hosts_max),
            ("contract-hosts-min", &self.contract_hosts_min),
            ("chart-zoom-in", &self.chart_zoom_in),
            ("chart-zoom-out", &self.chart_zoom_out),
            ("clear-trace-data", &self.clear_trace_data),
            ("clear-dns-cache", &self.clear_dns_cache),
            ("clear-selection", &self.clear_selection),
            ("toggle-as-info", &self.toggle_as_info),
            ("toggle-hop-details", &self.toggle_hop_details),
            ("quit", &self.quit),
            ("quit-preserve-screen", &self.quit_preserve_screen),
        ];
        pairs
            .into_iter()
            .filter_map(|(item, value)| value.as_ref().map(|v| (item, v.clone())))
            .collect()
    }
}

fn humantime_deser<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    humantime::parse_duration(&String::deserialize(deserializer)?)
        .map_err(serde::de::Error::custom)
        .map(Some)
}

fn addr_deser<'de, D>(deserializer: D) -> Result<Option<IpAddr>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    IpAddr::from_str(&String::deserialize(deserializer)?)
        .map_err(serde::de::Error::custom)
        .map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_section() {
        let file = parse(
            r#"
            [trippy]
            mode = "tui"
            unprivileged = true
            [strategy]
            protocol = "udp"
            min-round-duration = "2s"
            source-address = "10.0.0.1"
            [dns]
            dns-resolve-method = "cloudflare"
            [tui]
            tui-address-mode = "both"
            tui-timezone = "Europe/Berlin"
            [theme-colors]
            bg-color = "black"
            [bindings]
            toggle-chart = "x"
            [report]
            report-cycles = 3
            "#,
        )
        .unwrap();
        assert_eq!(file.trippy.unwrap().unprivileged, Some(true));
        let strategy = file.strategy.unwrap();
        assert_eq!(strategy.protocol, Some(ProtocolConfig::Udp));
        assert_eq!(strategy.min_round_duration, Some(Duration::from_secs(2)));
        assert_eq!(strategy.source_address, Some("10.0.0.1".parse().unwrap()));
        assert_eq!(
            file.dns.unwrap().dns_resolve_method,
            Some(DnsResolveMethodConfig::Cloudflare)
        );
        assert_eq!(file.tui.unwrap().tui_address_mode, Some(AddressMode::Both));
        assert_eq!(
            file.theme_colors.unwrap().items(),
            [("bg-color", String::from("black"))]
        );
        assert_eq!(
            file.bindings.unwrap().items(),
            [("toggle-chart", String::from("x"))]
        );
    }

    #[test]
    fn rejects_unknown_keys_and_bad_values() {
        assert!(parse("[tui]\nno-such-key = 1").is_err());
        assert!(parse("[strategy]\nmin-round-duration = \"soon\"").is_err());
        assert!(parse("[dns]\ndns-resolve-method = \"carrier-pigeon\"").is_err());
        assert!(parse("[nope]\nx = 1").is_err());
    }

    #[test]
    fn trippy_sample_config_is_all_defaults() {
        let sample = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../trippy/crates/trippy-tui/trippy-config-sample.toml"
        );
        let Ok(text) = std::fs::read_to_string(sample) else {
            eprintln!("skipped: {sample} not found");
            return;
        };
        let file = parse(&text).unwrap();
        let strategy = file.strategy.unwrap();
        assert_eq!(strategy.protocol, Some(ProtocolConfig::Icmp));
        assert_eq!(strategy.max_inflight, Some(24));
        assert_eq!(strategy.first_ttl, Some(1));
        assert_eq!(strategy.max_ttl, Some(64));
        let tui = file.tui.unwrap();
        assert_eq!(tui.tui_address_mode, Some(AddressMode::Host));
        assert_eq!(tui.tui_custom_columns.as_deref(), Some("holsravbwdt"));
        assert_eq!(tui.tui_max_addrs, Some(0));
        assert!(file.theme_colors.unwrap().items().len() >= 35);
        assert_eq!(file.bindings.unwrap().items().len(), 38);
    }
}
