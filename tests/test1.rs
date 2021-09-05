use async_io::Timer;
use async_runtime::executor;
use std::time::Duration;

async fn my_timer() {
    Timer::after(Duration::from_millis(2000)).await;
}
#[test]
fn test() {
    let (exec, enq) = executor::Executor::new();
    {
        let q = enq; //move enq for drop at  the end
        q.spawn(async {
            println!("hi...");
            my_timer().await;
            println!("bye");
        });
    }
    exec.run();
}
