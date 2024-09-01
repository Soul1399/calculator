use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, thread, time::Duration, error::Error, fmt::{Display, Formatter}, sync::{mpsc::{self, Receiver}, Mutex, Arc}, env::consts::OS,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let handle = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker { id, handle: Some(handle) }
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size < 1 {
            return Err(PoolCreationError { details: String::from(format!("Invalid thead pool size: {size}")) });
        }
        if size > 9 {
            return Err(PoolCreationError { details: String::from(format!("Thead pool size was too large: {size}")) });
        }

        let (sender, receiver) = mpsc::channel();
        let mut workers = Vec::with_capacity(size);
        let receiver = Arc::new(Mutex::new(receiver));

        for c in 0..size {
            // create some workers and store them in the vector
            workers.push(Worker::new(c, Arc::clone(&receiver)));
        }

        Ok(ThreadPool { workers, sender: Some(sender) })
    }

    pub fn execute<F>(&self, f: F) where F: FnOnce() + Send + 'static {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.handle.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[derive(Debug)]
pub struct PoolCreationError {
    details: String
}

impl Error for PoolCreationError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl Display for PoolCreationError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f,"{}", self.details)
    }
}

pub fn start_listening() -> Result<usize, Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::build(4)?;

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    Ok(Default::default())
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "./src/webserver/hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "./src/webserver/hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "./src/webserver/404.html"),
    };
    
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}