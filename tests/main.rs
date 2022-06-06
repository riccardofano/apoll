mod helpers;

use crate::helpers::TestApp;

#[tokio::test]
async fn database_gets_dropped_after_running_tests() {
    let app = TestApp::new().await;
    println!("app database = {}", &app.db_name);
}
