//! Zingo-Indexer gRPC server.

use http::Uri;
use nym_sphinx_anonymous_replies::requests::AnonymousSenderTag;
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};

use crate::server::{
    error::{IngestorError, ServerError, WorkerError},
    ingestor::{NymIngestor, TcpIngestor},
    queue::Queue,
    request::ZingoIndexerRequest,
    worker::{WorkerPool, WorkerPoolStatus},
    AtomicStatus, StatusType,
};

/// Holds the status of the server and all its components.
#[derive(Debug, Clone)]
pub struct ServerStatus {
    /// Status of the Server.
    pub server_status: AtomicStatus,
    tcp_ingestor_status: AtomicStatus,
    nym_ingestor_status: AtomicStatus,
    nym_dispatcher_status: AtomicStatus,
    workerpool_status: WorkerPoolStatus,
    request_queue_status: Arc<AtomicUsize>,
    nym_response_queue_status: Arc<AtomicUsize>,
}

impl ServerStatus {
    /// Creates a ServerStatus.
    pub fn new(max_workers: u16) -> Self {
        ServerStatus {
            server_status: AtomicStatus::new(5),
            tcp_ingestor_status: AtomicStatus::new(5),
            nym_ingestor_status: AtomicStatus::new(5),
            nym_dispatcher_status: AtomicStatus::new(5),
            workerpool_status: WorkerPoolStatus::new(max_workers),
            request_queue_status: Arc::new(AtomicUsize::new(0)),
            nym_response_queue_status: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Returns the ServerStatus.
    pub fn load(&self) -> ServerStatus {
        self.server_status.load();
        self.tcp_ingestor_status.load();
        self.nym_ingestor_status.load();
        self.nym_dispatcher_status.load();
        self.workerpool_status.load();
        self.request_queue_status.load(Ordering::SeqCst);
        self.nym_response_queue_status.load(Ordering::SeqCst);
        self.clone()
    }
}

/// LightWallet server capable of servicing clients over both http and nym.
pub struct Server {
    /// Listens for incoming gRPC requests over HTTP.
    tcp_ingestor: Option<TcpIngestor>,
    /// Listens for incoming gRPC requests over Nym Mixnet, also sends responses back to clients.
    nym_ingestor: Option<NymIngestor>,
    /// Dynamically sized pool of workers.
    worker_pool: WorkerPool,
    /// Request queue.
    request_queue: Queue<ZingoIndexerRequest>,
    /// Nym response queue.
    nym_response_queue: Queue<(Vec<u8>, AnonymousSenderTag)>,
    /// Servers current status.
    status: ServerStatus,
    /// Represents the Online status of the Server.
    pub online: Arc<AtomicBool>,
}

impl Server {
    /// Spawns a new Server.
    pub async fn spawn(
        tcp_active: bool,
        tcp_ingestor_listen_addr: Option<SocketAddr>,
        nym_active: bool,
        nym_conf_path: Option<String>,
        lightwalletd_uri: Uri,
        zebrad_uri: Uri,
        max_queue_size: u16,
        max_worker_pool_size: u16,
        idle_worker_pool_size: u16,
        status: ServerStatus,
        online: Arc<AtomicBool>,
    ) -> Result<Self, ServerError> {
        if (!tcp_active) && (!nym_active) {
            return Err(ServerError::ServerConfigError(
                "Cannot start server with no ingestors selected, at least one of either nym or tcp must be set to active in conf.".to_string(),
            ));
        }
        if tcp_active && tcp_ingestor_listen_addr.is_none() {
            return Err(ServerError::ServerConfigError(
                "TCP is active but no address provided.".to_string(),
            ));
        }
        if nym_active && nym_conf_path.is_none() {
            return Err(ServerError::ServerConfigError(
                "NYM is active but no conf path provided.".to_string(),
            ));
        }
        println!("Launching Server!\n");
        status.server_status.store(0);
        let request_queue: Queue<ZingoIndexerRequest> =
            Queue::new(max_queue_size as usize, status.request_queue_status.clone());
        status.request_queue_status.store(0, Ordering::SeqCst);
        let nym_response_queue: Queue<(Vec<u8>, AnonymousSenderTag)> = Queue::new(
            max_queue_size as usize,
            status.nym_response_queue_status.clone(),
        );
        status.nym_response_queue_status.store(0, Ordering::SeqCst);
        let tcp_ingestor = if tcp_active {
            println!("Launching TcpIngestor..");
            Some(
                TcpIngestor::spawn(
                    tcp_ingestor_listen_addr
                        .expect("tcp_ingestor_listen_addr returned none when used."),
                    request_queue.tx().clone(),
                    status.tcp_ingestor_status.clone(),
                    online.clone(),
                )
                .await?,
            )
        } else {
            None
        };
        let nym_ingestor = if nym_active {
            println!("Launching NymIngestor..");
            let nym_conf_path_string =
                nym_conf_path.expect("nym_conf_path returned none when used.");
            Some(
                NymIngestor::spawn(
                    nym_conf_path_string.clone().as_str(),
                    request_queue.tx().clone(),
                    nym_response_queue.rx().clone(),
                    nym_response_queue.tx().clone(),
                    status.nym_ingestor_status.clone(),
                    online.clone(),
                )
                .await?,
            )
        } else {
            None
        };

        println!("Launching WorkerPool..");
        let worker_pool = WorkerPool::spawn(
            max_worker_pool_size,
            idle_worker_pool_size,
            request_queue.rx().clone(),
            request_queue.tx().clone(),
            nym_response_queue.tx().clone(),
            lightwalletd_uri,
            zebrad_uri,
            status.workerpool_status.clone(),
            online.clone(),
        )
        .await;
        Ok(Server {
            tcp_ingestor,
            nym_ingestor,
            worker_pool,
            request_queue,
            nym_response_queue,
            status: status.clone(),
            online,
        })
    }

    /// Starts the gRPC service.
    ///
    /// Launches all components then enters command loop:
    /// - Checks request queue and workerpool to spawn / despawn workers as required.
    /// - Updates the ServerStatus.
    /// - Checks for shutdown signal, shutting down server if received.
    pub async fn serve(mut self) -> tokio::task::JoinHandle<Result<(), ServerError>> {
        tokio::task::spawn(async move {
            // NOTE: This interval may need to be reduced or removed / moved once scale testing begins.
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
            let mut nym_ingestor_handle = None;
            let mut tcp_ingestor_handle = None;
            let mut worker_handles;
            if let Some(ingestor) = self.nym_ingestor.take() {
                nym_ingestor_handle = Some(ingestor.serve().await);
            }
            if let Some(ingestor) = self.tcp_ingestor.take() {
                tcp_ingestor_handle = Some(ingestor.serve().await);
            }
            worker_handles = self.worker_pool.clone().serve().await;
            self.status.server_status.store(1);
            loop {
                if self.request_queue.queue_length() >= (self.request_queue.max_length() / 4)
                    && (self.worker_pool.workers() < self.worker_pool.max_size() as usize)
                {
                    match self.worker_pool.push_worker().await {
                        Ok(handle) => {
                            worker_handles.push(handle);
                        }
                        Err(_e) => {
                            eprintln!("WorkerPool at capacity");
                        }
                    }
                } else if (self.request_queue.queue_length() <= 1)
                    && (self.worker_pool.workers() > self.worker_pool.idle_size() as usize)
                {
                    let worker_index = self.worker_pool.workers() - 1;
                    let worker_handle = worker_handles.remove(worker_index);
                    match self.worker_pool.pop_worker(worker_handle).await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Failed to pop worker from pool: {}", e);
                            // TODO: Handle this error.
                        }
                    }
                }
                self.statuses();
                // TODO: Implement check_statuses() and run here.
                if self.check_for_shutdown().await {
                    self.status.server_status.store(4);
                    let worker_handle_options: Vec<
                        Option<tokio::task::JoinHandle<Result<(), WorkerError>>>,
                    > = worker_handles.into_iter().map(Some).collect();
                    self.shutdown_components(
                        tcp_ingestor_handle,
                        nym_ingestor_handle,
                        worker_handle_options,
                    )
                    .await;
                    self.status.server_status.store(5);
                    return Ok(());
                }
                interval.tick().await;
            }
        })
    }

    /// Checks indexers online status and servers internal status for closure signal.
    pub async fn check_for_shutdown(&self) -> bool {
        if self.status() >= 4 {
            return true;
        }
        if !self.check_online() {
            return true;
        }
        false
    }

    /// Sets the servers to close gracefully.
    pub async fn shutdown(&mut self) {
        self.status.server_status.store(4)
    }

    /// Sets the server's components to close gracefully.
    async fn shutdown_components(
        &mut self,
        tcp_ingestor_handle: Option<tokio::task::JoinHandle<Result<(), IngestorError>>>,
        nym_ingestor_handle: Option<tokio::task::JoinHandle<Result<(), IngestorError>>>,
        mut worker_handles: Vec<Option<tokio::task::JoinHandle<Result<(), WorkerError>>>>,
    ) {
        if let Some(handle) = tcp_ingestor_handle {
            self.status.tcp_ingestor_status.store(4);
            handle.await.ok();
        }
        if let Some(handle) = nym_ingestor_handle {
            self.status.nym_ingestor_status.store(4);
            handle.await.ok();
        }
        self.worker_pool.shutdown(&mut worker_handles).await;
    }

    /// Returns the servers current status usize.
    pub fn status(&self) -> usize {
        self.status.server_status.load()
    }

    /// Returns the servers current statustype.
    pub fn statustype(&self) -> StatusType {
        StatusType::from(self.status())
    }

    /// Updates and returns the status of the server and its parts.
    pub fn statuses(&mut self) -> ServerStatus {
        self.status.server_status.load();
        self.status.tcp_ingestor_status.load();
        self.status.nym_ingestor_status.load();
        self.status.nym_dispatcher_status.load();
        self.status
            .request_queue_status
            .store(self.request_queue.queue_length(), Ordering::SeqCst);
        self.status
            .nym_response_queue_status
            .store(self.nym_response_queue.queue_length(), Ordering::SeqCst);
        self.worker_pool.status();
        self.status.clone()
    }

    /// Checks statuses, handling errors.
    pub async fn check_statuses(&mut self) {
        todo!()
    }

    /// Check the online status on the indexer.
    fn check_online(&self) -> bool {
        self.online.load(Ordering::SeqCst)
    }
}
