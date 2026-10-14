//! The configuration: the command line layered over the config file over
//! trippy's defaults, validated like trippy does.
//!
//! Report modes (`--mode csv` and friends) are not ported.

pub mod file;

use crate::bindings::Bindings;
use crate::cli::Args;
use crate::columns::Columns;
use crate::settings::Settings;
use anyhow::anyhow;
use clap::ValueEnum;
use file::ConfigFile;
use serde::Deserialize;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;
use trippy_core::{MAX_TTL, MultipathStrategy, PortDirection, PrivilegeMode, Protocol, defaults};
use trippy_dns::{IpAddrFamily, ResolveMethod};
use trippy_privilege::Privilege;

/// The defaults and bounds trippy applies, where trippy-core has none.
pub mod constants {
    use super::{AddressMode, AsMode, GeoIpMode, IcmpExtensionMode};
    use std::time::Duration;

    pub const DEFAULT_CUSTOM_COLUMNS: &str = "holsravbwdt";
    pub const DEFAULT_TUI_ADDRESS_MODE: AddressMode = AddressMode::Host;
    pub const DEFAULT_TUI_AS_MODE: AsMode = AsMode::Asn;
    pub const DEFAULT_TUI_ICMP_EXTENSION_MODE: IcmpExtensionMode = IcmpExtensionMode::Off;
    pub const DEFAULT_TUI_GEOIP_MODE: GeoIpMode = GeoIpMode::Off;
    pub const DEFAULT_TUI_REFRESH_RATE: Duration = Duration::from_millis(100);
    pub const DEFAULT_DNS_TIMEOUT: Duration = Duration::from_millis(5000);
    pub const DEFAULT_DNS_TTL: Duration = Duration::from_secs(300);
    pub const TUI_MIN_REFRESH_RATE_MS: Duration = Duration::from_millis(50);
    pub const TUI_MAX_REFRESH_RATE_MS: Duration = Duration::from_millis(1000);
    pub const MIN_READ_TIMEOUT_MS: Duration = Duration::from_millis(10);
    pub const MAX_READ_TIMEOUT_MS: Duration = Duration::from_millis(100);
    pub const MIN_GRACE_DURATION_MS: Duration = Duration::from_millis(10);
    pub const MAX_GRACE_DURATION_MS: Duration = Duration::from_millis(1000);
    pub const MIN_PACKET_SIZE_IPV4: u16 = 28;
    pub const MIN_PACKET_SIZE_IPV6: u16 = 48;
    pub const MAX_PACKET_SIZE: u16 = 1024;
}

/// How to render addresses in the host column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AddressMode {
    /// Show the IP address only.
    Ip,
    /// Show the reverse-lookup DNS hostname only.
    #[default]
    Host,
    /// Show both.
    Both,
}

/// How to render autonomous system information.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AsMode {
    /// Show the ASN.
    #[default]
    Asn,
    /// Show the AS prefix.
    Prefix,
    /// Show the country code.
    CountryCode,
    /// Show the registry name.
    Registry,
    /// Show the allocated date.
    Allocated,
    /// Show the AS name.
    Name,
}

/// How to render ICMP extensions in the host column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IcmpExtensionMode {
    /// Do not show extensions.
    #[default]
    Off,
    /// Show MPLS labels only.
    Mpls,
    /// Show full data for all known extensions.
    Full,
    /// Show full data for all classes.
    All,
}

/// How to render GeoIP information in the host column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeoIpMode {
    /// Do not show GeoIP data.
    #[default]
    Off,
    /// City, subdivision code and country code.
    Short,
    /// City, subdivision, country and continent.
    Long,
    /// Latitude and longitude.
    Location,
}

/// The tracing protocol, as named on the command line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProtocolConfig {
    Icmp,
    Udp,
    Tcp,
}

/// The address family, as named on the command line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AddressFamilyConfig {
    /// IPv4 only.
    Ipv4,
    /// IPv6 only.
    Ipv6,
    /// IPv6 with a fallback to IPv4.
    Ipv6ThenIpv4,
    /// IPv4 with a fallback to IPv6.
    Ipv4ThenIpv6,
    /// Whatever the system resolver returns first.
    System,
}

/// The multipath strategy, as named on the command line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MultipathStrategyConfig {
    Classic,
    Paris,
    Dublin,
}

/// The DNS resolver, as named on the command line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DnsResolveMethodConfig {
    System,
    Resolv,
    Google,
    Cloudflare,
}

/// A named enum used by the settings dialog.
pub trait Named: Sized + Copy + 'static {
    /// All values, in order.
    const ALL: &'static [Self];
    /// The name used on the command line and in the settings.
    fn name(self) -> &'static str;
}

impl Named for AddressMode {
    const ALL: &'static [Self] = &[Self::Ip, Self::Host, Self::Both];
    fn name(self) -> &'static str {
        match self {
            Self::Ip => "ip",
            Self::Host => "host",
            Self::Both => "both",
        }
    }
}

impl Named for AsMode {
    const ALL: &'static [Self] = &[
        Self::Asn,
        Self::Prefix,
        Self::CountryCode,
        Self::Registry,
        Self::Allocated,
        Self::Name,
    ];
    fn name(self) -> &'static str {
        match self {
            Self::Asn => "asn",
            Self::Prefix => "prefix",
            Self::CountryCode => "country-code",
            Self::Registry => "registry",
            Self::Allocated => "allocated",
            Self::Name => "name",
        }
    }
}

impl Named for IcmpExtensionMode {
    const ALL: &'static [Self] = &[Self::Off, Self::Mpls, Self::Full, Self::All];
    fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Mpls => "mpls",
            Self::Full => "full",
            Self::All => "all",
        }
    }
}

impl Named for GeoIpMode {
    const ALL: &'static [Self] = &[Self::Off, Self::Short, Self::Long, Self::Location];
    fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Short => "short",
            Self::Long => "long",
            Self::Location => "location",
        }
    }
}

/// The immutable tracing parameters, fixed when the tracers start.
#[derive(Clone, Debug)]
pub struct TraceConfig {
    pub protocol: Protocol,
    pub addr_family: IpAddrFamily,
    pub first_ttl: u8,
    pub max_ttl: u8,
    pub min_round_duration: Duration,
    pub max_round_duration: Duration,
    pub grace_duration: Duration,
    pub max_inflight: u8,
    pub initial_sequence: u16,
    pub tos: u8,
    pub icmp_extensions: bool,
    pub read_timeout: Duration,
    pub packet_size: u16,
    pub payload_pattern: u8,
    pub source_addr: Option<IpAddr>,
    pub interface: Option<String>,
    pub multipath_strategy: MultipathStrategy,
    pub port_direction: PortDirection,
    pub dns_resolve_method: ResolveMethod,
    pub dns_resolve_all: bool,
    pub dns_timeout: Duration,
    pub dns_ttl: Duration,
    pub max_samples: usize,
    pub max_flows: usize,
    pub privilege_mode: PrivilegeMode,
    pub geoip_mmdb_file: Option<String>,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            protocol: defaults::DEFAULT_STRATEGY_PROTOCOL,
            addr_family: IpAddrFamily::System,
            first_ttl: defaults::DEFAULT_STRATEGY_FIRST_TTL,
            max_ttl: defaults::DEFAULT_STRATEGY_MAX_TTL,
            min_round_duration: defaults::DEFAULT_STRATEGY_MIN_ROUND_DURATION,
            max_round_duration: defaults::DEFAULT_STRATEGY_MAX_ROUND_DURATION,
            grace_duration: defaults::DEFAULT_STRATEGY_GRACE_DURATION,
            max_inflight: defaults::DEFAULT_STRATEGY_MAX_INFLIGHT,
            initial_sequence: defaults::DEFAULT_STRATEGY_INITIAL_SEQUENCE,
            tos: defaults::DEFAULT_STRATEGY_TOS,
            icmp_extensions: defaults::DEFAULT_ICMP_EXTENSION_PARSE_MODE.is_enabled(),
            read_timeout: defaults::DEFAULT_STRATEGY_READ_TIMEOUT,
            packet_size: defaults::DEFAULT_STRATEGY_PACKET_SIZE,
            payload_pattern: defaults::DEFAULT_STRATEGY_PAYLOAD_PATTERN,
            source_addr: None,
            interface: None,
            multipath_strategy: defaults::DEFAULT_STRATEGY_MULTIPATH,
            port_direction: PortDirection::None,
            dns_resolve_method: ResolveMethod::System,
            dns_resolve_all: false,
            dns_timeout: constants::DEFAULT_DNS_TIMEOUT,
            dns_ttl: constants::DEFAULT_DNS_TTL,
            max_samples: defaults::DEFAULT_MAX_SAMPLES,
            max_flows: defaults::DEFAULT_MAX_FLOWS,
            privilege_mode: defaults::DEFAULT_PRIVILEGE_MODE,
            geoip_mmdb_file: None,
        }
    }
}

impl TraceConfig {
    /// The effective maximum number of flows, which is 1 for the classic
    /// strategy.
    pub const fn max_flows(&self) -> usize {
        match self.multipath_strategy {
            MultipathStrategy::Classic => 1,
            _ => self.max_flows,
        }
    }
}

/// Everything the window starts from.
#[derive(Clone, Debug)]
pub struct Options {
    /// Host names or addresses to trace.
    pub targets: Vec<String>,
    /// Tracing parameters.
    pub trace: TraceConfig,
    /// The initial view settings.
    pub settings: Settings,
    /// How often the window repaints.
    pub refresh_rate: Duration,
    /// The locale for GeoIP names, if one was given.
    pub locale: Option<String>,
    /// Theme colours in trippy's `item=colour` form, command line over file.
    pub theme_colors: Vec<(String, String)>,
    /// The key map, with the command line and the file applied.
    pub bindings: Bindings,
}

/// Reads the config file, layers the command line over it and validates.
pub fn load(args: Args, privilege: &Privilege, pid: u16) -> anyhow::Result<Options> {
    let cfg_file = if let Some(path) = &args.config_file {
        file::read_config_file(path)?
    } else {
        file::read_default_config_file()?.unwrap_or_default()
    };
    build(
        args,
        cfg_file,
        privilege.has_privileges(),
        privilege.needs_privileges(),
        pid,
    )
}

/// Layers the command line over a config file and validates the result.
#[allow(clippy::too_many_lines)]
pub fn build(
    args: Args,
    cfg_file: ConfigFile,
    has_privileges: bool,
    needs_privileges: bool,
    pid: u16,
) -> anyhow::Result<Options> {
    let cfg_trippy = cfg_file.trippy.unwrap_or_default();
    let cfg_strategy = cfg_file.strategy.unwrap_or_default();
    let cfg_bindings = cfg_file.bindings.unwrap_or_default();
    let cfg_theme = cfg_file.theme_colors.unwrap_or_default();
    let cfg_tui = cfg_file.tui.unwrap_or_default();
    let cfg_dns = cfg_file.dns.unwrap_or_default();
    validate_deprecated(&cfg_tui, &cfg_bindings)?;

    let unprivileged = layer_flag(
        args.unprivileged,
        cfg_trippy.unprivileged,
        defaults::DEFAULT_PRIVILEGE_MODE.is_unprivileged(),
    );
    let privilege_mode = if unprivileged {
        PrivilegeMode::Unprivileged
    } else {
        PrivilegeMode::Privileged
    };
    let protocol_cfg = layer(
        args.protocol,
        cfg_strategy.protocol,
        protocol_config(defaults::DEFAULT_STRATEGY_PROTOCOL),
    );
    let protocol = match (args.udp, args.tcp, args.icmp, protocol_cfg) {
        (false, false, false, ProtocolConfig::Udp) | (true, _, _, _) => Protocol::Udp,
        (false, false, false, ProtocolConfig::Tcp) | (_, true, _, _) => Protocol::Tcp,
        (false, false, false, ProtocolConfig::Icmp) | (_, _, true, _) => Protocol::Icmp,
    };
    let addr_family_cfg = layer(
        args.addr_family,
        cfg_strategy.addr_family,
        AddressFamilyConfig::System,
    );
    let addr_family = match (args.ipv4, args.ipv6, addr_family_cfg) {
        (true, _, _) | (false, false, AddressFamilyConfig::Ipv4) => IpAddrFamily::Ipv4Only,
        (_, true, _) | (false, false, AddressFamilyConfig::Ipv6) => IpAddrFamily::Ipv6Only,
        (false, false, AddressFamilyConfig::Ipv4ThenIpv6) => IpAddrFamily::Ipv4thenIpv6,
        (false, false, AddressFamilyConfig::Ipv6ThenIpv4) => IpAddrFamily::Ipv6thenIpv4,
        (false, false, AddressFamilyConfig::System) => IpAddrFamily::System,
    };
    let multipath_cfg = layer(
        args.multipath_strategy,
        cfg_strategy.multipath_strategy,
        multipath_config(defaults::DEFAULT_STRATEGY_MULTIPATH),
    );
    let multipath_strategy = match multipath_cfg {
        MultipathStrategyConfig::Classic => MultipathStrategy::Classic,
        MultipathStrategyConfig::Paris => MultipathStrategy::Paris,
        MultipathStrategyConfig::Dublin => MultipathStrategy::Dublin,
    };
    let target_port = layer_opt(args.target_port, cfg_strategy.target_port);
    let source_port = layer_opt(args.source_port, cfg_strategy.source_port);
    let port_direction = match (protocol, source_port, target_port, multipath_cfg) {
        (Protocol::Icmp, _, _, _) => PortDirection::None,
        (Protocol::Udp, None, None, _) => PortDirection::new_fixed_src(pid.max(1024)),
        (Protocol::Udp, Some(src), None, _) => {
            validate_source_port(src)?;
            PortDirection::new_fixed_src(src)
        }
        (Protocol::Tcp, None, None, _) => PortDirection::new_fixed_dest(80),
        (Protocol::Tcp, Some(src), None, _) => PortDirection::new_fixed_src(src),
        (_, None, Some(dest), _) => PortDirection::new_fixed_dest(dest),
        (
            Protocol::Udp,
            Some(src),
            Some(dest),
            MultipathStrategyConfig::Dublin | MultipathStrategyConfig::Paris,
        ) => {
            validate_source_port(src)?;
            PortDirection::new_fixed_both(src, dest)
        }
        (_, Some(_), Some(_), _) => {
            return Err(anyhow!(
                "only one of source-port and target-port may be fixed (except IPv4/udp protocol with dublin or paris strategy)"
            ));
        }
    };
    let dns_resolve_method = match layer(
        args.dns_resolve_method,
        cfg_dns.dns_resolve_method,
        DnsResolveMethodConfig::System,
    ) {
        DnsResolveMethodConfig::System => ResolveMethod::System,
        DnsResolveMethodConfig::Resolv => ResolveMethod::Resolv,
        DnsResolveMethodConfig::Google => ResolveMethod::Google,
        DnsResolveMethodConfig::Cloudflare => ResolveMethod::Cloudflare,
    };
    let trace = TraceConfig {
        protocol,
        addr_family,
        first_ttl: layer(
            args.first_ttl,
            cfg_strategy.first_ttl,
            defaults::DEFAULT_STRATEGY_FIRST_TTL,
        ),
        max_ttl: layer(
            args.max_ttl,
            cfg_strategy.max_ttl,
            defaults::DEFAULT_STRATEGY_MAX_TTL,
        ),
        min_round_duration: layer(
            args.min_round_duration,
            cfg_strategy.min_round_duration,
            defaults::DEFAULT_STRATEGY_MIN_ROUND_DURATION,
        ),
        max_round_duration: layer(
            args.max_round_duration,
            cfg_strategy.max_round_duration,
            defaults::DEFAULT_STRATEGY_MAX_ROUND_DURATION,
        ),
        grace_duration: layer(
            args.grace_duration,
            cfg_strategy.grace_duration,
            defaults::DEFAULT_STRATEGY_GRACE_DURATION,
        ),
        max_inflight: layer(
            args.max_inflight,
            cfg_strategy.max_inflight,
            defaults::DEFAULT_STRATEGY_MAX_INFLIGHT,
        ),
        initial_sequence: layer(
            args.initial_sequence,
            cfg_strategy.initial_sequence,
            defaults::DEFAULT_STRATEGY_INITIAL_SEQUENCE,
        ),
        tos: layer(args.tos, cfg_strategy.tos, defaults::DEFAULT_STRATEGY_TOS),
        icmp_extensions: layer_flag(
            args.icmp_extensions,
            cfg_strategy.icmp_extensions,
            defaults::DEFAULT_ICMP_EXTENSION_PARSE_MODE.is_enabled(),
        ),
        read_timeout: layer(
            args.read_timeout,
            cfg_strategy.read_timeout,
            defaults::DEFAULT_STRATEGY_READ_TIMEOUT,
        ),
        packet_size: layer(
            args.packet_size,
            cfg_strategy.packet_size,
            defaults::DEFAULT_STRATEGY_PACKET_SIZE,
        ),
        payload_pattern: layer(
            args.payload_pattern,
            cfg_strategy.payload_pattern,
            defaults::DEFAULT_STRATEGY_PAYLOAD_PATTERN,
        ),
        source_addr: layer_opt(args.source_address, cfg_strategy.source_address),
        interface: layer_opt(args.interface, cfg_strategy.interface),
        multipath_strategy,
        port_direction,
        dns_resolve_method,
        dns_resolve_all: layer_flag(args.dns_resolve_all, cfg_dns.dns_resolve_all, false),
        dns_timeout: layer(
            args.dns_timeout,
            cfg_dns.dns_timeout,
            constants::DEFAULT_DNS_TIMEOUT,
        ),
        dns_ttl: layer(args.dns_ttl, cfg_dns.dns_ttl, constants::DEFAULT_DNS_TTL),
        max_samples: layer(
            args.max_samples,
            cfg_strategy.max_samples,
            defaults::DEFAULT_MAX_SAMPLES,
        ),
        max_flows: layer(
            args.max_flows,
            cfg_strategy.max_flows,
            defaults::DEFAULT_MAX_FLOWS,
        ),
        privilege_mode,
        geoip_mmdb_file: layer_opt(args.geoip_mmdb_file, cfg_tui.geoip_mmdb_file),
    };

    let columns = layer(
        args.tui_custom_columns,
        cfg_tui.tui_custom_columns,
        String::from(constants::DEFAULT_CUSTOM_COLUMNS),
    );
    let columns = Columns::from_codes(&columns).map_err(|e| anyhow!(e))?;
    let timezone = layer_opt(args.tui_timezone, cfg_tui.tui_timezone)
        .as_deref()
        .map(chrono_tz::Tz::from_str)
        .transpose()
        .map_err(|e| anyhow!("invalid timezone: {e}"))?;
    let geoip_mode = layer(
        args.tui_geoip_mode,
        cfg_tui.tui_geoip_mode,
        constants::DEFAULT_TUI_GEOIP_MODE,
    );
    let lookup_as_info = layer_flag(args.dns_lookup_as_info, cfg_dns.dns_lookup_as_info, false);
    let settings = Settings {
        address_mode: layer(
            args.tui_address_mode,
            cfg_tui.tui_address_mode,
            constants::DEFAULT_TUI_ADDRESS_MODE,
        ),
        as_mode: layer(
            args.tui_as_mode,
            cfg_tui.tui_as_mode,
            constants::DEFAULT_TUI_AS_MODE,
        ),
        icmp_extension_mode: layer(
            args.tui_icmp_extension_mode,
            cfg_tui.tui_icmp_extension_mode,
            constants::DEFAULT_TUI_ICMP_EXTENSION_MODE,
        ),
        geoip_mode,
        max_addrs: match layer_opt(args.tui_max_addrs, cfg_tui.tui_max_addrs) {
            Some(n) if n > 0 => Some(n),
            _ => None,
        },
        privacy_max_ttl: layer_opt(args.tui_privacy_max_ttl, cfg_tui.tui_privacy_max_ttl),
        columns,
        lookup_as_info,
        timezone,
        ..Settings::default()
    };
    let refresh_rate = layer(
        args.tui_refresh_rate,
        cfg_tui.tui_refresh_rate,
        constants::DEFAULT_TUI_REFRESH_RATE,
    );

    validate_privilege(privilege_mode, has_privileges, needs_privileges)?;
    validate_strategy(multipath_strategy, unprivileged)?;
    validate_protocol_strategy(protocol, multipath_strategy)?;
    validate_multi(protocol, &args.targets, trace.dns_resolve_all)?;
    validate_ttl(trace.first_ttl, trace.max_ttl)?;
    validate_max_inflight(trace.max_inflight)?;
    validate_read_timeout(trace.read_timeout)?;
    validate_round_duration(trace.min_round_duration, trace.max_round_duration)?;
    validate_grace_duration(trace.grace_duration)?;
    validate_packet_size(addr_family, trace.packet_size)?;
    validate_tos(addr_family, trace.tos)?;
    validate_tui_refresh_rate(refresh_rate)?;
    validate_dns(dns_resolve_method, lookup_as_info)?;
    validate_geoip(geoip_mode, trace.geoip_mmdb_file.as_ref())?;

    // The command line wins over the file for the items it names.
    let mut theme_colors: Vec<(String, String)> = cfg_theme
        .items()
        .into_iter()
        .map(|(item, value)| (item.to_string(), value))
        .collect();
    for (item, value) in args.tui_theme_colors {
        theme_colors.retain(|(existing, _)| *existing != item);
        theme_colors.push((item, value));
    }
    let mut key_bindings: Vec<(String, String)> = cfg_bindings
        .items()
        .into_iter()
        .map(|(item, value)| (item.to_string(), value))
        .collect();
    for (item, value) in args.tui_key_bindings {
        key_bindings.retain(|(existing, _)| *existing != item);
        key_bindings.push((item, value));
    }
    let bindings = Bindings::from_items(&key_bindings)?;
    crate::theme::colors::validate(&theme_colors)?;

    Ok(Options {
        targets: args.targets,
        trace,
        settings,
        refresh_rate,
        locale: layer_opt(args.tui_locale, cfg_tui.tui_locale),
        theme_colors,
        bindings,
    })
}

const fn protocol_config(protocol: Protocol) -> ProtocolConfig {
    match protocol {
        Protocol::Icmp => ProtocolConfig::Icmp,
        Protocol::Udp => ProtocolConfig::Udp,
        Protocol::Tcp => ProtocolConfig::Tcp,
    }
}

const fn multipath_config(strategy: MultipathStrategy) -> MultipathStrategyConfig {
    match strategy {
        MultipathStrategy::Classic => MultipathStrategyConfig::Classic,
        MultipathStrategy::Paris => MultipathStrategyConfig::Paris,
        MultipathStrategy::Dublin => MultipathStrategyConfig::Dublin,
    }
}

fn layer<T>(cli: Option<T>, file: Option<T>, default: T) -> T {
    cli.or(file).unwrap_or(default)
}

fn layer_opt<T>(cli: Option<T>, file: Option<T>) -> Option<T> {
    cli.or(file)
}

const fn layer_flag(cli: bool, file: Option<bool>, default: bool) -> bool {
    match (cli, file) {
        (true, _) => true,
        (false, Some(value)) => value,
        (false, None) => default,
    }
}

fn validate_deprecated(
    cfg_tui: &file::ConfigTui,
    cfg_bindings: &file::ConfigBindings,
) -> anyhow::Result<()> {
    if cfg_tui.deprecated_tui_max_samples.is_some() {
        Err(anyhow!(
            "tui-max-samples in [tui] section is deprecated, use max-samples in [strategy] section instead"
        ))
    } else if cfg_tui.deprecated_tui_max_flows.is_some() {
        Err(anyhow!(
            "tui-max-flows in [tui] section is deprecated, use max-flows in [strategy] section instead"
        ))
    } else if cfg_bindings.deprecated_toggle_privacy.is_some() {
        Err(anyhow!(
            "toggle-privacy in [bindings] section is deprecated, use expand-privacy and contract-privacy instead"
        ))
    } else {
        Ok(())
    }
}

fn validate_privilege(
    privilege_mode: PrivilegeMode,
    has_privileges: bool,
    needs_privileges: bool,
) -> anyhow::Result<()> {
    const PRIVILEGE_URL: &str = "https://github.com/fujiapple852/trippy#privileges";
    match (privilege_mode, has_privileges, needs_privileges) {
        (PrivilegeMode::Privileged, true, _) | (PrivilegeMode::Unprivileged, _, false) => Ok(()),
        (PrivilegeMode::Privileged, false, true) => Err(anyhow!(
            "privileges are required\n\nsee {PRIVILEGE_URL} for details"
        )),
        (PrivilegeMode::Privileged, false, false) => Err(anyhow!(
            "privileges are required (hint: try adding -u to run in unprivileged mode)\n\nsee {PRIVILEGE_URL} for details"
        )),
        (PrivilegeMode::Unprivileged, false, true) => Err(anyhow!(
            "unprivileged mode not supported on this platform\n\nsee {PRIVILEGE_URL} for details"
        )),
        (PrivilegeMode::Unprivileged, true, true) => Err(anyhow!(
            "unprivileged mode not supported on this platform (hint: process is privileged so disable unprivileged mode)\n\nsee {PRIVILEGE_URL} for details"
        )),
    }
}

fn validate_strategy(strategy: MultipathStrategy, unprivileged: bool) -> anyhow::Result<()> {
    match (strategy, unprivileged) {
        (MultipathStrategy::Dublin, true) => Err(anyhow!(
            "Dublin tracing strategy cannot be used in unprivileged mode"
        )),
        (MultipathStrategy::Paris, true) => Err(anyhow!(
            "Paris tracing strategy cannot be used in unprivileged mode"
        )),
        _ => Ok(()),
    }
}

fn validate_protocol_strategy(
    protocol: Protocol,
    strategy: MultipathStrategy,
) -> anyhow::Result<()> {
    match (protocol, strategy) {
        (Protocol::Tcp | Protocol::Icmp, MultipathStrategy::Classic) | (Protocol::Udp, _) => Ok(()),
        (Protocol::Icmp, MultipathStrategy::Paris) => {
            Err(anyhow!("Paris multipath strategy not support for icmp"))
        }
        (Protocol::Icmp, MultipathStrategy::Dublin) => {
            Err(anyhow!("Dublin multipath strategy not support for icmp"))
        }
        (Protocol::Tcp, MultipathStrategy::Paris) => Err(anyhow!(
            "Paris multipath strategy not yet supported for tcp"
        )),
        (Protocol::Tcp, MultipathStrategy::Dublin) => Err(anyhow!(
            "Dublin multipath strategy not yet supported for tcp"
        )),
    }
}

fn validate_multi(protocol: Protocol, targets: &[String], resolve_all: bool) -> anyhow::Result<()> {
    match protocol {
        Protocol::Tcp | Protocol::Udp if targets.len() > 1 || resolve_all => Err(anyhow!(
            "only a single target may be specified for TCP and UDP tracing"
        )),
        _ => Ok(()),
    }
}

fn validate_ttl(first_ttl: u8, max_ttl: u8) -> anyhow::Result<()> {
    if !(1..=MAX_TTL).contains(&first_ttl) {
        Err(anyhow!(
            "first-ttl ({first_ttl}) must be in the range 1..{MAX_TTL}"
        ))
    } else if !(1..=MAX_TTL).contains(&max_ttl) {
        Err(anyhow!(
            "max-ttl ({max_ttl}) must be in the range 1..{MAX_TTL}"
        ))
    } else if first_ttl > max_ttl {
        Err(anyhow!(
            "first-ttl ({first_ttl}) must be less than or equal to max-ttl ({max_ttl})"
        ))
    } else {
        Ok(())
    }
}

fn validate_max_inflight(max_inflight: u8) -> anyhow::Result<()> {
    if max_inflight == 0 {
        Err(anyhow!(
            "max-inflight ({max_inflight}) must be greater than zero"
        ))
    } else {
        Ok(())
    }
}

fn validate_read_timeout(read_timeout: Duration) -> anyhow::Result<()> {
    if read_timeout < constants::MIN_READ_TIMEOUT_MS
        || read_timeout > constants::MAX_READ_TIMEOUT_MS
    {
        Err(anyhow!(
            "read-timeout ({:?}) must be between {:?} and {:?} inclusive",
            read_timeout,
            constants::MIN_READ_TIMEOUT_MS,
            constants::MAX_READ_TIMEOUT_MS
        ))
    } else {
        Ok(())
    }
}

fn validate_round_duration(min: Duration, max: Duration) -> anyhow::Result<()> {
    if min > max {
        Err(anyhow!(
            "max-round-duration ({max:?}) must not be less than min-round-duration ({min:?})"
        ))
    } else {
        Ok(())
    }
}

fn validate_grace_duration(grace_duration: Duration) -> anyhow::Result<()> {
    if grace_duration < constants::MIN_GRACE_DURATION_MS
        || grace_duration > constants::MAX_GRACE_DURATION_MS
    {
        Err(anyhow!(
            "grace-duration ({:?}) must be between {:?} and {:?} inclusive",
            grace_duration,
            constants::MIN_GRACE_DURATION_MS,
            constants::MAX_GRACE_DURATION_MS
        ))
    } else {
        Ok(())
    }
}

fn validate_packet_size(address_family: IpAddrFamily, packet_size: u16) -> anyhow::Result<()> {
    let min_size = match address_family {
        IpAddrFamily::Ipv4Only => constants::MIN_PACKET_SIZE_IPV4,
        IpAddrFamily::Ipv6Only
        | IpAddrFamily::Ipv6thenIpv4
        | IpAddrFamily::Ipv4thenIpv6
        | IpAddrFamily::System => constants::MIN_PACKET_SIZE_IPV6,
    };
    if (min_size..=constants::MAX_PACKET_SIZE).contains(&packet_size) {
        Ok(())
    } else {
        Err(anyhow!(
            "packet-size ({}) must be between {} and {} inclusive for {}",
            packet_size,
            min_size,
            constants::MAX_PACKET_SIZE,
            address_family,
        ))
    }
}

fn validate_source_port(source_port: u16) -> anyhow::Result<()> {
    if source_port < 1024 {
        Err(anyhow!("source-port ({source_port}) must be >= 1024"))
    } else {
        Ok(())
    }
}

fn validate_tui_refresh_rate(refresh_rate: Duration) -> anyhow::Result<()> {
    if refresh_rate < constants::TUI_MIN_REFRESH_RATE_MS
        || refresh_rate > constants::TUI_MAX_REFRESH_RATE_MS
    {
        Err(anyhow!(
            "tui-refresh-rate ({:?}) must be between {:?} and {:?} inclusive",
            refresh_rate,
            constants::TUI_MIN_REFRESH_RATE_MS,
            constants::TUI_MAX_REFRESH_RATE_MS
        ))
    } else {
        Ok(())
    }
}

fn validate_dns(dns_resolve_method: ResolveMethod, lookup_as_info: bool) -> anyhow::Result<()> {
    match dns_resolve_method {
        ResolveMethod::System if lookup_as_info => Err(anyhow!(
            "AS lookup not supported by resolver `system` (use '-r' to choose another resolver)"
        )),
        _ => Ok(()),
    }
}

fn validate_geoip(geoip_mode: GeoIpMode, geoip_mmdb_file: Option<&String>) -> anyhow::Result<()> {
    if matches!(
        geoip_mode,
        GeoIpMode::Short | GeoIpMode::Long | GeoIpMode::Location
    ) && geoip_mmdb_file.is_none()
    {
        Err(anyhow!(
            "geoip-mmdb-file must be given for tui-geoip-mode of `{geoip_mode:?}`"
        ))
    } else {
        Ok(())
    }
}

fn validate_tos(address_family: IpAddrFamily, tos: u8) -> anyhow::Result<()> {
    if cfg!(target_os = "windows") && address_family != IpAddrFamily::Ipv4Only && tos != 0 {
        Err(anyhow!(
            "setting tos is only supported for IPv4 on Windows (hint: try setting --ipv4 to enforce IPv4)"
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn args(argv: &[&str]) -> Args {
        Args::try_parse_from(std::iter::once("trippy-ui").chain(argv.iter().copied())).unwrap()
    }

    fn with_file(argv: &[&str], toml: &str) -> anyhow::Result<Options> {
        build(args(argv), file::parse(toml).unwrap(), true, true, 4242)
    }

    fn opts(argv: &[&str]) -> anyhow::Result<Options> {
        with_file(argv, "")
    }

    #[test]
    fn defaults_follow_trippy() {
        let options = opts(&["example.com"]).unwrap();
        assert_eq!(options.targets, ["example.com"]);
        assert_eq!(options.trace.protocol, Protocol::Icmp);
        assert_eq!(options.trace.addr_family, IpAddrFamily::System);
        assert_eq!(options.trace.port_direction, PortDirection::None);
        assert_eq!(options.trace.max_inflight, 24);
        assert_eq!(options.settings.address_mode, AddressMode::Host);
        assert_eq!(options.settings.max_addrs, None);
        assert_eq!(options.settings.columns.to_codes(), "holsravbwdt");
        assert_eq!(options.refresh_rate, Duration::from_millis(100));
        assert!(options.theme_colors.is_empty());
        assert_eq!(options.bindings, Bindings::default());
    }

    #[test]
    fn command_line_beats_file_beats_default() {
        let file = r#"
            [strategy]
            max-inflight = 10
            first-ttl = 3
            [tui]
            tui-address-mode = "ip"
            tui-max-addrs = 0
            [bindings]
            toggle-chart = "x"
            toggle-map = "v"
            [theme-colors]
            bg-color = "black"
        "#;
        let options = with_file(
            &[
                "-U",
                "5",
                "-a",
                "both",
                "--tui-key-bindings",
                "toggle-chart=y",
                "x",
            ],
            file,
        )
        .unwrap();
        assert_eq!(options.trace.max_inflight, 5);
        assert_eq!(options.trace.first_ttl, 3);
        assert_eq!(options.settings.address_mode, AddressMode::Both);
        assert_eq!(options.settings.max_addrs, None);
        assert_eq!(
            options.bindings.display(crate::bindings::Item::ToggleMap),
            "v"
        );
        assert_eq!(
            options.bindings.display(crate::bindings::Item::ToggleChart),
            "y"
        );
        assert!(
            with_file(&["x"], "[bindings]\ntoggle-chart = \"m\"")
                .unwrap_err()
                .to_string()
                .contains("Duplicate key bindings")
        );
        assert_eq!(
            options.theme_colors,
            [(String::from("bg-color"), String::from("black"))]
        );
    }

    #[test]
    fn protocol_flags_and_ports() {
        let udp = opts(&["--udp", "x"]).unwrap();
        assert_eq!(udp.trace.port_direction, PortDirection::new_fixed_src(4242));
        let tcp = opts(&["--tcp", "x"]).unwrap();
        assert_eq!(tcp.trace.port_direction, PortDirection::new_fixed_dest(80));
        let paris = opts(&["--udp", "-R", "paris", "-S", "5000", "-P", "33434", "x"]).unwrap();
        assert_eq!(
            paris.trace.port_direction,
            PortDirection::new_fixed_both(5000, 33434)
        );
        assert!(opts(&["--udp", "-S", "80", "x"]).is_err());
        assert!(opts(&["--tcp", "-S", "5000", "-P", "80", "x"]).is_err());
        let file = opts(&["x"]).unwrap();
        assert_eq!(file.trace.protocol, Protocol::Icmp);
        let from_file = with_file(&["x"], "[strategy]\nprotocol = \"tcp\"").unwrap();
        assert_eq!(from_file.trace.protocol, Protocol::Tcp);
    }

    #[test]
    fn validation_matches_trippy() {
        let message = |argv: &[&str]| opts(argv).unwrap_err().to_string();
        assert!(message(&["-f", "10", "-t", "5", "x"]).contains("first-ttl (10)"));
        assert!(message(&["-U", "0", "x"]).contains("max-inflight (0)"));
        assert!(message(&["--read-timeout", "5s", "x"]).contains("read-timeout"));
        assert!(message(&["-i", "2s", "-T", "1s", "x"]).contains("max-round-duration"));
        assert!(message(&["-g", "1ms", "x"]).contains("grace-duration"));
        assert!(message(&["--packet-size", "10", "x"]).contains("packet-size (10)"));
        assert!(message(&["--tui-refresh-rate", "5s", "x"]).contains("tui-refresh-rate"));
        assert!(message(&["-z", "x"]).contains("AS lookup not supported"));
        assert!(message(&["--tui-geoip-mode", "short", "x"]).contains("geoip-mmdb-file"));
        assert!(message(&["--udp", "x", "y"]).contains("single target"));
        assert!(message(&["--tcp", "-R", "paris", "x"]).contains("Paris"));
        assert!(message(&["-R", "dublin", "x"]).contains("Dublin"));
        assert!(message(&["--udp", "-R", "paris", "-u", "x"]).contains("unprivileged"));
        assert!(message(&["--tui-timezone", "Mars/Olympus", "x"]).contains("timezone"));
        assert!(message(&["--tui-custom-columns", "hh", "x"]).contains("duplicate"));
        assert!(message(&["--tui-theme-colors", "bg-color=octarine", "x"]).contains("color"));
        assert!(message(&["--tui-theme-colors", "nope=red", "x"]).contains("theme item"));
        assert!(
            with_file(&["x"], "[tui]\ntui-max-samples = 5")
                .unwrap_err()
                .to_string()
                .contains("deprecated")
        );
    }

    #[test]
    fn privileges_are_checked_like_trippy() {
        let check = |argv: &[&str], has, needs| {
            build(args(argv), ConfigFile::default(), has, needs, 1).map(|_| ())
        };
        assert!(check(&["x"], true, true).is_ok());
        assert!(check(&["x"], false, true).is_err());
        assert!(check(&["-u", "x"], false, false).is_ok());
        assert!(check(&["-u", "x"], false, true).is_err());
        assert!(
            check(&["x"], false, false)
                .unwrap_err()
                .to_string()
                .contains("-u")
        );
    }
}
