#[cfg(test)]
mod tests {
    use payments_engine::engine::payments::AccountService;
    use payments_engine::payments_engine;

    async fn run_test_file(name: &str) -> AccountService {
        let csv_file = format!("file://tests/resources/{}.csv", name);
        let (ingestion_service, account_service) = payments_engine();
        ingestion_service.submit_payments_csv(&csv_file).await.unwrap();
        ingestion_service.run().await;
        ingestion_service.shutdown_gracefully().await;
        account_service
    }

    #[tokio::test]
    async fn test_cannot_with_draw_more_than_balance() {
        let account_service = run_test_file("withdraw_more_than_balance").await;
        assert_eq!(account_service.get_account(1).unwrap().total(), 1.0);
    }

    #[tokio::test]
    async fn test_precision() {
        let account_service = run_test_file("precision").await;
        assert_eq!(account_service.get_account(1).unwrap().total(), 0.123);
        assert_eq!(account_service.get_account(2).unwrap().total(), 0.1234);
        assert_eq!(account_service.get_account(3).unwrap().total(), 0.1234);
        assert_eq!(account_service.get_account(4).unwrap().total(), 0.1235);

    }
}
