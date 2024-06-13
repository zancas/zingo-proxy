//! Lightwallet service RPC implementations.

use hex::FromHex;
use tokio_stream::wrappers::ReceiverStream;
use zcash_client_backend::proto::{
    compact_formats::{ChainMetadata, CompactBlock, CompactTx},
    service::{
        compact_tx_streamer_server::CompactTxStreamer, Address, Balance, BlockId, BlockRange,
        Empty, GetAddressUtxosReply, LightdInfo, RawTransaction, SubtreeRoot,
    },
};
use zebra_chain::block::Height;

use crate::{
    blockcache::block::get_block_from_node,
    define_grpc_passthrough,
    jsonrpc::{
        connector::JsonRpcConnector,
        primitives::{GetBlockResponse, GetTransactionResponse, ProxyConsensusBranchIdHex},
    },
    primitives::ProxyClient,
    utils::get_build_info,
};

/// Stream of RawTransactions, output type of get_taddress_txids.
pub struct RawTransactionStream {
    inner: ReceiverStream<Result<RawTransaction, tonic::Status>>,
}

impl RawTransactionStream {
    /// Returns new instanse of RawTransactionStream.
    pub fn new(rx: tokio::sync::mpsc::Receiver<Result<RawTransaction, tonic::Status>>) -> Self {
        RawTransactionStream {
            inner: ReceiverStream::new(rx),
        }
    }
}

impl futures::Stream for RawTransactionStream {
    type Item = Result<RawTransaction, tonic::Status>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let poll = std::pin::Pin::new(&mut self.inner).poll_next(cx);
        match poll {
            std::task::Poll::Ready(Some(Ok(raw_tx))) => std::task::Poll::Ready(Some(Ok(raw_tx))),
            std::task::Poll::Ready(Some(Err(e))) => std::task::Poll::Ready(Some(Err(e))),
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// Stream of CompactBlocks, output type of get_block_range.
pub struct CompactBlockStream {
    inner: ReceiverStream<Result<CompactBlock, tonic::Status>>,
}

impl CompactBlockStream {
    /// Returns new instanse of CompactBlockStream.
    pub fn new(rx: tokio::sync::mpsc::Receiver<Result<CompactBlock, tonic::Status>>) -> Self {
        CompactBlockStream {
            inner: ReceiverStream::new(rx),
        }
    }
}

impl futures::Stream for CompactBlockStream {
    type Item = Result<CompactBlock, tonic::Status>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let poll = std::pin::Pin::new(&mut self.inner).poll_next(cx);
        match poll {
            std::task::Poll::Ready(Some(Ok(raw_tx))) => std::task::Poll::Ready(Some(Ok(raw_tx))),
            std::task::Poll::Ready(Some(Err(e))) => std::task::Poll::Ready(Some(Err(e))),
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

impl CompactTxStreamer for ProxyClient {
    /// Return the height of the tip of the best chain.
    fn get_latest_block<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<zcash_client_backend::proto::service::ChainSpec>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<zcash_client_backend::proto::service::BlockId>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_latest_block.");
        Box::pin(async {
            let blockchain_info = JsonRpcConnector::new(
                self.zebrad_uri.clone(),
                Some("xxxxxx".to_string()),
                Some("xxxxxx".to_string()),
            )
            .await
            .get_blockchain_info()
            .await
            .map_err(|e| e.to_grpc_status())?;

            let block_id = BlockId {
                height: blockchain_info.blocks.0 as u64,
                hash: blockchain_info.best_block_hash.0.to_vec(),
            };

            Ok(tonic::Response::new(block_id))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_latest_block(
    //         &self,
    //         request: tonic::Request<ChainSpec>,
    //     ) -> BlockId
    // );

    /// Return the compact block corresponding to the given block identifier.
    ///
    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    ///
    /// TODO: This RPC should be implemented alongside the block cache.
    fn get_block<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<zcash_client_backend::proto::service::BlockId>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<zcash_client_backend::proto::compact_formats::CompactBlock>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_block.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("get_block not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_block(
    //         &self,
    //         request: tonic::Request<BlockId>,
    //     ) -> CompactBlock
    // );

    /// Same as GetBlock except actions contain only nullifiers.
    ///
    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    fn get_block_nullifiers<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<zcash_client_backend::proto::service::BlockId>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<zcash_client_backend::proto::compact_formats::CompactBlock>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_block_nullifiers.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("get_block_nullifiers not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_block_nullifiers(
    //         &self,
    //         request: tonic::Request<BlockId>,
    //     ) -> CompactBlock
    // );

    /// Server streaming response type for the GetBlockRange method.
    #[doc = "Server streaming response type for the GetBlockRange method."]
    // type GetBlockRangeStream = tonic::Streaming<CompactBlock>;
    type GetBlockRangeStream = std::pin::Pin<Box<CompactBlockStream>>;

    /// Return a list of consecutive compact blocks.
    ///
    /// TODO: This implementation is slow. An internal block cache should be implemented that this rpc, along with the get_block rpc, can rely on.
    ///       - add get_block function that queries the block cache for block and calls get_block_from_node to fetch block if not present.
    fn get_block_range<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<zcash_client_backend::proto::service::BlockRange>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<Self::GetBlockRangeStream>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_block_range.");
        let zebrad_uri = self.zebrad_uri.clone();
        Box::pin(async move {
            let blockrange = request.into_inner();
            let mut start = blockrange
                .start
                .map(|s| s.height as u32)
                .ok_or(tonic::Status::invalid_argument("Start block not specified"))?;
            let mut end = blockrange
                .end
                .map(|e| e.height as u32)
                .ok_or(tonic::Status::invalid_argument("End block not specified"))?;
            if start > end {
                (start, end) = (end, start);
            }

            let (channel_tx, channel_rx) = tokio::sync::mpsc::channel(32);
            tokio::spawn(async move {
                for height in start..end {
                    let compact_block = get_block_from_node(&zebrad_uri, &height).await;
                    match compact_block {
                        Ok(block) => {
                            println!("\nCompact Block:\n{:?}\n", block);

                            if channel_tx.send(Ok(block)).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            if channel_tx
                                .send(Err(tonic::Status::internal(e.to_string())))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                }
            });
            let output_stream = CompactBlockStream::new(channel_rx);
            let stream_boxed = Box::pin(output_stream);
            Ok(tonic::Response::new(stream_boxed))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_block_range(
    //         &self,
    //         request: tonic::Request<BlockRange>,
    //     ) -> Self::GetBlockRangeStream
    // );

    /// Server streaming response type for the GetBlockRangeNullifiers method.
    #[doc = " Server streaming response type for the GetBlockRangeNullifiers method."]
    type GetBlockRangeNullifiersStream = tonic::Streaming<CompactBlock>;

    /// Same as GetBlockRange except actions contain only nullifiers.
    ///
    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    fn get_block_range_nullifiers<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<zcash_client_backend::proto::service::BlockRange>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<Self::GetBlockRangeNullifiersStream>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_block_range_nullifiers.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("get_block_range_nullifiers not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_block_range_nullifiers(
    //         &self,
    //         request: tonic::request<blockrange>,
    //     ) -> self::getblockrangenullifiersstream
    // );

    /// Return the requested full (not compact) transaction (as from zcashd).
    fn get_transaction<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<zcash_client_backend::proto::service::TxFilter>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<zcash_client_backend::proto::service::RawTransaction>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_transaction.");
        Box::pin(async {
            let hash = request.into_inner().hash;
            if hash.len() == 32 {
                let reversed_hash = hash.iter().rev().copied().collect::<Vec<u8>>();
                let hash_hex = hex::encode(reversed_hash);
                let tx = JsonRpcConnector::new(
                    self.zebrad_uri.clone(),
                    Some("xxxxxx".to_string()),
                    Some("xxxxxx".to_string()),
                )
                .await
                .get_raw_transaction(hash_hex, Some(1))
                .await
                .map_err(|e| e.to_grpc_status())?;

                let (hex, height) = if let GetTransactionResponse::Object { hex, height, .. } = tx {
                    (hex, height)
                } else {
                    return Err(tonic::Status::not_found("Transaction not received"));
                };

                // TODO: Remove unwrap on height and handle error.
                Ok(tonic::Response::new(
                    zcash_client_backend::proto::service::RawTransaction {
                        data: hex.bytes,
                        height: height.try_into().unwrap(),
                    },
                ))
            } else {
                Err(tonic::Status::invalid_argument(
                    "Transaction hash incorrect",
                ))
            }
        })
    }
    // define_grpc_passthrough!(
    //     fn get_transaction(
    //         &self,
    //         request: tonic::Request<TxFilter>,
    //     ) -> RawTransaction
    // );

    /// Submit the given transaction to the Zcash network.
    fn send_transaction<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<zcash_client_backend::proto::service::RawTransaction>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<zcash_client_backend::proto::service::SendResponse>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of send_transaction.");
        Box::pin(async {
            let hex_tx = hex::encode(request.into_inner().data);
            let tx_output = JsonRpcConnector::new(
                self.zebrad_uri.clone(),
                Some("xxxxxx".to_string()),
                Some("xxxxxx".to_string()),
            )
            .await
            .send_raw_transaction(hex_tx)
            .await
            .map_err(|e| e.to_grpc_status())?;

            Ok(tonic::Response::new(
                zcash_client_backend::proto::service::SendResponse {
                    error_code: 0,
                    error_message: tx_output.0.to_string(),
                },
            ))
        })
    }
    // define_grpc_passthrough!(
    //     fn send_transaction(
    //         &self,
    //         request: tonic::Request<RawTransaction>,
    //     ) -> SendResponse
    // );

    /// Server streaming response type for the GetTaddressTxids method.
    #[doc = "Server streaming response type for the GetTaddressTxids method."]
    // type GetTaddressTxidsStream = tonic::Streaming<RawTransaction>;
    type GetTaddressTxidsStream = std::pin::Pin<Box<RawTransactionStream>>;

    /// This name is misleading, returns the full transactions that have either inputs or outputs connected to the given transparent address.
    ///
    /// TODO: Add 30 second timout.
    fn get_taddress_txids<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<
            zcash_client_backend::proto::service::TransparentAddressBlockFilter,
        >,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<Self::GetTaddressTxidsStream>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_taddress_txids.");
        Box::pin(async move {
            let block_filter = request.into_inner();
            let address = block_filter.address;
            let start = block_filter
                .range
                .clone()
                .and_then(|r| r.start)
                .map(|s| s.height as u32)
                .ok_or(tonic::Status::invalid_argument("Start block not specified"))?;
            let end = block_filter
                .range
                .and_then(|r| r.end)
                .map(|e| e.height as u32)
                .ok_or(tonic::Status::invalid_argument("End block not specified"))?;

            let zebrad_client = JsonRpcConnector::new(
                self.zebrad_uri.clone(),
                Some("xxxxxx".to_string()),
                Some("xxxxxx".to_string()),
            )
            .await;
            let txids = zebrad_client
                .get_address_txids(vec![address], start, end)
                .await
                .map_err(|e| e.to_grpc_status())?;

            let (tx, rx) = tokio::sync::mpsc::channel(32);
            tokio::spawn(async move {
                for txid in txids.transactions {
                    let transaction = zebrad_client.get_raw_transaction(txid, Some(1)).await;
                    match transaction {
                        Ok(GetTransactionResponse::Object { hex, height, .. }) => {
                            if tx
                                .send(Ok(RawTransaction {
                                    data: hex.bytes,
                                    height: height as u64,
                                }))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        Ok(GetTransactionResponse::Raw(_)) => {
                            if tx
                                .send(Err(tonic::Status::internal(
                                    "Received raw transaction type, this should not be impossible.",
                                )))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            if tx
                                .send(Err(tonic::Status::internal(e.to_string())))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                }
            });
            let output_stream = RawTransactionStream::new(rx);
            let stream_boxed = Box::pin(output_stream);
            Ok(tonic::Response::new(stream_boxed))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_taddress_txids(
    //         &self,
    //         request: tonic::Request<TransparentAddressBlockFilter>,
    //     ) -> Self::GetTaddressTxidsStream
    // );

    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    fn get_taddress_balance<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<zcash_client_backend::proto::service::AddressList>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<zcash_client_backend::proto::service::Balance>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_taddress_balance.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("get_taddress_balance not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_taddress_balance(
    //         &self,
    //         request: tonic::Request<AddressList>,
    //     ) -> Balance
    // );

    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn get_taddress_balance_stream<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<tonic::Streaming<Address>>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<tonic::Response<Balance>, tonic::Status>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_taddress_balance_stream.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("get_taddress_balance_stream not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }

    /// Server streaming response type for the GetMempoolTx method.
    #[doc = "Server streaming response type for the GetMempoolTx method."]
    type GetMempoolTxStream = tonic::Streaming<CompactTx>;

    /// Return the compact transactions currently in the mempool; the results
    /// can be a few seconds out of date. If the Exclude list is empty, return
    /// all transactions; otherwise return all *except* those in the Exclude list
    /// (if any); this allows the client to avoid receiving transactions that it
    /// already has (from an earlier call to this rpc). The transaction IDs in the
    /// Exclude list can be shortened to any number of bytes to make the request
    /// more bandwidth-efficient; if two or more transactions in the mempool
    /// match a shortened txid, they are all sent (none is excluded). Transactions
    /// in the exclude list that don't exist in the mempool are ignored.
    ///
    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    fn get_mempool_tx<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<zcash_client_backend::proto::service::Exclude>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<Self::GetMempoolTxStream>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_mempool_tx.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("get_mempool_tx not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_mempool_tx(
    //         &self,
    //         request: tonic::Request<Exclude>,
    //     ) -> Self::GetMempoolTxStream
    // );

    /// Server streaming response type for the GetMempoolStream method.
    #[doc = "Server streaming response type for the GetMempoolStream method."]
    type GetMempoolStreamStream = tonic::Streaming<RawTransaction>;

    // /// Return a stream of current Mempool transactions. This will keep the output stream open while
    // /// there are mempool transactions. It will close the returned stream when a new block is mined.
    // fn get_mempool_stream<'life0, 'async_trait>(
    //     &'life0 self,
    //     request: tonic::Request<Empty>,
    // ) -> core::pin::Pin<
    //     Box<
    //         dyn core::future::Future<
    //                 Output = std::result::Result<
    //                     tonic::Response<Self::GetMempoolStreamStream>,
    //                     tonic::Status,
    //                 >,
    //             > + core::marker::Send
    //             + 'async_trait,
    //     >,
    // >
    // where
    //     'life0: 'async_trait,
    //     Self: 'async_trait,
    // {
    //     println!("@zingoproxyd: Received call of get_mempool_stream.");
    //     Box::pin(async { todo!("get_mempool_stream not yet implemented") })
    // }
    define_grpc_passthrough!(
        fn get_mempool_stream(
            &self,
            request: tonic::Request<Empty>,
        ) -> Self::GetMempoolStreamStream
    );

    /// GetTreeState returns the note commitment tree state corresponding to the given block.
    /// See section 3.7 of the Zcash protocol specification. It returns several other useful
    /// values also (even though they can be obtained using GetBlock).
    /// The block can be specified by either height or hash.
    fn get_tree_state<'life0, 'async_trait>(
        &'life0 self,
        request: tonic::Request<zcash_client_backend::proto::service::BlockId>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<zcash_client_backend::proto::service::TreeState>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_tree_state.");
        Box::pin(async {
            let block_id = request.into_inner();
            let hash_or_height = if block_id.height != 0 {
                block_id.height.to_string()
            } else {
                hex::encode(block_id.hash)
            };

            let zebrad_client = JsonRpcConnector::new(
                self.zebrad_uri.clone(),
                Some("xxxxxx".to_string()),
                Some("xxxxxx".to_string()),
            )
            .await;

            // TODO: This is slow. Chain, along with other blockchain info should be saved on startup and used here [blockcache?].
            let chain = zebrad_client
                .get_blockchain_info()
                .await
                .map_err(|e| e.to_grpc_status())?
                .chain;
            let treestate = zebrad_client
                .get_treestate(hash_or_height)
                .await
                .map_err(|e| e.to_grpc_status())?;
            Ok(tonic::Response::new(
                zcash_client_backend::proto::service::TreeState {
                    network: chain,
                    height: treestate.height as u64,
                    hash: treestate.hash.to_string(),
                    time: treestate.time,
                    sapling_tree: treestate.sapling.commitments.final_state.to_string(),
                    orchard_tree: treestate.orchard.commitments.final_state.to_string(),
                },
            ))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_tree_state(
    //         &self,
    //         request: tonic::Request<BlockId>,
    //     ) -> TreeState
    // );

    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    fn get_latest_tree_state<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<Empty>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<zcash_client_backend::proto::service::TreeState>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_latest_tree_state.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("get_latest_tree_state not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_latest_tree_state(
    //         &self,
    //         request: tonic::Request<Empty>,
    //     ) -> TreeState
    // );

    /// Server streaming response type for the GetSubtreeRoots method.
    #[doc = " Server streaming response type for the GetSubtreeRoots method."]
    type GetSubtreeRootsStream = tonic::Streaming<SubtreeRoot>;

    /// Returns a stream of information about roots of subtrees of the Sapling and Orchard
    /// note commitment trees.
    ///
    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    fn get_subtree_roots<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<zcash_client_backend::proto::service::GetSubtreeRootsArg>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<Self::GetSubtreeRootsStream>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_subtree_roots.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("get_subtree_roots not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_subtree_roots(
    //         &self,
    //         request: tonic::Request<GetSubtreeRootsArg>,
    //     ) -> Self::GetSubtreeRootsStream
    // );

    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    fn get_address_utxos<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<zcash_client_backend::proto::service::GetAddressUtxosArg>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<
                            zcash_client_backend::proto::service::GetAddressUtxosReplyList,
                        >,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_address_utxos.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("get_address_utxos not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_address_utxos(
    //         &self,
    //         request: tonic::Request<GetAddressUtxosArg>,
    //     ) -> GetAddressUtxosReplyList
    // );

    /// Server streaming response type for the GetAddressUtxosStream method.
    #[doc = "Server streaming response type for the GetAddressUtxosStream method."]
    type GetAddressUtxosStreamStream = tonic::Streaming<GetAddressUtxosReply>;

    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    fn get_address_utxos_stream<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<zcash_client_backend::proto::service::GetAddressUtxosArg>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<Self::GetAddressUtxosStreamStream>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_address_utxos_stream.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("get_address_utxos_stream not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_address_utxos_stream(
    //         &self,
    //         request: tonic::Request<GetAddressUtxosArg>,
    //     ) -> tonic::Streaming<GetAddressUtxosReply>
    // );

    /// Return information about this lightwalletd instance and the blockchain
    fn get_lightd_info<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<Empty>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<zcash_client_backend::proto::service::LightdInfo>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of get_lightd_info.");
        // TODO: Add user and password as fields of ProxyClient and use here.
        // TODO: Return Nym_Address in get_lightd_info response, for use by wallets.
        Box::pin(async {
            let zebrad_client = JsonRpcConnector::new(
                self.zebrad_uri.clone(),
                Some("xxxxxx".to_string()),
                Some("xxxxxx".to_string()),
            )
            .await;

            let zebra_info = zebrad_client
                .get_info()
                .await
                .map_err(|e| e.to_grpc_status())?;
            let blockchain_info = zebrad_client
                .get_blockchain_info()
                .await
                .map_err(|e| e.to_grpc_status())?;

            let sapling_id_str = "76b809bb";
            let sapling_id = ProxyConsensusBranchIdHex(
                zebra_chain::parameters::ConsensusBranchId::from_hex(sapling_id_str).unwrap(),
            );
            let sapling_height = blockchain_info
                .upgrades
                .get(&sapling_id)
                .map_or(Height(1), |sapling_json| sapling_json.activation_height);

            let build_info = get_build_info();

            let lightd_info = LightdInfo {
                version: build_info.version,
                vendor: "ZingoLabs ZingoProxyD".to_string(),
                taddr_support: true,
                chain_name: blockchain_info.chain,
                sapling_activation_height: sapling_height.0 as u64,
                consensus_branch_id: blockchain_info.consensus.chain_tip.0.to_string(),
                block_height: blockchain_info.blocks.0 as u64,
                git_commit: build_info.commit_hash,
                branch: build_info.branch,
                build_date: build_info.build_date,
                build_user: build_info.build_user,
                estimated_height: blockchain_info.estimated_height.0 as u64,
                zcashd_build: zebra_info.build,
                zcashd_subversion: zebra_info.subversion,
            };

            Ok(tonic::Response::new(lightd_info))
        })
    }
    // define_grpc_passthrough!(
    //     fn get_lightd_info(
    //         &self,
    //         request: tonic::Request<Empty>,
    //     ) -> LightdInfo
    // );

    // /// Testing-only, requires lightwalletd --ping-very-insecure (do not enable in production) [from zebrad]
    /// This RPC has not been implemented as it is not currently used by zingolib.
    /// If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy).
    fn ping<'life0, 'async_trait>(
        &'life0 self,
        _request: tonic::Request<zcash_client_backend::proto::service::Duration>,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<
                    Output = std::result::Result<
                        tonic::Response<zcash_client_backend::proto::service::PingResponse>,
                        tonic::Status,
                    >,
                > + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        println!("@zingoproxyd: Received call of ping.");
        Box::pin(async {
            Err(tonic::Status::unimplemented("ping not yet implemented. If you require this RPC please open an issue or PR at the Zingo-Proxy github (https://github.com/zingolabs/zingo-proxy)."))
        })
    }
    // define_grpc_passthrough!(
    //     fn ping(
    //         &self,
    //         request: tonic::Request<zcash_client_backend::proto::service::Duration>,
    //     ) -> PingResponse
    // );
}
