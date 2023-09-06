use sbor::rust::collections::HashMap;
use sbor::rust::vec::Vec;

use crate::hash_tree::types::{LeafKey, Version};

use super::tree_store::{
    ReadableTreeStore, TreeChildEntry, TreeInternalNode, TreeLeafNode, TreeNode,
};
use super::types::{
    Child, InternalNode, LeafNode, Nibble, NibblePath, Node, NodeKey, NodeType, StorageError,
    TreeReader,
};

impl TreeInternalNode {
    fn from(internal_node: &InternalNode) -> Self {
        let children = internal_node
            .children_sorted()
            .map(|(nibble, child)| TreeChildEntry {
                nibble: nibble.clone(),
                version: child.version,
                hash: child.hash,
                is_leaf: child.is_leaf(),
            })
            .collect::<Vec<TreeChildEntry>>();
        TreeInternalNode { children }
    }
}

impl TreeLeafNode {
    fn from(key: &NodeKey, leaf_node: &LeafNode<Version>) -> Self {
        TreeLeafNode {
            key_suffix: NibblePath::from_iter(
                NibblePath::new_even(leaf_node.leaf_key().bytes.clone())
                    .nibbles()
                    .skip(key.nibble_path().num_nibbles()),
            ),
            value_hash: leaf_node.value_hash(),
            last_hash_change_version: leaf_node.payload().clone(),
        }
    }
}

impl TreeNode {
    pub fn from(key: &NodeKey, node: &Node<Version>) -> Self {
        match node {
            Node::Internal(internal_node) => {
                TreeNode::Internal(TreeInternalNode::from(internal_node))
            }
            Node::Leaf(leaf_node) => TreeNode::Leaf(TreeLeafNode::from(key, leaf_node)),
            Node::Null => TreeNode::Null,
        }
    }
}

impl InternalNode {
    fn from(internal_node: &TreeInternalNode) -> Self {
        let child_map: HashMap<Nibble, Child> = internal_node
            .children
            .iter()
            .map(|child_entry| {
                let child: Child = Child::new(
                    child_entry.hash,
                    child_entry.version,
                    if child_entry.is_leaf {
                        NodeType::Leaf
                    } else {
                        // Note: the `0` passed here may be replaced with an actual value (which we
                        // would have to persist) once we have use-cases for quick look-ups of leaf
                        // counts.
                        NodeType::Internal { leaf_count: 0 }
                    },
                );
                (child_entry.nibble, child)
            })
            .collect();
        InternalNode::new(child_map)
    }
}

impl Node<Version> {
    fn from(key: &NodeKey, tree_node: &TreeNode) -> Self {
        match tree_node {
            TreeNode::Internal(internal_node) => Node::Internal(InternalNode::from(internal_node)),
            TreeNode::Leaf(leaf_node) => Node::Leaf(LeafNode::from(key, leaf_node)),
            TreeNode::Null => Node::Null,
        }
    }
}

impl LeafNode<Version> {
    pub fn from(key: &NodeKey, leaf_node: &TreeLeafNode) -> Self {
        let full_key = NibblePath::from_iter(
            key.nibble_path()
                .nibbles()
                .chain(leaf_node.key_suffix.nibbles()),
        );
        LeafNode::new(
            LeafKey::new(full_key.bytes()),
            leaf_node.value_hash,
            leaf_node.last_hash_change_version,
            key.version(),
        )
    }
}

impl<R: ReadableTreeStore> TreeReader<Version> for R {
    fn get_node_option(&self, node_key: &NodeKey) -> Result<Option<Node<Version>>, StorageError> {
        Ok(self
            .get_node(node_key)
            .map(|tree_node| Node::from(node_key, &tree_node)))
    }
}
