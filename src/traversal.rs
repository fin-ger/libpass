use std::iter::Skip;

use id_tree::{LevelOrderTraversalIds, PostOrderTraversalIds, PreOrderTraversalIds, Tree};

use crate::{Entry, PassNode};

pub enum TraversalOrder {
    LevelOrder,
    PostOrder,
    PreOrder,
}

enum InnerRecursiveTraversal<'a> {
    LevelOrder(Skip<LevelOrderTraversalIds<'a, PassNode>>),
    PostOrder(Skip<PostOrderTraversalIds>),
    PreOrder(Skip<PreOrderTraversalIds<'a, PassNode>>),
}

pub struct RecursiveTraversal<'a> {
    iter: InnerRecursiveTraversal<'a>,
    tree: &'a Tree<PassNode>,
}

impl<'a> RecursiveTraversal<'a> {
    pub(crate) fn new(tree: &'a Tree<PassNode>, order: TraversalOrder) -> Self {
        let root_id = tree
            .root_node_id()
            .expect("Failed to retrieve root node of internal tree")
            .clone();

        let iter = match order {
            TraversalOrder::LevelOrder => InnerRecursiveTraversal::LevelOrder(
                tree.traverse_level_order_ids(&root_id)
                    .expect("Failed to traverse level order on the internal tree")
                    .skip(1),
            ),
            TraversalOrder::PostOrder => InnerRecursiveTraversal::PostOrder(
                tree.traverse_post_order_ids(&root_id)
                    .expect("Failed to traverse post order on the internal tree")
                    .skip(1),
            ),
            TraversalOrder::PreOrder => InnerRecursiveTraversal::PreOrder(
                tree.traverse_pre_order_ids(&root_id)
                    .expect("Failed to traverse pre order on the internal tree")
                    .skip(1),
            ),
        };

        Self { iter, tree }
    }
}

impl<'a> Iterator for RecursiveTraversal<'a> {
    type Item = Entry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let node_id = match self.iter {
            InnerRecursiveTraversal::LevelOrder(ref mut t) => t.next(),
            InnerRecursiveTraversal::PostOrder(ref mut t) => t.next(),
            InnerRecursiveTraversal::PreOrder(ref mut t) => t.next(),
        }?;

        Some(Entry::new(node_id, self.tree))
    }
}
