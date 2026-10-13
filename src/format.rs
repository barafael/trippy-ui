//! Formatting of hop data into the strings the table and the details show.
//!
//! A faithful English port of the terminal frontend's `render/table.rs`.

use crate::columns::Column;
use crate::config::{AddressMode, AsMode, GeoIpMode, IcmpExtensionMode};
use crate::geoip;
use crate::settings::Settings;
use std::fmt::Write;
use std::net::IpAddr;
use trippy_core::{
    Dscp, Ecn, Extension, Extensions, Hop, IcmpPacketType, MplsLabelStackMember, NatStatus,
    UnknownExtension,
};
use trippy_dns::{AsInfo, DnsEntry, DnsResolver, Resolved, Resolver, Unresolved};

/// The text of one cell of the hops table.
pub fn cell(
    column: Column,
    hop: &Hop,
    dns: &DnsResolver,
    geoip: &geoip::Lookup,
    settings: &Settings,
) -> String {
    let total_recv = hop.total_recv();
    match column {
        Column::Ttl => hop.ttl().to_string(),
        Column::Host => host(hop, dns, geoip, settings),
        Column::LossPct => format!("{:.1}%", hop.loss_pct()),
        Column::Sent => hop.total_sent().to_string(),
        Column::Received => hop.total_recv().to_string(),
        Column::Failed => hop.total_failed().to_string(),
        Column::Last => float(hop.last_ms(), total_recv),
        Column::Average => {
            if total_recv > 0 {
                format!("{:.1}", hop.avg_ms())
            } else {
                String::new()
            }
        }
        Column::Best => float(hop.best_ms(), total_recv),
        Column::Worst => float(hop.worst_ms(), total_recv),
        Column::StdDev => {
            if total_recv > 1 {
                format!("{:.1}", hop.stddev_ms())
            } else {
                String::new()
            }
        }
        Column::Status => String::new(),
        Column::Jitter => float(hop.jitter_ms(), total_recv),
        Column::Javg => float(Some(hop.javg_ms()), total_recv),
        Column::Jmax => float(hop.jmax_ms(), total_recv),
        Column::Jinta => float(Some(hop.jinta()), total_recv),
        Column::LastSrcPort => port(hop.last_src_port()),
        Column::LastDestPort => port(hop.last_dest_port()),
        Column::LastSeq => hop.last_sequence().to_string(),
        Column::LastIcmpPacketType => icmp_packet_type(hop.last_icmp_packet_type()),
        Column::LastIcmpPacketCode => icmp_packet_code(hop.last_icmp_packet_type()),
        Column::LastNatStatus => match hop.last_nat_status() {
            NatStatus::NotApplicable => "n/a".into(),
            NatStatus::NotDetected => "No".into(),
            NatStatus::Detected => "Yes".into(),
        },
        Column::Floss => hop.total_forward_loss().to_string(),
        Column::Bloss => hop.total_backward_loss().to_string(),
        Column::FlossPct => format!("{:.1}%", hop.forward_loss_pct()),
        Column::Dscp => dscp(hop.dscp()),
        Column::Ecn => ecn(hop.ecn()),
        Column::Asn => asn(hop, dns, settings),
    }
}

/// The status of a hop, as a level the renderer colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// The target answered every probe.
    Ok,
    /// A hop other than the target lost some probes.
    Partial,
    /// A hop other than the target lost every probe.
    AllLost,
    /// The target lost some probes.
    TargetPartial,
    /// The target lost every probe.
    TargetLost,
}

/// Classifies a hop's status, distinguishing the target.
pub fn status_of(hop: &Hop, is_target: bool) -> Status {
    let lost = hop.total_sent() - hop.total_recv();
    match (lost, is_target) {
        (lost, true) if lost == hop.total_sent() => Status::TargetLost,
        (lost, true) if lost > 0 => Status::TargetPartial,
        (lost, false) if lost == hop.total_sent() => Status::AllLost,
        (lost, false) if lost > 0 => Status::Partial,
        _ => Status::Ok,
    }
}

fn float(value: Option<f64>, total_recv: usize) -> String {
    if total_recv > 0 {
        value.map(|v| format!("{v:.1}")).unwrap_or_default()
    } else {
        String::new()
    }
}

fn port(port: u16) -> String {
    if port > 0 {
        port.to_string()
    } else {
        "n/a".into()
    }
}

fn icmp_packet_type(packet_type: Option<IcmpPacketType>) -> String {
    match packet_type {
        None => "n/a".into(),
        Some(IcmpPacketType::TimeExceeded(_)) => "TE".into(),
        Some(IcmpPacketType::EchoReply(_)) => "ER".into(),
        Some(IcmpPacketType::Unreachable(_)) => "DU".into(),
        Some(IcmpPacketType::NotApplicable) => "NA".into(),
    }
}

fn icmp_packet_code(packet_type: Option<IcmpPacketType>) -> String {
    match packet_type {
        Some(
            IcmpPacketType::Unreachable(code)
            | IcmpPacketType::TimeExceeded(code)
            | IcmpPacketType::EchoReply(code),
        ) => code.0.to_string(),
        _ => "n/a".into(),
    }
}

fn dscp(dscp: Option<Dscp>) -> String {
    match dscp {
        Some(Dscp::Other(other)) => format!("0x{other:02x}"),
        Some(Dscp::DF) => "DF".into(),
        Some(Dscp::AF11) => "AF11".into(),
        Some(Dscp::AF12) => "AF12".into(),
        Some(Dscp::AF13) => "AF13".into(),
        Some(Dscp::AF21) => "AF21".into(),
        Some(Dscp::AF22) => "AF22".into(),
        Some(Dscp::AF23) => "AF23".into(),
        Some(Dscp::AF31) => "AF31".into(),
        Some(Dscp::AF32) => "AF32".into(),
        Some(Dscp::AF33) => "AF33".into(),
        Some(Dscp::AF41) => "AF41".into(),
        Some(Dscp::AF42) => "AF42".into(),
        Some(Dscp::AF43) => "AF43".into(),
        Some(Dscp::CS1) => "CS1".into(),
        Some(Dscp::CS2) => "CS2".into(),
        Some(Dscp::CS3) => "CS3".into(),
        Some(Dscp::CS4) => "CS4".into(),
        Some(Dscp::CS5) => "CS5".into(),
        Some(Dscp::CS6) => "CS6".into(),
        Some(Dscp::CS7) => "CS7".into(),
        Some(Dscp::EF) => "EF".into(),
        Some(Dscp::VA) => "VA".into(),
        Some(Dscp::LE) => "LE".into(),
        None => "n/a".into(),
    }
}

fn ecn(ecn: Option<Ecn>) -> String {
    match ecn {
        Some(Ecn::NotECT) => "NotECT".into(),
        Some(Ecn::ECT1) => "ECT1".into(),
        Some(Ecn::ECT0) => "ECT0".into(),
        Some(Ecn::CE) => "CE".into(),
        None => "n/a".into(),
    }
}

/// The host cell: every visible address, one per line.
pub fn host(hop: &Hop, dns: &DnsResolver, geoip: &geoip::Lookup, settings: &Settings) -> String {
    if hop.total_recv() == 0 {
        return "No response".into();
    }
    if settings.privacy_max_ttl >= Some(hop.ttl()) {
        return "**Hidden**".into();
    }
    visible_addresses(hop, settings.max_addrs)
        .into_iter()
        .map(|(addr, freq)| address(addr, freq, hop, dns, geoip, settings))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The addresses to show, most frequent first when limited.
fn visible_addresses(hop: &Hop, max_addrs: Option<u8>) -> Vec<(&IpAddr, usize)> {
    let addrs: Vec<(&IpAddr, usize)> = hop
        .addrs_with_counts()
        .map(|(addr, count)| (addr, *count))
        .collect();
    match max_addrs {
        None => addrs,
        Some(max) => {
            let mut addrs = addrs;
            addrs.sort_unstable_by_key(|a| std::cmp::Reverse(a.1));
            addrs.truncate(max as usize);
            addrs
        }
    }
}

/// One address, with hostname, GeoIP, extensions, NAT and frequency suffixes.
fn address(
    addr: &IpAddr,
    freq: usize,
    hop: &Hop,
    dns: &DnsResolver,
    geoip: &geoip::Lookup,
    settings: &Settings,
) -> String {
    let addr_fmt = match settings.address_mode {
        AddressMode::Ip => addr.to_string(),
        AddressMode::Host => dns_entry(*addr, dns, settings),
        AddressMode::Both => format!("{} ({addr})", dns_entry(*addr, dns, settings)),
    };
    let geo_fmt = match settings.geoip_mode {
        GeoIpMode::Off => None,
        GeoIpMode::Short => geoip.lookup(*addr).ok().flatten().map(|g| g.short_name()),
        GeoIpMode::Long => geoip.lookup(*addr).ok().flatten().map(|g| g.long_name()),
        GeoIpMode::Location => geoip.lookup(*addr).ok().flatten().map(|g| g.location()),
    };
    let ext_fmt = extensions(settings, hop);
    let nat = match hop.last_nat_status() {
        NatStatus::Detected => Some("NAT"),
        _ => None,
    };
    let freq_fmt = (hop.addr_count() > 1)
        .then(|| format!("{:.1}%", (freq as f64 / hop.total_recv() as f64) * 100.0));
    let mut address = addr_fmt;
    for suffix in [geo_fmt, ext_fmt, nat.map(str::to_string), freq_fmt]
        .into_iter()
        .flatten()
    {
        let _ = write!(address, " [{suffix}]");
    }
    address
}

/// The reverse DNS entry for an address, formatted according to the settings.
fn dns_entry(addr: IpAddr, dns: &DnsResolver, settings: &Settings) -> String {
    let entry = if settings.lookup_as_info {
        dns.lazy_reverse_lookup_with_asinfo(addr)
    } else {
        dns.lazy_reverse_lookup(addr)
    };
    format_dns_entry(entry, settings.lookup_as_info, settings.as_mode)
}

fn format_dns_entry(entry: DnsEntry, lookup_as_info: bool, as_mode: AsMode) -> String {
    match entry {
        DnsEntry::Resolved(Resolved::Normal(_, hosts)) => hosts.join(" "),
        DnsEntry::Resolved(Resolved::WithAsInfo(_, hosts, asinfo)) => {
            if lookup_as_info && !asinfo.asn.is_empty() {
                format!("{} {}", format_asinfo(&asinfo, as_mode), hosts.join(" "))
            } else {
                hosts.join(" ")
            }
        }
        DnsEntry::NotFound(Unresolved::Normal(ip)) | DnsEntry::Pending(ip) => ip.to_string(),
        DnsEntry::NotFound(Unresolved::WithAsInfo(ip, asinfo)) => {
            if lookup_as_info && !asinfo.asn.is_empty() {
                format!("{} {}", format_asinfo(&asinfo, as_mode), ip)
            } else {
                ip.to_string()
            }
        }
        DnsEntry::Failed(ip) => format!("Failed: {ip}"),
        DnsEntry::Timeout(ip) => format!("Timeout: {ip}"),
    }
}

fn format_asinfo(asinfo: &AsInfo, as_mode: AsMode) -> String {
    let detail = match as_mode {
        AsMode::Asn => String::new(),
        AsMode::Prefix => format!(" [{}]", asinfo.prefix),
        AsMode::CountryCode => format!(" [{}]", asinfo.cc),
        AsMode::Registry => format!(" [{}]", asinfo.registry),
        AsMode::Allocated => format!(" [{}]", asinfo.allocated),
        AsMode::Name => format!(" [{}]", asinfo.name),
    };
    format!("AS{}{detail}", asinfo.asn)
}

/// The ASN column.
fn asn(hop: &Hop, dns: &DnsResolver, settings: &Settings) -> String {
    if hop.total_recv() == 0 {
        "n/a".into()
    } else if settings.privacy_max_ttl >= Some(hop.ttl()) {
        "****".into()
    } else if !settings.lookup_as_info {
        "n/a".into()
    } else {
        visible_addresses(hop, settings.max_addrs)
            .into_iter()
            .map(
                |(addr, _)| match dns.lazy_reverse_lookup_with_asinfo(*addr) {
                    DnsEntry::Resolved(Resolved::WithAsInfo(_, _, info))
                    | DnsEntry::NotFound(Unresolved::WithAsInfo(_, info))
                        if !info.asn.is_empty() =>
                    {
                        format_asinfo(&info, AsMode::Asn)
                    }
                    DnsEntry::Pending(_) => String::new(),
                    DnsEntry::Failed(_) | DnsEntry::Timeout(_) => "?????".into(),
                    _ => "n/a".into(),
                },
            )
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn extensions(settings: &Settings, hop: &Hop) -> Option<String> {
    let extensions = hop.extensions()?;
    match settings.icmp_extension_mode {
        IcmpExtensionMode::Off => None,
        IcmpExtensionMode::Mpls => format_mpls(extensions),
        IcmpExtensionMode::Full => format_full(extensions),
        IcmpExtensionMode::All => Some(format_all(extensions)),
    }
}

fn format_mpls(extensions: &Extensions) -> Option<String> {
    let labels = extensions
        .extensions
        .iter()
        .filter_map(|ext| match ext {
            Extension::Unknown(_) => None,
            Extension::Mpls(stack) => Some(stack),
        })
        .flat_map(|ext| &ext.members)
        .map(|mem| mem.label.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    (!labels.is_empty()).then(|| format!("labels: {labels}"))
}

fn format_full(extensions: &Extensions) -> Option<String> {
    let formatted = extensions
        .extensions
        .iter()
        .filter_map(|ext| match ext {
            Extension::Unknown(_) => None,
            Extension::Mpls(stack) => Some(stack),
        })
        .flat_map(|ext| &ext.members)
        .map(format_mpls_member)
        .collect::<Vec<_>>()
        .join(", ");
    (!formatted.is_empty()).then_some(formatted)
}

fn format_all(extensions: &Extensions) -> String {
    extensions
        .extensions
        .iter()
        .flat_map(|ext| match ext {
            Extension::Unknown(unknown) => vec![format_unknown(unknown)],
            Extension::Mpls(stack) => stack.members.iter().map(format_mpls_member).collect(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_mpls_member(member: &MplsLabelStackMember) -> String {
    format!(
        "mpls(label={}, ttl={}, exp={}, bos={})",
        member.label, member.ttl, member.exp, member.bos
    )
}

fn format_unknown(unknown: &UnknownExtension) -> String {
    let bytes = unknown
        .bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "unknown(class={}, subtype={}, object={bytes})",
        unknown.class_num, unknown.class_subtype
    )
}

/// The expanded details of one address of a hop, one line per field.
pub fn details(
    hop: &Hop,
    offset: usize,
    dns: &DnsResolver,
    geoip: &geoip::Lookup,
    settings: &Settings,
) -> String {
    let Some(addr) = hop.addrs().nth(offset) else {
        return format!("Error: no addr for index {offset}");
    };
    let count = hop.addr_count();
    let index = offset + 1;
    let city = geoip.lookup(*addr).ok().flatten();
    let entry = if settings.lookup_as_info {
        dns.lazy_reverse_lookup_with_asinfo(*addr)
    } else {
        dns.lazy_reverse_lookup(*addr)
    };
    let (hostnames, asinfo) = match entry {
        DnsEntry::Pending(_) => (None, None),
        DnsEntry::Resolved(Resolved::WithAsInfo(_, hosts, info)) => (Some(hosts), Some(info)),
        DnsEntry::NotFound(Unresolved::WithAsInfo(_, info)) => (Some(vec![]), Some(info)),
        DnsEntry::Resolved(Resolved::Normal(_, hosts)) => (Some(hosts), None),
        DnsEntry::NotFound(Unresolved::Normal(_)) => (Some(vec![]), None),
        DnsEntry::Failed(ip) => return format!("Failed: {ip}"),
        DnsEntry::Timeout(ip) => return format!("Timeout: {ip}"),
    };
    let hosts = match hostnames {
        Some(hosts) if hosts.is_empty() => "Host: <not found>".to_string(),
        Some(hosts) => format!("Host: {}", hosts.join(" ")),
        None => "Host: <awaited>".to_string(),
    };
    let as_fmt = match (settings.lookup_as_info, asinfo) {
        (false, _) => "AS Name: <not enabled>\nAS Info: <not enabled>".to_string(),
        (true, None) => "AS Name: <awaited>\nAS Info: <awaited>".to_string(),
        (true, Some(info)) if info.asn.is_empty() => {
            "AS Name: <not found>\nAS Info: <not found>".to_string()
        }
        (true, Some(info)) => format!(
            "AS Name: AS{} {}\nAS Info: {} {} {}",
            info.asn, info.name, info.prefix, info.registry, info.allocated
        ),
    };
    let geo_fmt = match city {
        Some(city) => {
            let (lat, long, radius) = city.coordinates().unwrap_or_default();
            format!(
                "Geo: {}\nPos: {lat}, {long} (~{radius}km)",
                city.long_name()
            )
        }
        None => "Geo: <not found>\nPos: <not found>".to_string(),
    };
    let ext_fmt = match hop.extensions() {
        Some(extensions) => format!("Ext: [{}]", format_all(extensions)),
        None => "Ext: <none>".to_string(),
    };
    let nat = match hop.last_nat_status() {
        NatStatus::Detected => " [NAT]",
        _ => "",
    };
    format!("{addr}{nat} [{index} of {count}]\n{hosts}\n{as_fmt}\n{geo_fmt}\n{ext_fmt}")
}
