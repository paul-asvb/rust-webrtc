use std::time::Duration;

use tokio::time::sleep;

use bk::session::start_session;

#[tokio::test]
async fn test_session() {
    let res = start_session().await;
    sleep(Duration::from_millis(1000)).await;
    if res.is_err() {
        println!("error: {:?}", res.err())
    } else {
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}
