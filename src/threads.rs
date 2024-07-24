use std::num::NonZero;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

type WorkerReceiver = Arc<Mutex<mpsc::Receiver<Job>>>;
type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: WorkerReceiver) -> Worker {
        let thread = thread::spawn(move || loop {
            // recv will block until the next job is sent.
            let message = receiver
                .lock()
                .expect("Worker thread could not get message from main thread")
                .recv();

            match message {
                Ok(job) => {
                    job();
                }
                // The thread will stop when the job channel is sent an Err, which will happen when
                // the channel is closed.
                Err(_) => {
                    break;
                }
            }
        });
        Worker {
            _id: id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(count: NonZero<usize>) -> ThreadPool {
        // This channel is used to send Jobs to each thread.
        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(count.into());

        for id in 0..count.into() {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .as_ref()
            .expect("Executing search thread failed")
            .send(job)
            .expect("Unable to send data to search thread");
    }

    pub fn wait_for_all_jobs_and_stop(&mut self) {
        // Close the Jobs channel which will trigger each thread to stop when it finishes its
        // current work.
        drop(self.sender.take());

        for worker in &mut self.workers {
            // Collect each thread which all should have stopped working by now.
            if let Some(thread) = worker.thread.take() {
                thread.join().expect("Unable to close thread");
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.wait_for_all_jobs_and_stop();
    }
}
