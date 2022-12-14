use crate::client::{Client, ClientState};
use crate::DynResult;
use std::io;
use std::net::{Shutdown, TcpListener, ToSocketAddrs};
use std::panic;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Copy, Clone)]
pub struct Work {
    pub begin: u64,
    pub end: u64,
    pub workers: u64,
}

pub struct Job {
    pub id: u64,
    pub width: u64,
    pub height: u64,
    pub cam_pos: [f32; 3],
    pub cam_rot: [f32; 3],
    pub cam_fov: f32,
    pub cam_focus: f32,
    pub cam_blur: f32,
    pub scene: String,
    pub key: String,
    pub render_mode: u64,
    pub batch_size: u64,
    pub trace_batch_size: u64,
    pub work: Vec<Work>,
}

pub struct Server {
    pub listener: TcpListener,
    pub clients: RwLock<Vec<Arc<Client>>>,
    pub job_seq: AtomicU64,
    pub jobs: RwLock<Vec<Job>>,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(addr: A) -> DynResult<Arc<Server>> {
        let listener = TcpListener::bind(addr)?;

        Ok(Arc::new(Server {
            listener,
            clients: RwLock::new(Vec::new()),
            job_seq: AtomicU64::new(0),
            jobs: RwLock::new(Vec::new()),
        }))
    }

    pub fn run(self: Arc<Self>) -> DynResult<()> {
        let server = self.clone();

        let watcher: JoinHandle<Result<(), io::Error>> = thread::spawn(move || loop {
            thread::sleep(Duration::new(1, 0));

            let now = Instant::now();
            let clients = server.clients.read().unwrap();

            for client in clients.iter() {
                let unfinished_business = client.unfinished_business.read().unwrap();
                let last_update = client.last_update.read().unwrap();

                if unfinished_business.len() > 0 && now - *last_update >= Duration::new(300, 0) {
                    println!("[{}] timed out", client.addr);
                    client.stream.shutdown(Shutdown::Both)?;
                }
            }
        });

        for stream in self.listener.incoming() {
            let client = Client::new(Arc::downgrade(&self), stream?)?;

            {
                let mut clients = self.clients.write().unwrap();
                clients.push(client.clone());
            }

            thread::spawn(move || client.run());
        }

        match watcher.join() {
            Ok(x) => Ok(x?),
            Err(e) => panic::resume_unwind(e),
        }
    }

    pub fn update_jobs(&self) -> DynResult<()> {
        {
            let clients = self.clients.read().unwrap();
            let jobs = self.jobs.read().unwrap();

            if let Some(job) = jobs.first() {
                for client in clients.iter() {
                    let state = client.state.read().unwrap();

                    if let ClientState::Worker = *state {
                        client.send_job(job)?;
                    }
                }
            }
        }

        self.update_work()
    }

    pub fn update_work(&self) -> DynResult<()> {
        {
            let clients = self.clients.read().unwrap();
            let mut jobs = self.jobs.write().unwrap();

            if let Some(job) = jobs.first_mut() {
                for client in clients.iter() {
                    let state = client.state.read().unwrap();

                    if let ClientState::Worker = *state {
                        client.send_work(job)?;
                    }
                }
            }
        }

        Ok(())
    }
}
