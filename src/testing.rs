//! Test helpers: tracer states built from literal probes, so formatting and
//! commands can be exercised against real hops without tracing.

use std::net::IpAddr;
use std::time::{Duration, SystemTime};
use trippy_core::{
    CompletionReason, Flags, IcmpPacketType, Port, Probe, ProbeComplete, ProbeStatus, Round,
    RoundId, Sequence, State, TimeToLive, TraceId,
};

/// One probe of a round: the hop it answered from, and its round-trip time in
/// milliseconds, or `None` when it was lost.
#[derive(Clone, Copy, Debug)]
pub struct Reply {
    pub ttl: u8,
    pub addr: &'static str,
    pub rtt_ms: Option<f64>,
}

/// A reply from `addr` at `ttl` after `rtt_ms` milliseconds.
pub const fn reply(ttl: u8, addr: &'static str, rtt_ms: f64) -> Reply {
    Reply {
        ttl,
        addr,
        rtt_ms: Some(rtt_ms),
    }
}

/// A probe at `ttl` that was never answered.
pub const fn lost(ttl: u8) -> Reply {
    Reply {
        ttl,
        addr: "0.0.0.0",
        rtt_ms: None,
    }
}

/// A tracer state after the given rounds, applied in order.
pub fn state_from_rounds(rounds: &[&[Reply]]) -> State {
    state_from_rounds_with(trippy_core::defaults::DEFAULT_MAX_SAMPLES, rounds)
}

/// A tracer state keeping `max_samples` per hop, after the given rounds.
///
/// The state comes from a built (never run) tracer, because `State::default`
/// numbers its first flow 0 and so would count every round twice.
pub fn state_from_rounds_with(max_samples: usize, rounds: &[&[Reply]]) -> State {
    let mut state = trippy_core::Builder::new("10.0.0.254".parse().expect("address"))
        .max_samples(max_samples)
        .build()
        .expect("tracer")
        .snapshot();
    for (round_index, replies) in rounds.iter().enumerate() {
        let sent = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000 + round_index as u64);
        let probes: Vec<ProbeStatus> = replies
            .iter()
            .enumerate()
            .map(|(i, reply)| probe(reply, i as u16, round_index, sent))
            .collect();
        let largest_ttl = replies.iter().map(|r| r.ttl).max().unwrap_or(0);
        let round = Round::new(
            &probes,
            TimeToLive(largest_ttl),
            CompletionReason::TargetFound,
        );
        state.update_from_round(&round);
    }
    state
}

/// A state with a single hop that received `samples` in arrival order.
pub fn state_with_samples(samples: &[Option<f64>]) -> State {
    let rounds: Vec<Vec<Reply>> = samples
        .iter()
        .map(|sample| vec![sample.map_or(lost(1), |ms| reply(1, "10.0.0.1", ms))])
        .collect();
    let rounds: Vec<&[Reply]> = rounds.iter().map(Vec::as_slice).collect();
    state_from_rounds(&rounds)
}

fn probe(reply: &Reply, sequence: u16, round: usize, sent: SystemTime) -> ProbeStatus {
    let addr: IpAddr = reply.addr.parse().expect("address");
    match reply.rtt_ms {
        Some(ms) => ProbeStatus::Complete(ProbeComplete {
            sequence: Sequence(33_000 + sequence),
            identifier: TraceId(1),
            src_port: Port(0),
            dest_port: Port(0),
            ttl: TimeToLive(reply.ttl),
            round: RoundId(round),
            sent,
            host: addr,
            received: sent + Duration::from_secs_f64(ms / 1000.0),
            icmp_packet_type: IcmpPacketType::NotApplicable,
            tos: None,
            expected_udp_checksum: None,
            actual_udp_checksum: None,
            extensions: None,
        }),
        None => ProbeStatus::Awaited(Probe {
            sequence: Sequence(33_000 + sequence),
            identifier: TraceId(1),
            src_port: Port(0),
            dest_port: Port(0),
            ttl: TimeToLive(reply.ttl),
            round: RoundId(round),
            sent,
            flags: Flags::empty(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_are_newest_first() {
        let state = state_with_samples(&[Some(1.0), Some(2.0), None, Some(4.0)]);
        let hop = &state.hops()[0];
        assert_eq!(hop.total_sent(), 4);
        assert_eq!(hop.total_recv(), 3);
        let ms: Vec<u128> = hop.samples().iter().map(|d| d.as_millis()).collect();
        assert_eq!(ms, [4, 0, 2, 1]);
    }

    #[test]
    fn rounds_build_a_path() {
        let state = state_from_rounds(&[&[
            reply(1, "10.0.0.1", 1.0),
            lost(2),
            reply(3, "10.0.0.3", 3.0),
        ]]);
        assert_eq!(state.hops().len(), 3);
        assert_eq!(state.hops()[1].total_recv(), 0);
        assert!(state.is_target(&state.hops()[2], trippy_core::FlowId(0)));
    }
}
