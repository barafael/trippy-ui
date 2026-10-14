//! The tracing engine: one tracer per target, plus its latest snapshot.

use crate::config::TraceConfig;
use std::net::IpAddr;
use std::thread::JoinHandle;
use trippy_core::{Builder, IcmpExtensionParseMode, State, Tracer};
use trippy_dns::{DnsResolver, Resolver};

/// A running trace and the most recent snapshot of its data.
pub struct Trace {
    /// The tracer handle, used to take snapshots and clear data.
    tracer: Tracer,
    /// The background thread running the trace, until it finishes.
    handle: Option<JoinHandle<Result<(), trippy_core::Error>>>,
    /// The target hostname as given on the command line.
    pub hostname: String,
    /// The resolved target address.
    pub target_addr: IpAddr,
    /// The latest snapshot.
    pub snapshot: State,
    /// Why the tracing thread stopped, if it failed.
    pub error: Option<String>,
}

impl Trace {
    /// Why the trace failed: the error the tracer recorded in its state, or
    /// the one its thread returned.
    pub fn failure(&self) -> Option<&str> {
        self.snapshot.error().or(self.error.as_deref())
    }

    /// A trace that never ran, holding a prepared snapshot.
    #[cfg(test)]
    pub fn for_test(cfg: &TraceConfig, hostname: &str, addr: IpAddr, snapshot: State) -> Self {
        let tracer = Builder::new(addr)
            .protocol(cfg.protocol)
            .multipath_strategy(cfg.multipath_strategy)
            .port_direction(cfg.port_direction)
            .max_flows(cfg.max_flows())
            .max_samples(cfg.max_samples)
            .build()
            .expect("tracer");
        Self {
            tracer,
            handle: None,
            hostname: hostname.to_string(),
            target_addr: addr,
            snapshot,
            error: None,
        }
    }
}

impl Trace {
    /// Resolves the target, builds a tracer and spawns it.
    pub fn start(
        cfg: &TraceConfig,
        hostname: String,
        addr: IpAddr,
        trace_id: u16,
    ) -> anyhow::Result<Self> {
        let (tracer, handle) = Builder::new(addr)
            .interface(cfg.interface.clone())
            .source_addr(cfg.source_addr)
            .privilege_mode(cfg.privilege_mode)
            .protocol(cfg.protocol)
            .packet_size(cfg.packet_size)
            .payload_pattern(cfg.payload_pattern)
            .tos(cfg.tos)
            .icmp_extension_parse_mode(if cfg.icmp_extensions {
                IcmpExtensionParseMode::Enabled
            } else {
                IcmpExtensionParseMode::Disabled
            })
            .read_timeout(cfg.read_timeout)
            .tcp_connect_timeout(cfg.min_round_duration)
            .trace_identifier(trace_id)
            .first_ttl(cfg.first_ttl)
            .max_ttl(cfg.max_ttl)
            .grace_duration(cfg.grace_duration)
            .max_inflight(cfg.max_inflight)
            .initial_sequence(cfg.initial_sequence)
            .multipath_strategy(cfg.multipath_strategy)
            .port_direction(cfg.port_direction)
            .min_round_duration(cfg.min_round_duration)
            .max_round_duration(cfg.max_round_duration)
            .max_flows(cfg.max_flows())
            .max_samples(cfg.max_samples)
            .drop_privileges(true)
            .build()?
            .spawn()?;
        let snapshot = tracer.snapshot();
        Ok(Self {
            tracer,
            handle: Some(handle),
            hostname,
            target_addr: addr,
            snapshot,
            error: None,
        })
    }

    /// Refreshes the snapshot while the tracer runs, and notices when it has
    /// stopped; the last snapshot is kept after that.
    pub fn poll(&mut self) {
        let Some(handle) = &self.handle else {
            return;
        };
        if handle.is_finished() {
            let handle = self.handle.take().expect("handle was just checked");
            self.error = match handle.join() {
                Ok(Ok(())) => None,
                Ok(Err(e)) => Some(e.to_string()),
                Err(_) => Some(String::from("the tracing thread panicked")),
            };
        }
        self.snapshot = self.tracer.snapshot();
    }

    /// Discards all tracing data and starts again from an empty state.
    pub fn clear(&mut self) {
        self.tracer.clear();
        self.snapshot = self.tracer.snapshot();
    }

    /// The source address, once the tracer has discovered it.
    pub fn source_addr(&self) -> Option<IpAddr> {
        self.tracer.source_addr()
    }

    /// The maximum number of flows the tracer allows.
    pub fn max_flows(&self) -> usize {
        self.tracer.max_flows()
    }

    /// The maximum number of samples kept per hop.
    pub fn max_samples(&self) -> usize {
        self.tracer.max_samples()
    }
}

/// Resolves every target, honouring `--dns-resolve-all`.
pub fn resolve_targets(
    targets: &[String],
    cfg: &TraceConfig,
    resolver: &DnsResolver,
) -> anyhow::Result<Vec<(String, IpAddr)>> {
    let mut resolved = Vec::new();
    for target in targets {
        let addrs = resolver
            .lookup(target)
            .map_err(|e| anyhow::anyhow!("failed to resolve target: {target} ({e})"))?;
        for (i, addr) in addrs.into_iter().enumerate() {
            if i > 0 && !cfg.dns_resolve_all {
                break;
            }
            let hostname = if cfg.dns_resolve_all {
                format!("{target} [{}]", i + 1)
            } else {
                target.clone()
            };
            resolved.push((hostname, addr));
        }
    }
    Ok(resolved)
}
