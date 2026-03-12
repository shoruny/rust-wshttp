use std::{
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    thread,
};
pub mod config;
pub mod http;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);
            match worker.thread.join() {
                Ok(value) => value,
                _ => {}
            }
        }
    }
}
type Job = Box<dyn FnOnce() + Send + 'static>;
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // let thread = thread::spawn(move || {
        //     while let Ok(job) = receiver.lock().unwrap().recv() {
        //         println!("Worker {id} got a job; executing.");

        //         job();
        //     }
        // });
        let thread = thread::spawn(move || {
            loop {
                let job = {
                    // v1
                    // let result = receiver.lock().unwrap().recv();
                    // match result {
                    //     Ok(job) => job,
                    //     Err(_) => break, // 通道关了，跳出循环
                    // }
                    // v2 不用unwrap
                    // let Ok(lock) = receiver.lock() else { break };
                    // let Ok(job) = lock.recv() else {
                    //     break;
                    // };
                    // job
                    //
                    // v3
                    let mutex_guard = match receiver.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => {
                            // 重点：即便中毒了，我也强行拿回里面的 Receiver
                            // 这能防止 Worker 线程直接 break 退出
                            println!("警告：检测到锁污染，正在尝试恢复...");
                            poisoned.into_inner()
                        }
                    };
                    let Ok(job) = mutex_guard.recv() else {
                        break;
                    };
                    job
                };
                // <--- 锁在这里准时释放！let job = {
                //     let lock = receiver.lock();
                //     match lock {
                //         Ok(l) => {
                //             let result = l.recv();
                //             match result {
                //                 Ok(job) => job,
                //                 Err(_) => break, // 通道关了，跳出循环
                //             }
                //         }
                //         _ => {
                //             continue;
                //         }
                //     }
                // }; // <--- 锁在这里准时释放！
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    job();
                }));
                if let Err(_) = result {
                    println!("虽然 Job 崩了，但我的 Worker 线程还活着！");
                }
                println!("Worker {id} got a job; executing.");
                // job(); // <--- 执行 job 时，锁已经是释放状态了
            }
        });

        Worker { id, thread }
    }
}
impl ThreadPool {
    pub fn build(size: usize) -> Result<ThreadPool, u32> {
        if size <= 0 {
            return Err(1);
        }
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}
