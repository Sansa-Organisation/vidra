use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Clock {
    pub client_id: String,
    pub counter: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CursorPosition {
    Node(String), // The path to the node they are currently selecting/editing
    TextOffset(String, usize), // For text editing logic
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Presence {
    pub client_id: String,
    pub avatar_url: Option<String>,
    pub color: String,
    pub cursor: Option<CursorPosition>,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CrdtOperation {
    InsertNode {
        parent_id: String,
        node_id: String,
        node_data: serde_json::Value,
        index: Option<usize>,
    },
    DeleteNode {
        node_id: String,
    },
    UpdateProperty {
        node_id: String,
        key: String,
        value: serde_json::Value,
    },
    MoveNode {
        node_id: String,
        new_parent_id: String,
        index: Option<usize>,
    },
    PresenceUpdate(Presence),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncMessage {
    pub clock: Clock,
    pub operations: Vec<CrdtOperation>,
}

impl SyncMessage {
    pub fn new(client_id: String, counter: u64, operations: Vec<CrdtOperation>) -> Self {
        Self {
            clock: Clock {
                client_id,
                counter,
            },
            operations,
        }
    }
}
