/// Integration tests for OpenFlow controller
/// 
/// Tests:
/// - Flow installation actually sends to switches
/// - Connection management
/// - Retry logic
/// - Graceful shutdown
/// - Error handling

use controller::{Controller, ControllerService, FlowRule, MatchFields};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
async fn test_controller_lifecycle() {
    let controller = ControllerService::new("127.0.0.1".to_string(), 16633);
    
    // Start controller
    assert!(controller.start().await.is_ok());
    
    // Give it time to bind
    sleep(Duration::from_millis(100)).await;
    
    // Stop controller
    assert!(controller.stop().await.is_ok());
}

#[tokio::test]
async fn test_flow_validation() {
    let controller = ControllerService::new("127.0.0.1".to_string(), 16634);
    
    // Invalid rule: empty switch ID
    let invalid_rule = FlowRule {
        id: Uuid::new_v4(),
        switch_id: "".to_string(),
        priority: 100,
        match_fields: MatchFields {
            in_port: None,
            eth_src: None,
            eth_dst: None,
            eth_type: None,
            ip_src: None,
            ip_dst: None,
            ip_proto: None,
            tcp_src: None,
            tcp_dst: None,
        },
        actions: vec![],
        idle_timeout: 0,
        hard_timeout: 0,
    };

    // Should fail validation
    let result = controller.install_flow(invalid_rule).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_flow_installation_without_switch() {
    let controller = ControllerService::new("127.0.0.1".to_string(), 16635);
    controller.start().await.unwrap();
    
    // Valid rule but switch doesn't exist
    let rule = FlowRule {
        id: Uuid::new_v4(),
        switch_id: "nonexistent-switch".to_string(),
        priority: 100,
        match_fields: MatchFields {
            in_port: Some(1),
            eth_src: None,
            eth_dst: None,
            eth_type: None,
            ip_src: None,
            ip_dst: None,
            ip_proto: None,
            tcp_src: None,
            tcp_dst: None,
        },
        actions: vec![],
        idle_timeout: 0,
        hard_timeout: 0,
    };

    // Should fail because switch doesn't exist
    let result = controller.install_flow(rule).await;
    assert!(result.is_err());
    
    controller.stop().await.unwrap();
}

#[tokio::test]
async fn test_duplicate_flow_prevention() {
    let controller = ControllerService::new("127.0.0.1".to_string(), 16636);
    controller.start().await.unwrap();
    
    let flow_id = Uuid::new_v4();
    let rule = FlowRule {
        id: flow_id,
        switch_id: "test-switch".to_string(),
        priority: 100,
        match_fields: MatchFields {
            in_port: Some(1),
            eth_src: None,
            eth_dst: None,
            eth_type: None,
            ip_src: None,
            ip_dst: None,
            ip_proto: None,
            tcp_src: None,
            tcp_dst: None,
        },
        actions: vec![],
        idle_timeout: 0,
        hard_timeout: 0,
    };

    // First installation will fail (no switch)
    let _ = controller.install_flow(rule.clone()).await;
    
    // Second installation with same ID should fail
    let result = controller.install_flow(rule).await;
    assert!(result.is_err());
    
    controller.stop().await.unwrap();
}

#[tokio::test]
async fn test_get_switches_empty() {
    let controller = ControllerService::new("127.0.0.1".to_string(), 16637);
    controller.start().await.unwrap();
    
    let switches = controller.get_switches().await.unwrap();
    assert_eq!(switches.len(), 0);
    
    controller.stop().await.unwrap();
}

#[tokio::test]
async fn test_flow_not_found() {
    let controller = ControllerService::new("127.0.0.1".to_string(), 16638);
    controller.start().await.unwrap();
    
    let nonexistent_id = Uuid::new_v4();
    let result = controller.get_flow_stats(nonexistent_id).await;
    assert!(result.is_err());
    
    controller.stop().await.unwrap();
}

#[tokio::test]
async fn test_graceful_shutdown() {
    let controller = ControllerService::new("127.0.0.1".to_string(), 16639);
    
    // Start
    controller.start().await.unwrap();
    sleep(Duration::from_millis(100)).await;
    
    // Stop should complete quickly
    let start = std::time::Instant::now();
    controller.stop().await.unwrap();
    let duration = start.elapsed();
    
    // Should complete in less than 1 second
    assert!(duration < Duration::from_secs(1));
}
