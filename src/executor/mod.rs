/// a custom executor
use {
    crossbeam_channel,
    //provides extra utilies when working with async
    futures::task::{waker_ref, ArcWake},
    std::{
        future::Future,
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll},
        // time::Duration,
    },
};

type BoxFut = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub struct Executor {
    rx: crossbeam_channel::Receiver<Arc<Task>>,
}

/// package future with a channel to enqueue into an executor
struct Task {
    fut: Mutex<Option<BoxFut>>,
    tx: crossbeam_channel::Sender<Arc<Task>>,
}

impl Drop for Task {
    fn drop(&mut self) {
        println!("task dropped");
    }
}
/// allow easy conversion to Waker
impl ArcWake for Task {
    fn wake_by_ref(arc: &Arc<Self>) {
        //wake by enqueuing the task to executor
        arc.tx.send(arc.clone()).expect("wake task enqueue failed");
    }
}

impl Executor {
    pub fn new() -> (Executor, Enqueue) {
        //use channels to queue tasks
        let (sender, receiver): (
            crossbeam_channel::Sender<Arc<Task>>,
            crossbeam_channel::Receiver<Arc<Task>>,
        ) = crossbeam_channel::unbounded();
        (Executor { rx: receiver }, Enqueue(sender))
    }
    pub fn run(&self) {
        while let Ok(t) = self.rx.recv() {
            let mut guard = t.fut.lock().unwrap();
            println!("received a task");
            if let Some(mut boxed_future) = guard.take() {
                let pinned_future = boxed_future.as_mut();

                //convert ArcWake to WakerRef and Waker and get a Context
                let w = waker_ref(&t);
                let mut ctx = Context::from_waker(&*w);

                match pinned_future.poll(&mut ctx) {
                    Poll::Pending => *guard = Some(boxed_future),
                    _ => {}
                }
                // {
                //     let count = Arc::strong_count(&t);
                //     println!("arc count: {}", count);
                // }
            }
        }
    }
}

#[derive(Clone)]
pub struct Enqueue(crossbeam_channel::Sender<Arc<Task>>);

impl Enqueue {
    /// execute future
    pub fn spawn(&self, f: impl Future<Output = ()> + Send + 'static) {
        //pin and queue it for executor
        let t = Task {
            fut: Mutex::new(Some(Pin::from(Box::new(f)))),
            tx: self.0.clone(),
        };
        self.0.send(Arc::new(t)).expect("task enqueue failed");
    }
}
