#!/usr/bin/env python3
"""
Multi-path topology for RustFlow-AI
Creates a network with multiple paths between hosts for load balancing testing
"""

from mininet.topo import Topo
from mininet.net import Mininet
from mininet.node import RemoteController
from mininet.cli import CLI
from mininet.log import setLogLevel, info
from mininet.link import TCLink

class MultiPathTopo(Topo):
    """
    Multi-path topology with 6 switches and 4 hosts
    
    Topology:
           h1 --- s1 --- s2 --- s5 --- h3
                   |  X  |      |
                   | / \ |      |
                   s3 - s4 --- s6 --- h4
                   |
                  h2
    """
    
    def build(self):
        # Add hosts
        h1 = self.addHost('h1', ip='10.0.0.1/24', mac='00:00:00:00:00:01')
        h2 = self.addHost('h2', ip='10.0.0.2/24', mac='00:00:00:00:00:02')
        h3 = self.addHost('h3', ip='10.0.0.3/24', mac='00:00:00:00:00:03')
        h4 = self.addHost('h4', ip='10.0.0.4/24', mac='00:00:00:00:00:04')
        
        # Add switches
        s1 = self.addSwitch('s1', dpid='0000000000000001')
        s2 = self.addSwitch('s2', dpid='0000000000000002')
        s3 = self.addSwitch('s3', dpid='0000000000000003')
        s4 = self.addSwitch('s4', dpid='0000000000000004')
        s5 = self.addSwitch('s5', dpid='0000000000000005')
        s6 = self.addSwitch('s6', dpid='0000000000000006')
        
        # Add links with bandwidth and delay constraints
        # Host to switch links
        self.addLink(h1, s1, bw=100, delay='5ms')
        self.addLink(h2, s3, bw=100, delay='5ms')
        self.addLink(h3, s5, bw=100, delay='5ms')
        self.addLink(h4, s6, bw=100, delay='5ms')
        
        # Core switch links - multiple paths
        self.addLink(s1, s2, bw=50, delay='10ms')
        self.addLink(s1, s3, bw=50, delay='10ms')
        self.addLink(s2, s4, bw=50, delay='10ms')
        self.addLink(s3, s4, bw=50, delay='10ms')
        self.addLink(s2, s5, bw=50, delay='10ms')
        self.addLink(s4, s6, bw=50, delay='10ms')
        self.addLink(s5, s6, bw=50, delay='10ms')

def run_topology():
    """Start the topology with remote controller"""
    setLogLevel('info')
    
    topo = MultiPathTopo()
    
    # Connect to RustFlow-AI controller
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
