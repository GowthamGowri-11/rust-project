#!/usr/bin/env python3
"""
Congestion-prone topology for testing traffic engineering
Creates bottleneck links to simulate congestion scenarios
"""

from mininet.topo import Topo
from mininet.net import Mininet
from mininet.node import RemoteController
from mininet.cli import CLI
from mininet.log import setLogLevel, info
from mininet.link import TCLink

class CongestionTopo(Topo):
    """
    Topology with intentional bottlenecks
    
    Topology:
        h1 ---\           /--- h5
        h2 --- s1 - s3 - s5 --- h6
        h3 ---/     |     \--- h7
        h4 -------- s4 -------- h8
        
    s1-s3 and s3-s5 are bottleneck links (10Mbps)
    """
    
    def build(self):
        # Add hosts
        hosts = []
        for i in range(1, 9):
            h = self.addHost(f'h{i}', 
                           ip=f'10.0.0.{i}/24',
                           mac=f'00:00:00:00:00:0{i}')
            hosts.append(h)
        
        # Add switches
        s1 = self.addSwitch('s1', dpid='0000000000000001')
        s3 = self.addSwitch('s3', dpid='0000000000000003')
        s4 = self.addSwitch('s4', dpid='0000000000000004')
        s5 = self.addSwitch('s5', dpid='0000000000000005')
        
        # Host to edge switch links (high bandwidth)
        self.addLink(hosts[0], s1, bw=100, delay='2ms')
        self.addLink(hosts[1], s1, bw=100, delay='2ms')
        self.addLink(hosts[2], s1, bw=100, delay='2ms')
        self.addLink(hosts[3], s4, bw=100, delay='2ms')
        self.addLink(hosts[4], s5, bw=100, delay='2ms')
        self.addLink(hosts[5], s5, bw=100, delay='2ms')
        self.addLink(hosts[6], s5, bw=100, delay='2ms')
        self.addLink(hosts[7], s4, bw=100, delay='2ms')
        
        # Core links - BOTTLENECK
        self.addLink(s1, s3, bw=10, delay='20ms', loss=0)  # Bottleneck
        self.addLink(s3, s5, bw=10, delay='20ms', loss=0)  # Bottleneck
        
        # Alternative path
        self.addLink(s1, s4, bw=50, delay='15ms')
        self.addLink(s4, s5, bw=50, delay='15ms')
        self.addLink(s3, s4, bw=30, delay='10ms')

def run_topology():
    """Start the topology with remote controller"""
    setLogLevel('info')
    
    topo = CongestionTopo()
    
    net = Mininet(
        topo=topo,
        controller=lambda name: RemoteController(
            name, ip='127.0.0.1', port=6653
        ),
        link=TCLink,
        autoSetMacs=True
    )
    
    info('*** Starting network\n')
    net.start()
    
    info('*** Testing connectivity\n')
    net.pingAll()
    
    info('*** Running CLI\n')
    CLI(net)
    
    info('*** Stopping network\n')
    net.stop()

if __name__ == '__main__':
    run_topology()
