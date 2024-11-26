//! Zaino Testing Utilities.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

use once_cell::sync::Lazy;
use std::{path::PathBuf, str::FromStr};
use tempfile::TempDir;
use zcash_local_net::validator::Validator;

/// Path for zcashd binary.
pub static ZCASHD_BIN: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let mut workspace_root_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    workspace_root_path.pop();
    Some(workspace_root_path.join("test_binaries/bins/zcashd"))
});

/// Path for zcash-cli binary.
pub static ZCASH_CLI_BIN: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let mut workspace_root_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    workspace_root_path.pop();
    Some(workspace_root_path.join("test_binaries/bins/zcash-cli"))
});

/// Path for zebrad binary.
pub static ZEBRAD_BIN: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let mut workspace_root_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    workspace_root_path.pop();
    Some(workspace_root_path.join("test_binaries/bins/zebrad"))
});

/// Path for lightwalletd binary.
pub static LIGHTWALLETD_BIN: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let mut workspace_root_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    workspace_root_path.pop();
    Some(workspace_root_path.join("test_binaries/bins/lightwalletd"))
});

/// Path for zainod binary.
pub static ZAINOD_BIN: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let mut workspace_root_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    workspace_root_path.pop();
    Some(workspace_root_path.join("target/release/zainod"))
});

/// Path for zcashd chain cache.
pub static ZCASHD_CHAIN_CACHE_BIN: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let mut workspace_root_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    workspace_root_path.pop();
    Some(workspace_root_path.join("integration-tests/chain_cache/client_rpc_tests"))
});

/// Path for zebrad chain cache.
pub static ZEBRAD_CHAIN_CACHE_BIN: Lazy<Option<PathBuf>> = Lazy::new(|| {
    let mut workspace_root_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    workspace_root_path.pop();
    Some(workspace_root_path.join("integration-tests/chain_cache/client_rpc_tests_large"))
});

/// Represents the type of validator to launch.
pub enum ValidatorKind {
    /// Zcashd.
    Zcashd,
    /// Zebrad.
    Zebrad,
}

impl std::str::FromStr for ValidatorKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zcashd" => Ok(ValidatorKind::Zcashd),
            "zebrad" => Ok(ValidatorKind::Zebrad),
            _ => Err(format!("Invalid validator kind: {}", s)),
        }
    }
}

/// Config for validators.
pub enum ValidatorConfig {
    /// Zcashd Config.
    ZcashdConfig(zcash_local_net::validator::ZcashdConfig),
    /// Zebrad Config.
    ZebradConfig(zcash_local_net::validator::ZebradConfig),
}

/// Available zcash-local-net configurations.
pub enum LocalNet {
    /// Zcash-local-net backed by Zcashd.
    Zcashd(
        zcash_local_net::LocalNet<
            zcash_local_net::indexer::Empty,
            zcash_local_net::validator::Zcashd,
        >,
    ),
    /// Zcash-local-net backed by Zebrad.
    Zebrad(
        zcash_local_net::LocalNet<
            zcash_local_net::indexer::Empty,
            zcash_local_net::validator::Zebrad,
        >,
    ),
}

impl zcash_local_net::validator::Validator for LocalNet {
    const CONFIG_FILENAME: &str = "";

    type Config = ValidatorConfig;

    #[allow(clippy::manual_async_fn)]
    fn launch(
        config: Self::Config,
    ) -> impl std::future::Future<Output = Result<Self, zcash_local_net::error::LaunchError>> + Send
    {
        async move {
            match config {
                ValidatorConfig::ZcashdConfig(cfg) => {
                    let net = zcash_local_net::LocalNet::<
                        zcash_local_net::indexer::Empty,
                        zcash_local_net::validator::Zcashd,
                    >::launch(
                        zcash_local_net::indexer::EmptyConfig {}, cfg
                    )
                    .await;
                    Ok(LocalNet::Zcashd(net))
                }
                ValidatorConfig::ZebradConfig(cfg) => {
                    let net = zcash_local_net::LocalNet::<
                        zcash_local_net::indexer::Empty,
                        zcash_local_net::validator::Zebrad,
                    >::launch(
                        zcash_local_net::indexer::EmptyConfig {}, cfg
                    )
                    .await;
                    Ok(LocalNet::Zebrad(net))
                }
            }
        }
    }

    fn stop(&mut self) {
        match self {
            LocalNet::Zcashd(net) => net.validator_mut().stop(),
            LocalNet::Zebrad(net) => net.validator_mut().stop(),
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn generate_blocks(
        &self,
        n: u32,
    ) -> impl std::future::Future<Output = std::io::Result<()>> + Send {
        async move {
            match self {
                LocalNet::Zcashd(net) => net.validator().generate_blocks(n).await,
                LocalNet::Zebrad(net) => net.validator().generate_blocks(n).await,
            }
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn get_chain_height(
        &self,
    ) -> impl std::future::Future<Output = zcash_protocol::consensus::BlockHeight> + Send {
        async move {
            match self {
                LocalNet::Zcashd(net) => net.validator().get_chain_height().await,
                LocalNet::Zebrad(net) => net.validator().get_chain_height().await,
            }
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn poll_chain_height(
        &self,
        target_height: zcash_protocol::consensus::BlockHeight,
    ) -> impl std::future::Future<Output = ()> + Send {
        async move {
            match self {
                LocalNet::Zcashd(net) => net.validator().poll_chain_height(target_height).await,
                LocalNet::Zebrad(net) => net.validator().poll_chain_height(target_height).await,
            }
        }
    }

    fn config_dir(&self) -> &TempDir {
        match self {
            LocalNet::Zcashd(net) => net.validator().config_dir(),
            LocalNet::Zebrad(net) => net.validator().config_dir(),
        }
    }

    fn logs_dir(&self) -> &TempDir {
        match self {
            LocalNet::Zcashd(net) => net.validator().logs_dir(),
            LocalNet::Zebrad(net) => net.validator().logs_dir(),
        }
    }

    fn data_dir(&self) -> &TempDir {
        match self {
            LocalNet::Zcashd(net) => net.validator().data_dir(),
            LocalNet::Zebrad(net) => net.validator().data_dir(),
        }
    }

    fn network(&self) -> zcash_local_net::network::Network {
        match self {
            LocalNet::Zcashd(net) => net.validator().network(),
            LocalNet::Zebrad(net) => *net.validator().network(),
        }
    }

    fn load_chain(
        chain_cache: PathBuf,
        validator_data_dir: PathBuf,
        validator_network: zcash_local_net::network::Network,
    ) -> PathBuf {
        match validator_network {
            zcash_local_net::network::Network::Regtest => {
                zcash_local_net::validator::Zcashd::load_chain(
                    chain_cache,
                    validator_data_dir,
                    validator_network,
                )
            }
            _ => zcash_local_net::validator::Zebrad::load_chain(
                chain_cache,
                validator_data_dir,
                validator_network,
            ),
        }
    }
}

/// Holds zingo lightclients along with thier TempDir for wallet-2-validator tests.
pub struct Clients {
    /// Lightclient TempDir location.
    pub lightclient_dir: TempDir,
    /// Faucet (zingolib lightclient).
    ///
    /// Mining rewards are recieved by this client for use in tests.
    pub faucet: zingolib::lightclient::LightClient,
    /// Recipient (zingolib lightclient).
    pub recipient: zingolib::lightclient::LightClient,
}

impl Clients {
    /// Returns the zcash address of the faucet.
    pub async fn get_faucet_address(&self, pool: &str) -> String {
        zingolib::get_base_address_macro!(self.faucet, pool)
    }

    /// Returns the zcash address of the recipient.
    pub async fn get_recipient_address(&self, pool: &str) -> String {
        zingolib::get_base_address_macro!(self.recipient, pool)
    }
}

/// Configuration data for Zingo-Indexer Tests.
pub struct TestManager2 {
    /// Zcash-local-net.
    pub local_net: LocalNet,
    /// Zebrad/Zcashd JsonRpc listen port.
    pub zebrad_rpc_listen_port: u16,
    /// Zaino Indexer JoinHandle.
    pub zaino_handle: Option<tokio::task::JoinHandle<Result<(), zainodlib::error::IndexerError>>>,
    /// Zingo-Indexer gRPC listen port.
    pub zaino_grpc_listen_port: Option<u16>,
    /// Zingolib lightclients.
    pub clients: Option<Clients>,
    /// Online status of Zingo-Indexer.
    pub online: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl TestManager2 {
    /// Launches zcash-local-net<Empty, Validator>.
    ///
    /// Possible validators: Zcashd, Zebrad.
    ///
    /// If chain_cache is given a path the chain will be loaded.
    ///
    /// If clients is set to active zingolib lightclients will be created for test use.
    pub async fn launch(
        validator: &str,
        chain_cache: Option<PathBuf>,
        enable_zaino: bool,
        enable_clients: bool,
    ) -> Result<Self, std::io::Error> {
        let validator_kind = ValidatorKind::from_str(validator).unwrap();
        if enable_clients && !enable_zaino {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot enable clients when zaino is not enabled.",
            ));
        }
        let online = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

        // Launch LocalNet:
        let zebrad_rpc_listen_port = portpicker::pick_unused_port().expect("No ports free");
        let validator_config = match validator_kind {
            ValidatorKind::Zcashd => {
                let cfg = zcash_local_net::validator::ZcashdConfig {
                    zcashd_bin: ZCASHD_BIN.clone(),
                    zcash_cli_bin: ZCASH_CLI_BIN.clone(),
                    rpc_port: Some(zebrad_rpc_listen_port),
                    activation_heights: zcash_local_net::network::ActivationHeights::default(),
                    miner_address: Some(zingolib::testvectors::REG_O_ADDR_FROM_ABANDONART),
                    chain_cache,
                };
                ValidatorConfig::ZcashdConfig(cfg)
            }
            ValidatorKind::Zebrad => {
                let cfg = zcash_local_net::validator::ZebradConfig {
                    zebrad_bin: ZEBRAD_BIN.clone(),
                    network_listen_port: None,
                    rpc_listen_port: Some(zebrad_rpc_listen_port),
                    activation_heights: zcash_local_net::network::ActivationHeights::default(),
                    miner_address: zcash_local_net::validator::ZEBRAD_DEFAULT_MINER,
                    chain_cache,
                    network: zcash_local_net::network::Network::Regtest,
                };
                ValidatorConfig::ZebradConfig(cfg)
            }
        };
        let local_net = LocalNet::launch(validator_config).await.unwrap();

        // Launch Zaino:
        let (zaino_grpc_listen_port, zaino_handle) = if enable_zaino {
            let zaino_grpc_listen_port = portpicker::pick_unused_port().expect("No ports free");
            // NOTE: queue and workerpool sizes may need to be changed here.
            let indexer_config = zainodlib::config::IndexerConfig {
                tcp_active: true,
                listen_port: Some(zaino_grpc_listen_port),
                // NOTE: Remove field from IndexerConfig with the removal of current testutils.
                lightwalletd_port: portpicker::pick_unused_port().expect("No ports free"),
                zebrad_port: zebrad_rpc_listen_port,
                node_user: Some("xxxxxx".to_string()),
                node_password: Some("xxxxxx".to_string()),
                max_queue_size: 512,
                max_worker_pool_size: 64,
                idle_worker_pool_size: 4,
            };
            let handle = zainodlib::indexer::Indexer::new(indexer_config, online.clone())
                .await
                .unwrap()
                .serve()
                .await
                .unwrap();
            // NOTE: This is required to give the server time to launch, this is not used in production code but could be rewritten to improve testing efficiency.
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            (Some(zaino_grpc_listen_port), Some(handle))
        } else {
            (None, None)
        };

        // Launch Zingolib Lightclients:
        let clients = if enable_clients {
            let lightclient_dir = tempfile::tempdir().unwrap();
            let lightclients = zcash_local_net::client::build_lightclients(
                lightclient_dir.path().to_path_buf(),
                zaino_grpc_listen_port
                    .expect("Error launching zingo lightclients. `enable_zaino` is None."),
            )
            .await;
            Some(Clients {
                lightclient_dir,
                faucet: lightclients.0,
                recipient: lightclients.1,
            })
        } else {
            None
        };

        Ok(Self {
            local_net,
            zebrad_rpc_listen_port,
            zaino_handle,
            zaino_grpc_listen_port,
            clients,
            online,
        })
    }

    /// Closes the TestManager.
    pub async fn close(&mut self) {
        self.online
            .store(false, std::sync::atomic::Ordering::SeqCst);
        if let Some(zaino_handle) = self.zaino_handle.take() {
            if let Err(e) = zaino_handle.await {
                eprintln!("Error awaiting zaino_handle: {:?}", e);
            }
        }
    }
}

impl Drop for TestManager2 {
    fn drop(&mut self) {
        self.online
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn launch_testmanager_zebrad() {
        let mut test_manager = TestManager2::launch("zebrad", None, false, false)
            .await
            .unwrap();
        assert_eq!(
            1,
            u32::from(test_manager.local_net.get_chain_height().await)
        );
        test_manager.close().await;
    }

    #[tokio::test]
    async fn launch_testmanager_zcashd() {
        let mut test_manager = TestManager2::launch("zcashd", None, false, false)
            .await
            .unwrap();
        assert_eq!(
            1,
            u32::from(test_manager.local_net.get_chain_height().await)
        );
        test_manager.close().await;
    }

    #[tokio::test]
    async fn launch_testmanager_zebrad_generate_blocks() {
        let mut test_manager = TestManager2::launch("zebrad", None, false, false)
            .await
            .unwrap();
        assert_eq!(
            1,
            u32::from(test_manager.local_net.get_chain_height().await)
        );
        test_manager.local_net.generate_blocks(1).await.unwrap();
        assert_eq!(
            2,
            u32::from(test_manager.local_net.get_chain_height().await)
        );
        test_manager.close().await;
    }

    #[tokio::test]
    async fn launch_testmanager_zcashd_generate_blocks() {
        let mut test_manager = TestManager2::launch("zcashd", None, false, false)
            .await
            .unwrap();
        assert_eq!(
            1,
            u32::from(test_manager.local_net.get_chain_height().await)
        );
        test_manager.local_net.generate_blocks(1).await.unwrap();
        assert_eq!(
            2,
            u32::from(test_manager.local_net.get_chain_height().await)
        );
        test_manager.close().await;
    }

    #[tokio::test]
    async fn launch_testmanager_zebrad_with_chain() {
        let mut test_manager =
            TestManager2::launch("zebrad", ZEBRAD_CHAIN_CACHE_BIN.clone(), false, false)
                .await
                .unwrap();
        assert_eq!(
            52,
            u32::from(test_manager.local_net.get_chain_height().await)
        );
        test_manager.close().await;
    }

    #[tokio::test]
    async fn launch_testmanager_zcashd_with_chain() {
        let mut test_manager =
            TestManager2::launch("zcashd", ZCASHD_CHAIN_CACHE_BIN.clone(), false, false)
                .await
                .unwrap();
        assert_eq!(
            10,
            u32::from(test_manager.local_net.get_chain_height().await)
        );
        test_manager.close().await;
    }

    #[tokio::test]
    async fn launch_testmanager_zebrad_zaino() {
        let mut test_manager = TestManager2::launch("zebrad", None, true, false)
            .await
            .unwrap();
        let mut grpc_client =
            zcash_local_net::client::build_client(zcash_local_net::network::localhost_uri(
                test_manager
                    .zaino_grpc_listen_port
                    .expect("Zaino listen port not available but zaino is active."),
            ))
            .await
            .unwrap();
        grpc_client
            .get_lightd_info(tonic::Request::new(
                zcash_client_backend::proto::service::Empty {},
            ))
            .await
            .unwrap();
        test_manager.close().await;
    }

    #[tokio::test]
    async fn launch_testmanager_zcashd_zaino() {
        let mut test_manager = TestManager2::launch("zcashd", None, true, false)
            .await
            .unwrap();
        let mut grpc_client =
            zcash_local_net::client::build_client(zcash_local_net::network::localhost_uri(
                test_manager
                    .zaino_grpc_listen_port
                    .expect("Zaino listen port is not available but zaino is active."),
            ))
            .await
            .unwrap();
        grpc_client
            .get_lightd_info(tonic::Request::new(
                zcash_client_backend::proto::service::Empty {},
            ))
            .await
            .unwrap();
        test_manager.close().await;
    }

    #[tokio::test]
    async fn launch_testmanager_zebrad_zaino_clients() {
        let mut test_manager = TestManager2::launch("zebrad", None, true, true)
            .await
            .unwrap();
        let clients = test_manager
            .clients
            .as_ref()
            .expect("Clients are not initialized");
        clients.faucet.do_info().await;
        clients.recipient.do_info().await;
        test_manager.close().await;
    }

    #[tokio::test]
    async fn launch_testmanager_zcashd_zaino_clients() {
        let mut test_manager = TestManager2::launch("zcashd", None, true, true)
            .await
            .unwrap();
        let clients = test_manager
            .clients
            .as_ref()
            .expect("Clients are not initialized");
        clients.faucet.do_info().await;
        clients.recipient.do_info().await;
        test_manager.close().await;
    }

    #[tokio::test]
    async fn launch_testmanager_zebrad_zaino_clients_receive_mining_reward() {
        let mut test_manager = TestManager2::launch("zebrad", None, true, true)
            .await
            .unwrap();
        let clients = test_manager
            .clients
            .as_ref()
            .expect("Clients are not initialized");

        clients.faucet.do_sync(true).await.unwrap();

        assert!(
                clients.faucet.do_balance().await.orchard_balance.unwrap() > 0
                    || clients.faucet.do_balance().await.transparent_balance.unwrap() > 0,
                "No mining reward recieved from Zebrad. Faucet Orchard Balance: {:}. Faucet Transparent Balance: {:}.",
                clients.faucet.do_balance().await.orchard_balance.unwrap(), 
                clients.faucet.do_balance().await.transparent_balance.unwrap()
            );

        test_manager.close().await;
    }

        #[tokio::test]
    async fn launch_testmanager_zcashd_zaino_clients_receive_mining_reward() {
        let mut test_manager = TestManager2::launch("zcashd", None, true, true)
            .await
            .unwrap();
        let clients = test_manager
            .clients
            .as_ref()
            .expect("Clients are not initialized");

        clients.faucet.do_sync(true).await.unwrap();

        assert!(
                clients.faucet.do_balance().await.orchard_balance.unwrap() > 0
                    || clients.faucet.do_balance().await.transparent_balance.unwrap() > 0,
                "No mining reward recieved from Zcashd. Faucet Orchard Balance: {:}. Faucet Transparent Balance: {:}.",
                clients.faucet.do_balance().await.orchard_balance.unwrap(), 
                clients.faucet.do_balance().await.transparent_balance.unwrap()
            );

        test_manager.close().await;
    }
}
