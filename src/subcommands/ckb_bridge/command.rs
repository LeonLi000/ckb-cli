use clap::{App, ArgMatches};
use ckb_sdk::{HttpRpcClient, GenesisInfo};
use crate::plugin::PluginManager;
use std::path::PathBuf;
use crate::utils::index::IndexController;
use ckb_index::{IndexDatabase, with_index_db};
use crate::utils::other::{sync_to_tip, get_network_type, get_arg_value};
use ckb_types::core::BlockView;
use ckb_types::H256;
use ckb_types::prelude::Unpack;
use crate::subcommands::{CliSubCommand, Output};

pub struct CkbBridgeSubCommand<'a> {
    rpc_client: &'a mut HttpRpcClient,
    plugin_mgr: &'a mut PluginManager,
    genesis_info: Option<GenesisInfo>,
    index_dir: PathBuf,
    index_controller: IndexController,
    wait_for_sync: bool,
}

pub struct ToCkbLog<'a> {
    status: &'a mut ToCkbLogStatus,
}

pub struct FromCkbLog<'a> {
    status: &'a mut FromCkbLogStatus,
}

#[derive(Clone, Copy, IntEnum, PartialEq, Debug)]
pub enum ToCkbLogStatus {
    UnKnow = 0,
    Approved = 1,
    Locked = 2,
    ParseProof = 3,
    WaitBlockSafe = 4,
    Mint = 5,
}

#[derive(Clone, Copy, IntEnum, PartialEq, Debug)]
pub enum FromCkbLogStatus {
    UnKnow = 0,
    Burned = 1,
    ParseProof = 2,
    WaitBlockSafe = 3,
    Mint = 4,
}


impl<'a> CkbBridgeSubCommand<'a> {
    pub fn new(
        rpc_client: &'a mut HttpRpcClient,
        plugin_mgr: &'a mut PluginManager,
        genesis_info: Option<GenesisInfo>,
        index_dir: PathBuf,
        index_controller: IndexController,
        wait_for_sync: bool,
    ) -> CkbBridgeSubCommand<'a> {
        CkbBridgeSubCommand {
            rpc_client,
            plugin_mgr,
            genesis_info,
            index_dir,
            index_controller,
            wait_for_sync,
        }
    }

    fn genesis_info(&mut self) -> Result<GenesisInfo, String> {
        if self.genesis_info.is_none() {
            let genesis_block: BlockView = self
                .rpc_client
                .get_block_by_number(0)?
                .expect("Can not get genesis block?")
                .into();
            self.genesis_info = Some(GenesisInfo::from_block(&genesis_block)?);
        }
        Ok(self.genesis_info.clone().unwrap())
    }

    fn with_db<F, T>(&mut self, func: F) -> Result<T, String>
        where
            F: FnOnce(IndexDatabase) -> T,
    {
        if self.wait_for_sync {
            sync_to_tip(&self.index_controller)?;
        }
        let network_type = get_network_type(self.rpc_client)?;
        let genesis_info = self.genesis_info()?;
        let genesis_hash: H256 = genesis_info.header().hash().unpack();
        with_index_db(&self.index_dir, genesis_hash, |backend, cf| {
            let db = IndexDatabase::from_db(backend, cf, network_type, genesis_info, false)?;
            Ok(func(db))
        })
            .map_err(|_err| {
                format!(
                    "Index database may not ready, sync process: {}",
                    self.index_controller.state().read().to_string()
                )
            })
    }

    pub fn subcommand() -> App<'static> {
        App::new("ckb-bridge")
            .about("ckb bridge cli tools")
            .subcommands(vec![
                App::new("transfer-erc20-to-ckb").about("transfer erc20 token from ethereum to ckb chain"),
                App::new("transfer-erc20-from-ckb")
                    .about("transfer erc20 token from ckb chain to ethereum"),
                App::new("deploy-sol")
                    .about("set btc difficulty cell and write the outpoint to config"),
                App::new("deploy-ckb")
                    .about("deploy toCKB scripts"),
            ])
    }

    /**
        发生 transfer 的过程中可能因为某些原因发生中断，故需要保存 transfer 状态。
        0. 初始化状态 status: UnKnow
        1. 调用erc20 approve() => status: approved
        2. 调用sol lock() => status: locked
        3. 通过上一步得到的tx_hash, 解析得到 spv proof data => status: parseProof
        4. 等待 eth block 达到 commit 状态 => status: waitBlockSafe
        5. 组装 ckb tx, 验证 eth spv proof => status: mint
        6. issue token => status: UnKnow
    */
    pub fn transfer_to_ckb(&mut self) -> Result<Output, String>{
        let log = load_to_ckb_log();
        match log.status {
            ToCkbLogStatus::UnKnow =>{
                //TODO: do erc20 approve()
            },
            ToCkbLogStatus::Approved =>{
                //TODO: do lock()
            },
            ToCkbLogStatus::Locked => {
                // TODO: do parse proof
            },
            ToCkbLogStatus::ParseProof =>{
                // TODO: do wait block safe
            },
            ToCkbLogStatus::WaitBlockSafe => {
                // TODO: do send ckb tx to verify spv proof
            },
            ToCkbLogStatus::Mint => {
                // TODO: do issue new token.
            },
        }
        Ok(Output::new_output("finished to transfer erc20 to ckb."))
    }

    pub fn transfer_from_ckb(&mut self) -> Result<Output, String>{
        let log = load_from_ckb_log();
        match log.status {
            FromCkbLogStatus::UnKnow =>{
                //TODO: do erc20 approve()
            },
            FromCkbLogStatus::Burned =>{
                //TODO: do lock()
            },
            FromCkbLogStatus::ParseProof =>{
                // TODO: do wait block safe
            },
            FromCkbLogStatus::WaitBlockSafe => {
                // TODO: do send ckb tx to verify spv proof
            },
            FromCkbLogStatus::Mint => {
                // TODO: do issue new token.
            },
        }
        Ok(Output::new_output("finished to transfer erc20 from ckb."))
    }

    pub fn deploy_sol(&mut self) {
        todo!()
    }

    pub fn deploy_ckb(&mut self) {
        todo!()
    }




}

fn load_to_ckb_log() -> ToCkbLog {
    ToCkbLog{ status: &mut ToCkbLogStatus::UnKnow }
}

fn load_from_ckb_log() -> FromCkbLog {
    FromCkbLog{ status: &mut FromCkbLogStatus::UnKnow }
}

impl<'a> CliSubCommand for CkbBridgeSubCommand<'a> {
    fn process(&mut self, matches: &ArgMatches, debug: bool) -> Result<Output, String> {
        match matches.subcommand() {
            ("transfer-erc20-to-ckb", Some(m)) => {
                self.transfer_to_ckb()
            }
            ("transfer-erc20-from-ckb", Some(m)) => {
                self.transfer_from_ckb()
            }
            ("deploy-sol", Some(m)) => {
                self.deploy_sol()
            }
            ("deploy-ckb", Some(m)) => {
                self.deploy_ckb()
            }
            _ => Err(Self::subcommand().generate_usage()),
        }
    }
}