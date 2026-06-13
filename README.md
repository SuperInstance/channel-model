# Channel Model — Communication Channel Simulation

**Channel modeling** is the simulation of a communication link's behavior — bandwidth, latency, jitter, packet loss, and ordering — to predict application performance under realistic network conditions. This crate provides a parametric `ChannelModel` that accepts `ChannelProperties` and simulates message transmission outcomes.

## Why It Matters

Every distributed system depends on network links with non-ideal properties. A service that works perfectly on localhost may degrade catastrophically over a transcontinental link with 200ms latency, 1% packet loss, and 50ms jitter. Channel models let you answer "what happens to throughput if loss doubles?" without deploying to production. They're essential for capacity planning, SLA modeling, protocol design, and chaos engineering. Engineers who build RPC frameworks, CDN routing logic, or real-time streaming systems all need channel models to reason about worst-case behavior.

## How It Works

The model is governed by five parameters in `ChannelProperties`:

| Parameter | Unit | Description |
|---|---|---|
| `bandwidth_bps` | bits/sec | Maximum theoretical throughput |
| `latency` | `Duration` | One-way propagation delay |
| `jitter` | `Duration` | Variance in latency (±) |
| `loss_rate` | `0.0–1.0` | Probability of packet loss |
| `ordered` | `bool` | Whether delivery preserves send order |

**Loss simulation** uses a deterministic pseudo-random function: for message ID `m`, the loss check is `(m × 7.3) mod 100 / 100 < loss_rate`. This gives reproducible results without a PRNG dependency.

**Effective throughput** accounts for loss:

```
effective = bandwidth × (1 - loss_rate)
```

For a 10 Mbps link with 1% loss: `effective = 10,000,000 × 0.99 = 9.9 Mbps`. This is an upper bound — it ignores retransmission overhead, head-of-line blocking, and congestion control backoff, all of which further reduce real throughput.

**Latency budget** for a message is:

```
T_total = T_propagation + T_transmission + T_queuing + T_jitter
T_transmission = payload_size × 8 / bandwidth
```

The Shannon-Hartley theorem gives the theoretical maximum capacity: `C = B log₂(1 + S/N)`, which sets the physical ceiling that `bandwidth_bps` approaches but cannot exceed.

## Quick Start

```rust
use channel_model::{ChannelModel, ChannelProperties, TransmitResult};
use std::time::Duration;

let props = ChannelProperties {
    bandwidth_bps: 1_000_000,      // 1 Mbps
    latency: Duration::from_millis(50),
    jitter: Duration::from_millis(5),
    loss_rate: 0.0,                // no loss
    ordered: true,
};

let mut channel = ChannelModel::new(props);
let result = channel.send(vec![0xDE, 0xAD, 0xBE, 0xEF]);

match result {
    TransmitResult::Delivered { latency } => {
        println!("Delivered in {:?}", latency);
    }
    TransmitResult::Lost => println!("Packet lost"),
    TransmitResult::Reordered { position_offset } => {
        println!("Reordered by {position_offset} positions");
    }
}

// Effective throughput accounting for loss
println!("Effective throughput: {:.0} bps", channel.effective_throughput());
```

## API

| Type | Description |
|---|---|
| `ChannelProperties` | Configuration: bandwidth, latency, jitter, loss_rate, ordered. |
| `ChannelModel` | Simulator instance. Create with `new(props)`, send with `send(payload)`. |
| `Message` | Internal representation: `id`, `payload`, `sent_at`, `received_at`. |
| `TransmitResult` | Outcome enum: `Delivered { latency }`, `Lost`, `Reordered { position_offset }`. |
| `ChannelModel::effective_throughput()` | Returns `bandwidth × (1 - loss_rate)` in bytes/sec. |

## Architecture Notes

Channel models serve the η (evaluation) side of γ + η = C in SuperInstance. They simulate the network conditions that fleet instances experience, enabling capacity planning and protocol tuning without production experiments. See [SuperInstance Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

1. Shannon, C. E. (1948). *A Mathematical Theory of Communication*. Bell System Technical Journal 27, 379–423. — Capacity theorem.
2. Tanenbaum, A. S. & Wetherall, D. J. (2011). *Computer Networks* (5th ed.), Ch. 3. Pearson.
3. Begen, A., Akgul, T., & Baugher, M. (2010). *Watching Video over the Web*. IEEE Internet Computing 14(3), 54–63.

## License

MIT
