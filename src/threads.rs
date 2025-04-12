use colored::Colorize;
use std::num::NonZero;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

type WorkerReceiver = Arc<Mutex<mpsc::Receiver<Job>>>;
type ThreadControllerReceiver = crossbeam_channel::Receiver<usize>;
type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(
        id: usize,
        debug: bool,
        receiver: WorkerReceiver,
        controller: ThreadControllerReceiver,
    ) -> Worker {
        let thread = thread::spawn(move || loop {
            // If the controller says to stop, ignore remaining jobs on the receiver and just stop.
            if !controller.is_empty() {
                if debug {
                    println!("thread {}: {}", id, "Controller said to stop".red());
                }
                break;
            }

            // recv will block until the next job is sent.
            if debug {
                println!("thread {}: {}", id, "Waiting for job".green());
            }
            let message = receiver
                .lock()
                .expect("Worker thread could not get message from main thread")
                .recv();

            match message {
                Ok(job) => {
                    if debug {
                        println!("thread {}: {}", id, "Job received; running".green());
                    }
                    // Run the job inside this thread.
                    job();
                }
                // The thread will stop when the job channel is sent an Err, which will happen when
                // the channel is closed.
                Err(_) => {
                    if debug {
                        println!("thread {}: {}", id, "No more jobs; stopping".red());
                    }
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
    thread_control_tx: crossbeam_channel::Sender<usize>,
}

impl ThreadPool {
    pub fn new(count: NonZero<usize>, debug: bool) -> ThreadPool {
        // This channel is used to send Jobs to each thread.
        let (sender, receiver) = mpsc::channel();
        // This channel is used to send a stop signal to each thread.
        let (thread_control_tx, thread_control_rx) = crossbeam_channel::unbounded();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(count.into());

        for id in 0..count.into() {
            workers.push(Worker::new(
                id,
                debug,
                Arc::clone(&receiver),
                thread_control_rx.clone(),
            ));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
            thread_control_tx,
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

    pub fn stop(&mut self) {
        self.thread_control_tx
            .send(1)
            .expect("Unable to send stop message to threads");
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.wait_for_all_jobs_and_stop();
    }
}
