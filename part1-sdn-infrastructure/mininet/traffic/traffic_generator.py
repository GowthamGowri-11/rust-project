#!/usr/bin/env python3
"""
Traffic generation scripts for RustFlow-AI testing
Generates various traffic patterns: video, bulk download, mixed
"""

import argparse
import time
from mininet.net import Mininet
from mininet.log import info, error

class TrafficGenerator:
    def __init__(self, net):
        self.net = net
    
    def generate_video_traffic(self, src_host, dst_host, duration=60, rate='5M'):
        """
        Simulate video streaming traffic (UDP)
        Args:
            src_host: Source host name
            dst_host: Destination host name
            duration: Duration in seconds
            rate: Bandwidth rate (e.g., '5M' for 5 Mbps)
        """
        info(f'*** Generating video traffic: {src_host} -> {dst_host}\n')
        
        src = self.net.get(src_host)
        dst = self.net.get(dst_host)
        
        if not src or not dst:
            error(f'Host not found: {src_host} or {dst_host}\n')
            return
        
        dst_ip = dst.IP()
        
        # Start iperf server on destination (UDP)
        dst.cmd(f'iperf -s -u -p 5001 > /tmp/{dst_host}_video.log 2>&1 &')
        time.sleep(1)
        
        # Start iperf client on source
        src.cmd(f'iperf -c {dst_ip} -u -b {rate} -t {duration} -p 5001 > /tmp/{src_host}_video.log 2>&1 &')
        
        info(f'Video traffic started: {rate} for {duration}s\n')
    
    def generate_bulk_download(self, src_host, dst_host, duration=60):
        """
        Simulate bulk download traffic (TCP)
        Args:
            src_host: Source host name
            dst_host: Destination host name
            duration: Duration in seconds
        """
        info(f'*** Generating bulk download: {src_host} -> {dst_host}\n')
        
        src = self.net.get(src_host)
        dst = self.net.get(dst_host)
        
        if not src or not dst:
            error(f'Host not found: {src_host} or {dst_host}\n')
            return
        
        dst_ip = dst.IP()
        
        # Start iperf server on destination (TCP)
        dst.cmd(f'iperf -s -p 5002 > /tmp/{dst_host}_bulk.log 2>&1 &')
        time.sleep(1)
        
        # Start iperf client on source (TCP, maximum throughput)
        src.cmd(f'iperf -c {dst_ip} -t {duration} -p 5002 > /tmp/{src_host}_bulk.log 2>&1 &')
        
        info(f'Bulk download started for {duration}s\n')
    
    def generate_mixed_traffic(self, duration=60):
        """
        Generate mixed traffic patterns across multiple host pairs
        """
        info('*** Generating mixed traffic patterns\n')
        
        hosts = self.net.hosts
        if len(hosts) < 4:
            error('Need at least 4 hosts for mixed traffic\n')
            return
        
        # Video traffic: h1 -> h3
        self.generate_video_traffic('h1', 'h3', duration, '5M')
        time.sleep(2)
        
        # Bulk download: h2 -> h4
        self.generate_bulk_download('h2', 'h4', duration)
        time.sleep(2)
        
        # Additional video: h3 -> h1
        if len(hosts) >= 4:
            self.generate_video_traffic('h3', 'h1', duration, '3M')
        
        info('Mixed traffic generation complete\n')
    
    def stop_all_traffic(self):
        """Stop all iperf processes"""
        info('*** Stopping all traffic\n')
        for host in self.net.hosts:
            host.cmd('killall iperf')

def main():
    parser = argparse.ArgumentParser(description='Traffic Generator for RustFlow-AI')
    parser.add_argument('--type', choices=['video', 'bulk', 'mixed'], 
                       default='mixed', help='Traffic type')
    parser.add_argument('--duration', type=int, default=60, 
                       help='Duration in seconds')
    parser.add_argument('--src', help='Source host')
    parser.add_argument('--dst', help='Destination host')
    
    args = parser.parse_args()
    
    info('Traffic generator ready. Use with Mininet CLI:\n')
    info('  py traffic_gen.generate_video_traffic("h1", "h3", 60, "5M")\n')
    info('  py traffic_gen.generate_bulk_download("h2", "h4", 60)\n')
    info('  py traffic_gen.generate_mixed_traffic(60)\n')

if __name__ == '__main__':
    main()
