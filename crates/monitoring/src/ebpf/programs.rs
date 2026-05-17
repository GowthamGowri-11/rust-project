/// eBPF Program Definitions
/// 
/// This module contains the eBPF program source code that will be compiled
/// and loaded into the kernel for packet monitoring.

pub const PACKET_MONITOR_PROGRAM: &str = r#"
#include <uapi/linux/ptrace.h>
#include <net/sock.h>
#include <bcc/proto.h>

// Event structure for packet data
struct packet_event {
    u64 timestamp;
    u32 src_ip;
    u32 dst_ip;
    u16 src_port;
    u16 dst_port;
    u8 protocol;
    u16 packet_len;
    u64 bytes;
};

// Ring buffer for events
BPF_RINGBUF_OUTPUT(events, 256);

// Per-flow statistics
BPF_HASH(flow_stats, u64, u64);

// Packet counter
BPF_ARRAY(packet_count, u64, 1);

// Bandwidth tracker
BPF_HASH(bandwidth, u32, u64);

int trace_packet(struct __sk_buff *skb) {
    void *data_end = (void *)(long)skb->data_end;
    void *data = (void *)(long)skb->data;
    
    struct ethhdr *eth = data;
    if ((void *)(eth + 1) > data_end)
        return 0;
    
    // Only process IP packets
    if (eth->h_proto != htons(ETH_P_IP))
        return 0;
    
    struct iphdr *ip = (void *)(eth + 1);
    if ((void *)(ip + 1) > data_end)
        return 0;
    
    // Create event
    struct packet_event event = {};
    event.timestamp = bpf_ktime_get_ns();
    event.src_ip = ip->saddr;
    event.dst_ip = ip->daddr;
    event.protocol = ip->protocol;
    event.packet_len = skb->len;
    event.bytes = skb->len;
    
    // Extract port information for TCP/UDP
    if (ip->protocol == IPPROTO_TCP || ip->protocol == IPPROTO_UDP) {
        struct tcphdr *tcp = (void *)(ip + 1);
        if ((void *)(tcp + 1) <= data_end) {
            event.src_port = tcp->source;
            event.dst_port = tcp->dest;
        }
    }
    
    // Send event to userspace
    events.ringbuf_output(&event, sizeof(event), 0);
    
    // Update statistics
    u64 flow_key = ((u64)ip->saddr << 32) | ip->daddr;
    u64 *flow_bytes = flow_stats.lookup_or_try_init(&flow_key, &event.bytes);
    if (flow_bytes)
        __sync_fetch_and_add(flow_bytes, event.bytes);
    
    // Update bandwidth
    u32 interface = skb->ifindex;
    u64 *bw = bandwidth.lookup_or_try_init(&interface, &event.bytes);
    if (bw)
        __sync_fetch_and_add(bw, event.bytes);
    
    // Increment packet counter
    u32 idx = 0;
    u64 *count = packet_count.lookup_or_try_init(&idx, &event.bytes);
    if (count)
        __sync_fetch_and_add(count, 1);
    
    return 0;
}

// Tracepoint for TCP connection tracking
TRACEPOINT_PROBE(tcp, tcp_connect) {
    struct packet_event event = {};
    event.timestamp = bpf_ktime_get_ns();
    event.protocol = IPPROTO_TCP;
    
    events.ringbuf_output(&event, sizeof(event), 0);
    return 0;
}

// Tracepoint for packet loss detection
TRACEPOINT_PROBE(skb, skb_drop_tcp) {
    struct packet_event event = {};
    event.timestamp = bpf_ktime_get_ns();
    event.protocol = IPPROTO_TCP;
    
    events.ringbuf_output(&event, sizeof(event), 0);
    return 0;
}
"#;

pub const LATENCY_MONITOR_PROGRAM: &str = r#"
#include <uapi/linux/ptrace.h>
#include <net/sock.h>

// Latency tracking structure
struct latency_event {
    u64 timestamp;
    u64 latency_ns;
    u32 src_ip;
    u32 dst_ip;
};

BPF_RINGBUF_OUTPUT(latency_events, 256);
BPF_HASH(packet_timestamps, u64, u64);

// Track packet send time
int trace_send(struct __sk_buff *skb) {
    u64 ts = bpf_ktime_get_ns();
    u64 flow_key = ((u64)skb->saddr << 32) | skb->daddr;
    packet_timestamps.update(&flow_key, &ts);
    return 0;
}

// Calculate latency on receive
int trace_receive(struct __sk_buff *skb) {
    u64 now = bpf_ktime_get_ns();
    u64 flow_key = ((u64)skb->saddr << 32) | skb->daddr;
    
    u64 *send_time = packet_timestamps.lookup(&flow_key);
    if (send_time) {
        struct latency_event event = {};
        event.timestamp = now;
        event.latency_ns = now - *send_time;
        event.src_ip = skb->saddr;
        event.dst_ip = skb->daddr;
        
        latency_events.ringbuf_output(&event, sizeof(event), 0);
        packet_timestamps.delete(&flow_key);
    }
    
    return 0;
}
"#;

/// Get the appropriate eBPF program for the given probe type
pub fn get_program(probe_type: &str) -> Option<&'static str> {
    match probe_type {
        "packet_monitor" => Some(PACKET_MONITOR_PROGRAM),
        "latency_monitor" => Some(LATENCY_MONITOR_PROGRAM),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_availability() {
        assert!(get_program("packet_monitor").is_some());
        assert!(get_program("latency_monitor").is_some());
        assert!(get_program("unknown").is_none());
    }

    #[test]
    fn test_program_not_empty() {
        assert!(!PACKET_MONITOR_PROGRAM.is_empty());
        assert!(!LATENCY_MONITOR_PROGRAM.is_empty());
    }
}
