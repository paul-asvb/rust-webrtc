use crate::session::start_session;

#[tokio::test]
async fn test_session() {
    let res = start_session().await;
    assert!(res.is_ok());
}
