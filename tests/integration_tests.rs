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
    async fn test_smoketest() {
        let account_service = run_test_file("smoketest").await;
        // client,available,held,total,locked
        // 3,0,0,0,true
        // 2,2,0,2,false
        // 1,1.5,0,1.5,false

        assert_eq!(account_service.get_account(1).unwrap().total(), 1.5);
        assert_eq!(account_service.get_account(1).unwrap().available(), 1.5);
        assert_eq!(account_service.get_account(1).unwrap().held(), 0.0);
        assert_eq!(account_service.get_account(1).unwrap().locked(), false);

        assert_eq!(account_service.get_account(2).unwrap().total(), 2.0);
        assert_eq!(account_service.get_account(2).unwrap().available(), 2.0);
        assert_eq!(account_service.get_account(2).unwrap().held(), 0.0);
        assert_eq!(account_service.get_account(2).unwrap().locked(), false);

        assert_eq!(account_service.get_account(3).unwrap().total(), 0.0);
        assert_eq!(account_service.get_account(3).unwrap().available(), 0.0);
        assert_eq!(account_service.get_account(3).unwrap().held(), 0.0);
        assert_eq!(account_service.get_account(3).unwrap().locked(), true);

        assert_eq!(account_service.get_account(4).unwrap().total(), 20.0);
        assert_eq!(account_service.get_account(4).unwrap().available(), 20.0);
        assert_eq!(account_service.get_account(4).unwrap().held(), 0.0);
        assert_eq!(account_service.get_account(4).unwrap().locked(), false);
    }

    #[tokio::test]
    async fn test_cannot_withdraw_more_than_balance() {
        let account_service = run_test_file("withdraw_more_than_balance").await;
        assert_eq!(account_service.get_account(1).unwrap().total(), 1.0);
    }

    #[tokio::test]
    async fn test_cannot_withdraw_held_funds() {
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
