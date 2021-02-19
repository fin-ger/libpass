use id_tree::{LevelOrderTraversalIds, NodeId, PostOrderTraversalIds, PreOrderTraversalIds, Tree};

use crate::{Entry, PassNode};

pub enum TraversalOrder {
    LevelOrder,
    PostOrder,
    PreOrder,
}

enum EntriesTraversal<'a> {
    LevelOrder(LevelOrderTraversalIds<'a, PassNode>),
    PostOrder(PostOrderTraversalIds),
    PreOrder(PreOrderTraversalIds<'a, PassNode>),
}

pub struct Entries<'a> {
    iter: EntriesTraversal<'a>,
    tree: &'a Tree<PassNode>,
}

impl<'a> Entries<'a> {
    pub(crate) fn new(
        tree: &'a Tree<PassNode>,
        node_id: &'a NodeId,
        order: TraversalOrder,
    ) -> Self {
        let iter = match order {
            TraversalOrder::LevelOrder => EntriesTraversal::LevelOrder(
                tree.traverse_level_order_ids(node_id)
                    .expect("Failed to traverse level order on the internal tree"),
            ),
            TraversalOrder::PostOrder => EntriesTraversal::PostOrder(
                tree.traverse_post_order_ids(node_id)
                    .expect("Failed to traverse post order on the internal tree"),
            ),
            TraversalOrder::PreOrder => EntriesTraversal::PreOrder(
                tree.traverse_pre_order_ids(node_id)
                    .expect("Failed to traverse pre order on the internal tree"),
            ),
        };

        Self { iter, tree }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let node_id = match self.iter {
            EntriesTraversal::LevelOrder(ref mut t) => t.next(),
            EntriesTraversal::PostOrder(ref mut t) => t.next(),
            EntriesTraversal::PreOrder(ref mut t) => t.next(),
        }?;

        let data = self
            .tree
            .get(&node_id)
            .expect("node id for node which must exists not found")
            .data()
            .clone();
        Some(Entry::new(node_id, data))
    }
}
