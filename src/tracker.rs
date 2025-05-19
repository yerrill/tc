use crate::metainfo::*;
use reqwest::get;

pub async fn tracker_get(metainfo: Meta, peer_id: &str) -> Result<(), ()> {

    let info_hash = metainfo.info_hash().iter().map(|b| format!("%{:X}", b)).collect::<String>();
    //let body = get()
    //let query_string = format!("?info_hash{}&peer_id{}", metainfo.info_hash(), peer_id);

    Ok(())
}
