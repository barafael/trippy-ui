//! The tracing configuration and the command line.
//!
//! Follows trippy's command line for the options that affect tracing and the
//! interactive view. Report modes (`--mode csv` and friends) are not ported.

use crate::settings::Settings;
use std::ffi::OsString;
use std::net::IpAddr;
use std::time::Duration;
use trippy_core::{MultipathStrategy, PortDirection, PrivilegeMode, Protocol, defaults};
use trippy_dns::{IpAddrFamily, ResolveMethod};

/// How to render addresses in the host column.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

/// A named enum used by the command line and the settings dialog.
pub trait Named: Sized + Copy + 'static {
    /// All values, in order.
    const ALL: &'static [Self];
    /// The name used on the command line and in the settings.
    fn name(self) -> &'static str;
    /// Parses a name, case-insensitively.
    fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|v| v.name().eq_ignore_ascii_case(name))
    }
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
            addr_family: IpAddrFamily::Ipv4thenIpv6,
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
            dns_timeout: Duration::from_secs(5),
            dns_ttl: Duration::from_secs(300),
            max_samples: defaults::DEFAULT_MAX_SAMPLES,
            max_flows: defaults::DEFAULT_MAX_FLOWS,
            privilege_mode: defaults::DEFAULT_PRIVILEGE_MODE,
            geoip_mmdb_file: None,
        }
    }
}

/// The maximum number of flows, which is 1 for the classic strategy.
impl TraceConfig {
    /// The effective maximum number of flows.
    pub const fn max_flows(&self) -> usize {
        match self.multipath_strategy {
            MultipathStrategy::Classic => 1,
            _ => self.max_flows,
        }
    }
}

/// Everything parsed from the command line.
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
}

pub const USAGE: &str = "\
Usage: trippy-ui [OPTIONS] <TARGET>...

Trace the network path to one or more targets in a window.

Tracing:
  -p, --protocol <icmp|udp|tcp>       Tracing protocol [default: icmp]
  -F, --addr-family <FAMILY>          ipv4, ipv6, ipv6-then-ipv4, ipv4-then-ipv6, system
      --udp / --tcp / --icmp          Shorthand for --protocol
  -4, --ipv4                          Use IPv4 only
  -6, --ipv6                          Use IPv6 only
  -u, --unprivileged                  Use unprivileged mode where supported
  -R, --multipath-strategy <STRATEGY> classic, paris, dublin [default: classic]
  -U, --max-inflight <N>              Maximum probes in flight [default: 24]
  -S, --source-port <N>               Fix the source port
  -P, --target-port <N>               Fix the target port
  -A, --source-address <ADDR>         Source address to use
  -I, --interface <NAME>              Interface to use
  -f, --first-ttl <N>                 First TTL [default: 1]
  -t, --max-ttl <N>                   Maximum TTL [default: 64]
  -i, --min-round-duration <DURATION> Minimum round duration [default: 1s]
  -T, --max-round-duration <DURATION> Maximum round duration [default: 1s]
  -g, --grace-duration <DURATION>     Grace duration [default: 100ms]
      --initial-sequence <N>          Initial sequence number [default: 33434]
      --packet-size <N>               Packet size [default: 84]
      --payload-pattern <N>           Payload pattern [default: 0]
  -Q, --tos <N>                       Type of service [default: 0]
  -e, --icmp-extensions               Parse ICMP extensions
      --read-timeout <DURATION>       Read timeout [default: 10ms]
  -s, --max-samples <N>               Samples per hop [default: 256]
      --max-flows <N>                 Maximum flows [default: 64]

DNS:
  -r, --dns-resolve-method <METHOD>   system, resolv, google, cloudflare [default: system]
  -y, --dns-resolve-all               Resolve all addresses of each target
  -z, --dns-lookup-as-info            Look up AS information for hops
      --dns-timeout <DURATION>        DNS timeout [default: 5s]
      --dns-ttl <DURATION>            DNS cache TTL [default: 300s]

View:
  -a, --tui-address-mode <MODE>       ip, host, both [default: ip]
      --tui-as-mode <MODE>            asn, prefix, country-code, registry, allocated, name
      --tui-icmp-extension-mode <M>   off, mpls, full, all [default: off]
      --tui-geoip-mode <MODE>         off, short, long, location [default: off]
  -M, --tui-max-addrs <N>             Maximum addresses shown per hop
      --tui-custom-columns <COLUMNS>  Columns to show [default: holsravbwdt]
      --tui-privacy-max-ttl <N>       Hide hops up to this TTL
      --tui-refresh-rate <DURATION>   Repaint interval [default: 100ms]
      --tui-timezone <TZ>             Timezone for the clock
  -G, --geoip-mmdb-file <PATH>        MaxMind GeoIP database file

  -h, --help                          Print this help
  -V, --version                       Print the version";

/// Parses the command line.
///
/// Returns `None` when the program printed help or version and should exit.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Option<Options>, String> {
    let mut trace = TraceConfig::default();
    let mut targets = Vec::new();
    let mut address_mode = AddressMode::default();
    let mut as_mode = AsMode::default();
    let mut icmp_extension_mode = IcmpExtensionMode::default();
    let mut geoip_mode = GeoIpMode::default();
    let mut max_addrs = None;
    let mut privacy_max_ttl = None;
    let mut columns = crate::columns::Columns::default();
    let mut lookup_as_info = false;
    let mut timezone = None;
    let mut refresh_rate = Duration::from_millis(100);
    let mut protocol_flag = None;
    let mut addr_family_flag = None;
    let mut multipath_flag = None;
    let mut source_port = None;
    let mut target_port = None;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let arg = arg.to_string_lossy().into_owned();
        let mut value = |flag: &str| -> Result<String, String> {
            args.next()
                .map(|v| v.to_string_lossy().into_owned())
                .ok_or_else(|| format!("{flag} needs a value\n\n{USAGE}"))
        };
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(None);
            }
            "-V" | "--version" => {
                println!("trippy-ui {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "-p" | "--protocol" => protocol_flag = Some(value(&arg)?),
            "--udp" => protocol_flag = Some(String::from("udp")),
            "--tcp" => protocol_flag = Some(String::from("tcp")),
            "--icmp" => protocol_flag = Some(String::from("icmp")),
            "-F" | "--addr-family" => addr_family_flag = Some(value(&arg)?),
            "-4" | "--ipv4" => addr_family_flag = Some(String::from("ipv4")),
            "-6" | "--ipv6" => addr_family_flag = Some(String::from("ipv6")),
            "-u" | "--unprivileged" => trace.privilege_mode = PrivilegeMode::Unprivileged,
            "-R" | "--multipath-strategy" => multipath_flag = Some(value(&arg)?),
            "-U" | "--max-inflight" => trace.max_inflight = number(&arg, value(&arg)?)?,
            "-S" | "--source-port" => source_port = Some(number(&arg, value(&arg)?)?),
            "-P" | "--target-port" => target_port = Some(number(&arg, value(&arg)?)?),
            "-A" | "--source-address" => {
                let addr = value(&arg)?;
                trace.source_addr = Some(
                    addr.parse()
                        .map_err(|_| format!("not an address: `{addr}`"))?,
                );
            }
            "-I" | "--interface" => trace.interface = Some(value(&arg)?),
            "-f" | "--first-ttl" => trace.first_ttl = number(&arg, value(&arg)?)?,
            "-t" | "--max-ttl" => trace.max_ttl = number(&arg, value(&arg)?)?,
            "-i" | "--min-round-duration" => {
                trace.min_round_duration = duration(&arg, value(&arg)?)?
            }
            "-T" | "--max-round-duration" => {
                trace.max_round_duration = duration(&arg, value(&arg)?)?
            }
            "-g" | "--grace-duration" => trace.grace_duration = duration(&arg, value(&arg)?)?,
            "--initial-sequence" => trace.initial_sequence = number(&arg, value(&arg)?)?,
            "--packet-size" => trace.packet_size = number(&arg, value(&arg)?)?,
            "--payload-pattern" => trace.payload_pattern = number(&arg, value(&arg)?)?,
            "-Q" | "--tos" => trace.tos = number(&arg, value(&arg)?)?,
            "-e" | "--icmp-extensions" => trace.icmp_extensions = true,
            "--read-timeout" => trace.read_timeout = duration(&arg, value(&arg)?)?,
            "-s" | "--max-samples" => trace.max_samples = number(&arg, value(&arg)?)?,
            "--max-flows" => trace.max_flows = number(&arg, value(&arg)?)?,
            "-r" | "--dns-resolve-method" => {
                trace.dns_resolve_method = match value(&arg)?.as_str() {
                    "system" => ResolveMethod::System,
                    "resolv" => ResolveMethod::Resolv,
                    "google" => ResolveMethod::Google,
                    "cloudflare" => ResolveMethod::Cloudflare,
                    other => return Err(format!("unknown DNS resolver `{other}`\n\n{USAGE}")),
                }
            }
            "-y" | "--dns-resolve-all" => trace.dns_resolve_all = true,
            "-z" | "--dns-lookup-as-info" => lookup_as_info = true,
            "--dns-timeout" => trace.dns_timeout = duration(&arg, value(&arg)?)?,
            "--dns-ttl" => trace.dns_ttl = duration(&arg, value(&arg)?)?,
            "-a" | "--tui-address-mode" => address_mode = named(&arg, value(&arg)?)?,
            "--tui-as-mode" => as_mode = named(&arg, value(&arg)?)?,
            "--tui-icmp-extension-mode" => icmp_extension_mode = named(&arg, value(&arg)?)?,
            "--tui-geoip-mode" => geoip_mode = named(&arg, value(&arg)?)?,
            "-M" | "--tui-max-addrs" => max_addrs = Some(number(&arg, value(&arg)?)?),
            "--tui-custom-columns" => {
                let codes = value(&arg)?;
                columns = crate::columns::Columns::from_codes(&codes)?;
            }
            "--tui-privacy-max-ttl" => privacy_max_ttl = Some(number(&arg, value(&arg)?)?),
            "--tui-refresh-rate" => refresh_rate = duration(&arg, value(&arg)?)?,
            "--tui-timezone" => timezone = Some(value(&arg)?),
            "-G" | "--geoip-mmdb-file" => trace.geoip_mmdb_file = Some(value(&arg)?),
            flag if flag.starts_with('-') && flag.len() > 1 => {
                return Err(format!("unknown option `{flag}`\n\n{USAGE}"));
            }
            _ => targets.push(arg),
        }
    }

    if targets.is_empty() {
        return Err(format!("no target given\n\n{USAGE}"));
    }
    if lookup_as_info && trace.dns_resolve_method == ResolveMethod::System {
        return Err(format!(
            "AS lookup is not supported by the `system` resolver (use `-r` to choose another)\n\n{USAGE}"
        ));
    }

    trace.protocol = match protocol_flag.as_deref() {
        Some("udp") => Protocol::Udp,
        Some("tcp") => Protocol::Tcp,
        Some("icmp") | None => Protocol::Icmp,
        Some(other) => return Err(format!("unknown protocol `{other}`\n\n{USAGE}")),
    };
    trace.addr_family = match addr_family_flag.as_deref() {
        Some("ipv4") => IpAddrFamily::Ipv4Only,
        Some("ipv6") => IpAddrFamily::Ipv6Only,
        Some("ipv6-then-ipv4") => IpAddrFamily::Ipv6thenIpv4,
        Some("ipv4-then-ipv6") => IpAddrFamily::Ipv4thenIpv6,
        Some("system") => IpAddrFamily::System,
        None => IpAddrFamily::Ipv4thenIpv6,
        Some(other) => return Err(format!("unknown address family `{other}`\n\n{USAGE}")),
    };
    trace.multipath_strategy = match multipath_flag.as_deref() {
        Some("classic") | None => MultipathStrategy::Classic,
        Some("paris") => MultipathStrategy::Paris,
        Some("dublin") => MultipathStrategy::Dublin,
        Some(other) => return Err(format!("unknown multipath strategy `{other}`\n\n{USAGE}")),
    };
    trace.port_direction = match (trace.protocol, source_port, target_port) {
        (Protocol::Icmp, _, _) => PortDirection::None,
        (Protocol::Udp, None, None) => PortDirection::new_fixed_src(1024),
        (Protocol::Tcp, None, None) => PortDirection::new_fixed_dest(80),
        (_, Some(src), None) => {
            if src < 1024 {
                return Err(format!("source port ({src}) must be >= 1024"));
            }
            PortDirection::new_fixed_src(src)
        }
        (_, None, Some(dest)) => PortDirection::new_fixed_dest(dest),
        (_, Some(_), Some(_)) => {
            return Err(format!(
                "only one of source-port and target-port may be fixed\n\n{USAGE}"
            ));
        }
    };
    if trace.multipath_strategy != MultipathStrategy::Classic && trace.protocol != Protocol::Udp {
        return Err(format!(
            "the paris and dublin strategies require the udp protocol\n\n{USAGE}"
        ));
    }

    let settings = Settings {
        address_mode,
        as_mode,
        icmp_extension_mode,
        geoip_mode,
        max_addrs,
        privacy_max_ttl,
        columns,
        lookup_as_info,
        timezone,
        ..Settings::default()
    };
    Ok(Some(Options {
        targets,
        trace,
        settings,
        refresh_rate,
    }))
}

fn named<T: Named>(flag: &str, value: String) -> Result<T, String> {
    T::from_name(&value).ok_or_else(|| format!("unknown value `{value}` for {flag}\n\n{USAGE}"))
}

fn number<T: std::str::FromStr>(flag: &str, value: String) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("{flag} needs a number, got `{value}`\n\n{USAGE}"))
}

fn duration(flag: &str, value: String) -> Result<Duration, String> {
    humantime::parse_duration(&value)
        .map_err(|_| format!("{flag} needs a duration, got `{value}`\n\n{USAGE}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_strs(args: &[&str]) -> Result<Option<Options>, String> {
        parse(args.iter().map(OsString::from))
    }

    #[test]
    fn flags_and_targets() {
        let options = parse_strs(&[
            "--udp",
            "-R",
            "paris",
            "-U",
            "10",
            "--tui-custom-columns",
            "hoA",
            "example.com",
            "example.org",
        ])
        .unwrap()
        .unwrap();
        assert_eq!(options.targets, ["example.com", "example.org"]);
        assert_eq!(options.trace.protocol, Protocol::Udp);
        assert_eq!(options.trace.multipath_strategy, MultipathStrategy::Paris);
        assert_eq!(options.trace.max_inflight, 10);
        assert_eq!(options.settings.columns.to_codes(), "hoA");
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse_strs(&["--protocol", "sctp", "x"]).is_err());
        assert!(parse_strs(&["-U", "many", "x"]).is_err());
        assert!(parse_strs(&["--tui-custom-columns", "zz", "x"]).is_err());
        assert!(parse_strs(&["--bogus", "x"]).is_err());
        assert!(parse_strs(&[]).is_err());
    }

    #[test]
    fn help_and_version_exit() {
        assert!(parse_strs(&["--help"]).unwrap().is_none());
        assert!(parse_strs(&["-V"]).unwrap().is_none());
    }

    #[test]
    fn protocol_defaults_ports() {
        let udp = parse_strs(&["--udp", "x"]).unwrap().unwrap();
        assert_eq!(udp.trace.port_direction, PortDirection::new_fixed_src(1024));
        let tcp = parse_strs(&["--tcp", "x"]).unwrap().unwrap();
        assert_eq!(tcp.trace.port_direction, PortDirection::new_fixed_dest(80));
        let icmp = parse_strs(&["x"]).unwrap().unwrap();
        assert_eq!(icmp.trace.port_direction, PortDirection::None);
    }
}
