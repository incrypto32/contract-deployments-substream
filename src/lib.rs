mod pb;
mod tables;

use pb::{test::{Contract, Contracts}, substreams::entity::v1::entity_change};

use substreams::{Hex, log};
use substreams_entity_change::pb::entity::EntityChanges;
use substreams_ethereum::pb::eth;

use tables::Tables;

#[substreams::handlers::map]
fn map_contract(block: eth::v2::Block) -> Result<Contracts, substreams::errors::Error> {
    let contracts = block
        .transactions()
        .flat_map(|tx| {
            tx.calls
                .iter()
                .filter(|call| call.call_type == 5)
                .map(|call| Contract {
                    address: format!("0x{}", Hex(&call.address).to_string()),
                    creation_block_number: block.number,
                    creation_txn_hash: format!("0x{}", Hex::encode(&tx.hash).to_string()),
                    timestamp: block.timestamp_seconds().to_string(),
                    ordinal: tx.begin_ordinal,
                })
        })
        .collect();
    log::info!("Found {:?}", contracts);
    Ok(Contracts { contracts })
}

#[substreams::handlers::map]
pub fn graph_out(contracts: Contracts) -> Result<EntityChanges, substreams::errors::Error> {
    // hash map of name to a table
    let mut tables = Tables::new();

    for contract in &contracts.contracts {
        let row = tables.create_row("Contract", &contract.address);
        row.set("timestamp", &contract.timestamp);
        row.set("creation_block_number", contract.creation_block_number);
        row.set("creation_txn_hash", &contract.creation_txn_hash);
    }
    let entity_changes = tables.to_entity_changes();
    log::info!("Entity changes: {:?}", entity_changes);
    Ok(entity_changes)
}
