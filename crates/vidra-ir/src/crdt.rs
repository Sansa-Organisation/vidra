use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Clock {
    pub client_id: String,
    pub counter: u64,
}

impl Clock {
    /// Deterministic total ordering for conflict resolution.
    ///
    /// Higher counter wins; ties broken by lexicographic `client_id`.
    pub fn cmp_lww(a: &Clock, b: &Clock) -> std::cmp::Ordering {
        match a.counter.cmp(&b.counter) {
            std::cmp::Ordering::Equal => a.client_id.cmp(&b.client_id),
            other => other,
        }
    }

    pub fn gt_lww(&self, other: &Clock) -> bool {
        Self::cmp_lww(self, other) == std::cmp::Ordering::Greater
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CursorPosition {
    Node(String),              // The path to the node they are currently selecting/editing
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
            clock: Clock { client_id, counter },
            operations,
        }
    }
}

#[derive(Debug, Error)]
pub enum CrdtError {
    #[error("root node must not have a parent")]
    RootHasParent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtNode {
    pub node_id: String,
    pub node_data: serde_json::Value,
    pub parent_id: Option<String>,
    pub index_hint: Option<usize>,

    pub deleted: bool,

    #[serde(skip)]
    delete_clock: Option<Clock>,
    #[serde(skip)]
    position_clock: Option<Clock>,
    #[serde(skip)]
    property_clocks: HashMap<String, Clock>,
}

impl CrdtNode {
    fn new(node_id: String, node_data: serde_json::Value) -> Self {
        Self {
            node_id,
            node_data,
            parent_id: None,
            index_hint: None,
            deleted: false,
            delete_clock: None,
            position_clock: None,
            property_clocks: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MessageId {
    client_id: String,
    counter: u64,
}

/// A minimal, deterministic CRDT-like document for real-time collaboration.
///
/// It implements:
/// - LWW registers for properties (`UpdateProperty`)
/// - LWW for node position (`MoveNode`)
/// - tombstoning with optional revive via `InsertNode` if newer
///
/// This is intentionally generic (`serde_json::Value`) so it can be used for
/// multiplayer editing and/or producing IR patches.
#[derive(Debug, Clone)]
pub struct CrdtDocument {
    pub root_id: String,
    pub nodes: HashMap<String, CrdtNode>,
    pub presence: HashMap<String, Presence>,

    seen_messages: HashSet<MessageId>,
}

impl CrdtDocument {
    pub fn new(root_id: impl Into<String>) -> Result<Self, CrdtError> {
        let root_id = root_id.into();
        let mut nodes = HashMap::new();
        let mut root = CrdtNode::new(
            root_id.clone(),
            serde_json::Value::Object(serde_json::Map::new()),
        );
        root.parent_id = None;
        nodes.insert(root_id.clone(), root);

        Ok(Self {
            root_id,
            nodes,
            presence: HashMap::new(),
            seen_messages: HashSet::new(),
        })
    }

    pub fn get_node(&self, node_id: &str) -> Option<&CrdtNode> {
        self.nodes.get(node_id)
    }

    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut CrdtNode> {
        self.nodes.get_mut(node_id)
    }

    pub fn apply_message(&mut self, message: &SyncMessage) {
        let message_id = MessageId {
            client_id: message.clock.client_id.clone(),
            counter: message.clock.counter,
        };
        if !self.seen_messages.insert(message_id) {
            return;
        }

        for op in &message.operations {
            self.apply_operation_with_clock(op, &message.clock);
        }
    }

    pub fn apply_messages<I: IntoIterator<Item = SyncMessage>>(&mut self, messages: I) {
        let mut messages: Vec<SyncMessage> = messages.into_iter().collect();
        messages.sort_by(|a, b| Clock::cmp_lww(&a.clock, &b.clock));
        for message in &messages {
            self.apply_message(message);
        }
    }

    pub fn apply_operation(&mut self, operation: &CrdtOperation, clock: Clock) {
        self.apply_operation_with_clock(operation, &clock);
    }

    fn apply_operation_with_clock(&mut self, operation: &CrdtOperation, clock: &Clock) {
        match operation {
            CrdtOperation::InsertNode {
                parent_id,
                node_id,
                node_data,
                index,
            } => {
                let existing = self.nodes.get(node_id).cloned();
                match existing {
                    None => {
                        let mut node = CrdtNode::new(node_id.clone(), node_data.clone());
                        node.parent_id = Some(parent_id.clone());
                        node.index_hint = *index;
                        node.position_clock = Some(clock.clone());
                        self.nodes.insert(node_id.clone(), node);
                    }
                    Some(mut node) => {
                        if node.deleted {
                            // Allow revive only if this insert is newer than the delete.
                            let can_revive = node
                                .delete_clock
                                .as_ref()
                                .map(|d| clock.gt_lww(d))
                                .unwrap_or(true);
                            if !can_revive {
                                return;
                            }
                            node.deleted = false;
                            node.delete_clock = None;
                        }

                        // Keep the newer node_data (if desired), but at minimum ensure the node exists.
                        node.node_data = node_data.clone();

                        // Treat as a positioning update.
                        let should_move = node
                            .position_clock
                            .as_ref()
                            .map(|c| clock.gt_lww(c))
                            .unwrap_or(true);
                        if should_move {
                            node.parent_id = Some(parent_id.clone());
                            node.index_hint = *index;
                            node.position_clock = Some(clock.clone());
                        }
                        self.nodes.insert(node_id.clone(), node);
                    }
                }
            }
            CrdtOperation::DeleteNode { node_id } => {
                self.delete_subtree(node_id, clock);
            }
            CrdtOperation::UpdateProperty {
                node_id,
                key,
                value,
            } => {
                let Some(node) = self.nodes.get_mut(node_id) else {
                    return;
                };
                if node.deleted {
                    return;
                }
                if node_id == &self.root_id {
                    // Root is still mutable; this is allowed.
                }

                let should_apply = match node.property_clocks.get(key) {
                    None => true,
                    Some(prev) => clock.gt_lww(prev),
                };
                if !should_apply {
                    return;
                }

                let object = match node.node_data.as_object_mut() {
                    Some(obj) => obj,
                    None => {
                        node.node_data = serde_json::Value::Object(serde_json::Map::new());
                        node.node_data.as_object_mut().expect("just set to object")
                    }
                };
                object.insert(key.clone(), value.clone());
                node.property_clocks.insert(key.clone(), clock.clone());
            }
            CrdtOperation::MoveNode {
                node_id,
                new_parent_id,
                index,
            } => {
                let Some(node) = self.nodes.get_mut(node_id) else {
                    return;
                };
                if node.deleted {
                    return;
                }
                if node_id == &self.root_id {
                    // Root cannot be moved.
                    return;
                }

                let should_move = node
                    .position_clock
                    .as_ref()
                    .map(|c| clock.gt_lww(c))
                    .unwrap_or(true);
                if !should_move {
                    return;
                }

                node.parent_id = Some(new_parent_id.clone());
                node.index_hint = *index;
                node.position_clock = Some(clock.clone());
            }
            CrdtOperation::PresenceUpdate(p) => {
                let entry = self.presence.entry(p.client_id.clone());
                match entry {
                    std::collections::hash_map::Entry::Vacant(v) => {
                        v.insert(p.clone());
                    }
                    std::collections::hash_map::Entry::Occupied(mut o) => {
                        if p.timestamp >= o.get().timestamp {
                            o.insert(p.clone());
                        }
                    }
                }
            }
        }
    }

    fn delete_subtree(&mut self, node_id: &str, clock: &Clock) {
        if node_id == self.root_id {
            // Root is never deleted.
            return;
        }

        let mut stack = vec![node_id.to_string()];
        while let Some(current_id) = stack.pop() {
            let can_delete = match self.nodes.get(&current_id) {
                None => false,
                Some(node) => match node.delete_clock.as_ref() {
                    None => true,
                    Some(prev) => clock.gt_lww(prev),
                },
            };
            if !can_delete {
                continue;
            }

            if let Some(node) = self.nodes.get_mut(&current_id) {
                node.deleted = true;
                node.delete_clock = Some(clock.clone());
            }

            // Find children by scanning; avoids needing a second index.
            let children: Vec<String> = self
                .nodes
                .values()
                .filter(|n| n.parent_id.as_deref() == Some(&current_id) && !n.deleted)
                .map(|n| n.node_id.clone())
                .collect();
            for child in children {
                stack.push(child);
            }
        }
    }

    /// Reconstructs a deterministic tree snapshot starting from `root_id`.
    pub fn export_tree(&self) -> serde_json::Value {
        let mut visited = HashSet::new();
        self.export_node_recursive(&self.root_id, &mut visited)
            .unwrap_or_else(|| serde_json::json!({ "node_id": self.root_id, "deleted": false, "data": {}, "children": [] }))
    }

    fn export_node_recursive(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
    ) -> Option<serde_json::Value> {
        if !visited.insert(node_id.to_string()) {
            return None;
        }
        let node = self.nodes.get(node_id)?;
        if node.deleted {
            return None;
        }

        let mut children: Vec<&CrdtNode> = self
            .nodes
            .values()
            .filter(|n| n.parent_id.as_deref() == Some(node_id) && !n.deleted)
            .collect();

        children.sort_by(|a, b| {
            let ia = a.index_hint.unwrap_or(usize::MAX);
            let ib = b.index_hint.unwrap_or(usize::MAX);
            match ia.cmp(&ib) {
                std::cmp::Ordering::Equal => {
                    let clock_cmp = match (a.position_clock.as_ref(), b.position_clock.as_ref()) {
                        (None, None) => std::cmp::Ordering::Equal,
                        (None, Some(_)) => std::cmp::Ordering::Less,
                        (Some(_), None) => std::cmp::Ordering::Greater,
                        (Some(ca), Some(cb)) => Clock::cmp_lww(ca, cb),
                    };
                    match clock_cmp {
                        std::cmp::Ordering::Equal => a.node_id.cmp(&b.node_id),
                        other => other,
                    }
                }
                other => other,
            }
        });

        let children_json: Vec<serde_json::Value> = children
            .iter()
            .filter_map(|c| self.export_node_recursive(&c.node_id, visited))
            .collect();

        Some(serde_json::json!({
            "node_id": node.node_id,
            "data": node.node_data,
            "children": children_json,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(client: &str, counter: u64, ops: Vec<CrdtOperation>) -> SyncMessage {
        SyncMessage::new(client.to_string(), counter, ops)
    }

    #[test]
    fn lww_property_updates_commute_on_tie_counter() {
        let mut doc = CrdtDocument::new("root").unwrap();
        doc.apply_message(&msg(
            "a",
            1,
            vec![CrdtOperation::InsertNode {
                parent_id: "root".to_string(),
                node_id: "n1".to_string(),
                node_data: serde_json::json!({}),
                index: Some(0),
            }],
        ));

        let m1 = msg(
            "a",
            2,
            vec![CrdtOperation::UpdateProperty {
                node_id: "n1".to_string(),
                key: "x".to_string(),
                value: serde_json::json!(10),
            }],
        );
        let m2 = msg(
            "b",
            2,
            vec![CrdtOperation::UpdateProperty {
                node_id: "n1".to_string(),
                key: "x".to_string(),
                value: serde_json::json!(20),
            }],
        );

        let mut doc_a = doc.clone();
        doc_a.apply_message(&m1);
        doc_a.apply_message(&m2);

        let mut doc_b = doc.clone();
        doc_b.apply_message(&m2);
        doc_b.apply_message(&m1);

        let xa = doc_a.get_node("n1").unwrap().node_data["x"].clone();
        let xb = doc_b.get_node("n1").unwrap().node_data["x"].clone();
        assert_eq!(xa, serde_json::json!(20));
        assert_eq!(xb, serde_json::json!(20));
    }

    #[test]
    fn lww_move_is_deterministic() {
        let mut base = CrdtDocument::new("root").unwrap();
        base.apply_message(&msg(
            "a",
            1,
            vec![
                CrdtOperation::InsertNode {
                    parent_id: "root".to_string(),
                    node_id: "n1".to_string(),
                    node_data: serde_json::json!({}),
                    index: Some(0),
                },
                CrdtOperation::InsertNode {
                    parent_id: "root".to_string(),
                    node_id: "n2".to_string(),
                    node_data: serde_json::json!({}),
                    index: Some(1),
                },
            ],
        ));

        let m1 = msg(
            "a",
            5,
            vec![CrdtOperation::MoveNode {
                node_id: "n1".to_string(),
                new_parent_id: "root".to_string(),
                index: Some(1),
            }],
        );
        let m2 = msg(
            "b",
            5,
            vec![CrdtOperation::MoveNode {
                node_id: "n1".to_string(),
                new_parent_id: "root".to_string(),
                index: Some(0),
            }],
        );

        let mut doc_a = base.clone();
        doc_a.apply_message(&m1);
        doc_a.apply_message(&m2);

        let mut doc_b = base.clone();
        doc_b.apply_message(&m2);
        doc_b.apply_message(&m1);

        let ta = doc_a.export_tree();
        let tb = doc_b.export_tree();
        assert_eq!(ta, tb);

        let children = ta["children"].as_array().unwrap();
        assert_eq!(children[0]["node_id"], serde_json::json!("n1"));
        assert_eq!(children[1]["node_id"], serde_json::json!("n2"));
    }

    #[test]
    fn applying_same_message_twice_is_idempotent() {
        let mut doc = CrdtDocument::new("root").unwrap();
        let m = msg(
            "a",
            1,
            vec![CrdtOperation::InsertNode {
                parent_id: "root".to_string(),
                node_id: "n1".to_string(),
                node_data: serde_json::json!({ "x": 1 }),
                index: Some(0),
            }],
        );
        doc.apply_message(&m);
        let once = doc.export_tree();
        doc.apply_message(&m);
        let twice = doc.export_tree();
        assert_eq!(once, twice);
    }

    #[test]
    fn delete_tombstones_and_blocks_property_updates() {
        let mut doc = CrdtDocument::new("root").unwrap();
        doc.apply_message(&msg(
            "a",
            1,
            vec![CrdtOperation::InsertNode {
                parent_id: "root".to_string(),
                node_id: "n1".to_string(),
                node_data: serde_json::json!({}),
                index: Some(0),
            }],
        ));

        doc.apply_message(&msg(
            "a",
            2,
            vec![CrdtOperation::DeleteNode {
                node_id: "n1".to_string(),
            }],
        ));

        doc.apply_message(&msg(
            "b",
            10,
            vec![CrdtOperation::UpdateProperty {
                node_id: "n1".to_string(),
                key: "x".to_string(),
                value: serde_json::json!(123),
            }],
        ));

        let tree = doc.export_tree();
        let children = tree["children"].as_array().unwrap();
        assert!(children.is_empty());
    }
}
