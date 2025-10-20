// 統合テスト
// Note: WASMターゲットではsecp256k1のビルドに問題があるため、
// これらのテストはネイティブターゲットでのみ実行されます。

// クレート名が`core`なので、明示的にエイリアスを使用
extern crate core as rustr_core;

#[cfg(test)]
mod tests {
    use super::rustr_core;
    use rustr_core::storage::mock::MockStorage;

    #[test]
    fn test_storage_mock() {
        let _storage = MockStorage::new();
        
        // 初期状態では空 (非同期処理は実際のWASM環境でテスト)
        // 非同期テストはWASM環境でのみ実行可能
    }

    #[test]
    fn test_relay_message_parse() {
        use rustr_core::relay::RelayMessage;
        
        let json = r#"["EVENT","sub1",{"id":"abc","kind":1}]"#;
        let msg = RelayMessage::parse(json).unwrap();
        match msg {
            RelayMessage::Event { sub_id, .. } => assert_eq!(sub_id, "sub1"),
            _ => panic!("Expected EVENT message"),
        }
    }
    
    #[test]
    fn test_exponential_backoff() {
        use rustr_core::relay::ExponentialBackoff;
        
        let mut backoff = ExponentialBackoff::new();
        assert_eq!(backoff.next_delay(), 1);
        assert_eq!(backoff.next_delay(), 2);
        assert_eq!(backoff.next_delay(), 4);
        
        backoff.reset();
        assert_eq!(backoff.next_delay(), 1);
    }
}

