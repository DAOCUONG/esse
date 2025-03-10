use std::sync::Arc;
use tdn::types::{
    group::GroupId,
    primitive::{new_io_error, HandleResult, PeerAddr},
    rpc::{json, rpc_response, RpcHandler, RpcParam},
};

use crate::group::GroupEvent;
use crate::rpc::RpcState;
use crate::storage::consensus_db;
use crate::utils::device_status::device_status as local_device_status;

use super::Device;

#[inline]
pub(crate) fn device_create(mgid: GroupId, device: &Device) -> RpcParam {
    rpc_response(0, "device-create", json!(device.to_rpc()), mgid)
}

#[inline]
pub(crate) fn _device_remove(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "device-remove", json!([id]), mgid)
}

#[inline]
pub(crate) fn device_online(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "device-online", json!([id]), mgid)
}

#[inline]
pub(crate) fn device_offline(mgid: GroupId, id: i64) -> RpcParam {
    rpc_response(0, "device-offline", json!([id]), mgid)
}

#[inline]
pub(crate) fn device_status(
    mgid: GroupId,
    cpu: u32,
    memory: u32,
    swap: u32,
    disk: u32,
    cpu_p: u16,
    memory_p: u16,
    swap_p: u16,
    disk_p: u16,
    uptime: u32,
) -> RpcParam {
    rpc_response(
        0,
        "device-status",
        json!([cpu, memory, swap, disk, cpu_p, memory_p, swap_p, disk_p, uptime]),
        mgid,
    )
}

#[inline]
fn device_list(devices: Vec<Device>) -> RpcParam {
    let mut results = vec![];
    for device in devices {
        results.push(device.to_rpc());
    }
    json!(results)
}

pub(crate) fn new_rpc_handler(handler: &mut RpcHandler<RpcState>) {
    handler.add_method("device-echo", |_, params, _| async move {
        Ok(HandleResult::rpc(json!(params)))
    });

    handler.add_method(
        "device-list",
        |gid: GroupId, _params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let db = consensus_db(state.layer.read().await.base(), &gid)?;
            let devices = Device::all(&db)?;
            drop(db);
            let online_devices = state.group.read().await.online_devices(&gid, devices);
            Ok(HandleResult::rpc(device_list(online_devices)))
        },
    );

    handler.add_method(
        "device-status",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let addr = PeerAddr::from_hex(params[0].as_str()?)
                .map_err(|_e| new_io_error("PeerAddr invalid!"))?;

            let group_lock = state.group.read().await;
            if &addr == group_lock.addr() {
                let uptime = group_lock.uptime(&gid)?;
                let (cpu, memory, swap, disk, cpu_p, memory_p, swap_p, disk_p) =
                    local_device_status();
                return Ok(HandleResult::rpc(json!([
                    cpu, memory, swap, disk, cpu_p, memory_p, swap_p, disk_p, uptime
                ])));
            }
            drop(group_lock);

            let msg = state
                .group
                .write()
                .await
                .event_message(addr, &GroupEvent::StatusRequest)?;

            Ok(HandleResult::group(gid, msg))
        },
    );

    handler.add_method(
        "device-create",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let addr = PeerAddr::from_hex(params[0].as_str()?)
                .map_err(|_e| new_io_error("PeerAddr invalid!"))?;

            let msg = state.group.read().await.create_message(&gid, addr)?;
            Ok(HandleResult::group(gid, msg))
        },
    );

    handler.add_method(
        "device-connect",
        |gid: GroupId, params: Vec<RpcParam>, state: Arc<RpcState>| async move {
            let addr = PeerAddr::from_hex(params[0].as_str()?)
                .map_err(|_e| new_io_error("PeerAddr invalid!"))?;

            let msg = state.group.read().await.connect_message(&gid, addr)?;
            Ok(HandleResult::group(gid, msg))
        },
    );

    handler.add_method(
        "device-delete",
        |_gid: GroupId, params: Vec<RpcParam>, _state: Arc<RpcState>| async move {
            let _id = params[0].as_i64()?;
            // TODO delete a device.
            Ok(HandleResult::new())
        },
    );
}
