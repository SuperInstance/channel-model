//! channel-model
//! Models for abstract communication channels — capacity, latency, loss, and ordering semantics.

use std::time::Duration;

/// Describes the properties of a communication channel.
#[derive(Debug, Clone)]
pub struct ChannelProperties {
    pub bandwidth_bps: u64,
    pub latency: Duration,
    pub jitter: Duration,
    pub loss_rate: f64,
    pub ordered: bool,
}

/// A message traveling through a channel.
#[derive(Debug, Clone)]
pub struct Message {
    pub id: u64,
    pub payload: Vec<u8>,
    pub sent_at: Option<Duration>,
    pub received_at: Option<Duration>,
}

/// Simulates a simple channel with configurable properties.
pub struct ChannelModel {
    props: ChannelProperties,
    messages: Vec<Message>,
    next_id: u64,
}

/// Result of a transmission attempt.
#[derive(Debug)]
pub enum TransmitResult {
    Delivered { latency: Duration },
    Lost,
    Reordered { position_offset: i32 },
}

impl ChannelModel {
    pub fn new(props: ChannelProperties) -> Self {
        Self {
            props,
            messages: Vec::new(),
            next_id: 1,
        }
    }

    /// Send a payload through the channel.
    pub fn send(&mut self, payload: Vec<u8>) -> TransmitResult {
        let msg = Message {
            id: self.next_id,
            payload,
            sent_at: None,
            received_at: None,
        };
        self.next_id += 1;

        // Simple loss simulation based on loss rate
        if self.props.loss_rate > 0.0 && (msg.id as f64 * 7.3) % 100.0 / 100.0 < self.props.loss_rate {
            self.messages.push(msg);
            return TransmitResult::Lost;
        }

        self.messages.push(msg);
        TransmitResult::Delivered {
            latency: self.props.latency,
        }
    }

    /// Get the number of messages that passed through.
    pub fn delivered_count(&self) -> usize {
        self.messages.len()
    }

    /// Effective throughput in bytes/sec based on bandwidth.
    pub fn effective_throughput(&self) -> f64 {
        self.props.bandwidth_bps as f64 * (1.0 - self.props.loss_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_message() {
        let model = ChannelModel::new(ChannelProperties {
            bandwidth_bps: 1_000_000,
            latency: Duration::from_millis(50),
            jitter: Duration::from_millis(5),
            loss_rate: 0.0,
            ordered: true,
        });
        let mut m = model;
        let result = m.send(vec![1, 2, 3]);
        assert!(matches!(result, TransmitResult::Delivered { .. }));
        assert_eq!(m.delivered_count(), 1);
    }
}

/// FNV-1a 64 — the digest every substrate in the SuperInstance fleet agrees on.
pub const FNV_OFFSET: u64 = 0xcbf29ce484222325;
pub const FNV_PRIME: u64 = 0x100000001b3;

#[inline]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h = (h ^ b as u64).wrapping_mul(FNV_PRIME);
    }
    h
}

/// True if this crate's FNV-1a still agrees with the rest of the fleet.
pub fn canary_holds() -> bool {
    fnv1a64("café Δ 日本語".as_bytes()) == 0x024a555471370b18d
}
