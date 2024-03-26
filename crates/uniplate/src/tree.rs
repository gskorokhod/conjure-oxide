//#![cfg(feature = "unstable")]

use std::{iter::zip, sync::Arc};

use im::vector;
use proptest::prelude::*;
use proptest_derive::Arbitrary;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tree<T: Sized + Clone + Eq> {
    Zero,
    One(T),
    Many(im::Vector<Tree<T>>),
}

use self::Tree::*;

impl<T: Sized + Clone + Eq + 'static> Tree<T> {
    /// Returns the tree as a list alongside a function to reconstruct the tree from a list.
    ///
    /// This preserves the structure of the tree.
    pub fn list(self) -> (im::Vector<T>, Box<dyn Fn(im::Vector<T>) -> Tree<T>>) {
        // inspired by the Uniplate Haskell equivalent Data.Generics.Str::strStructure
        // https://github.com/ndmitchell/uniplate/blob/master/Data/Generics/Str.hs#L85

        fn flatten<T: Sized + Clone + Eq>(t: Tree<T>, xs: im::Vector<T>) -> im::Vector<T> {
            match (t, xs) {
                (Zero, xs) => xs,
                (One(x), xs) => {
                    let mut xs1 = xs.clone();
                    xs1.push_front(x);
                    xs1
                }
                (Many(ts), xs) => ts.into_iter().fold(xs, |xs, t| flatten(t, xs)),
            }
        }

        // Iterate over both the old tree and the new list.
        // We use the node types of the old tree to know what node types to use for the new tree.
        fn recons<T: Sized + Clone + Eq>(
            old_tree: Tree<T>,
            xs: im::Vector<T>,
        ) -> (Tree<T>, im::Vector<T>) {
            #[allow(clippy::unwrap_used)]
            match (old_tree, xs) {
                (Zero, xs) => (Zero, xs),
                (One(_), xs) => {
                    let mut xs1 = xs.clone();
                    (One(xs1.pop_back().unwrap()), xs1)
                }
                (Many(ts), xs) => {
                    let (ts1, xs1) = ts.into_iter().fold((vector![], xs), |(ts1, xs), t| {
                        let (t1, xs1) = recons(t, xs);
                        let mut ts2 = ts1.clone();
                        ts2.push_back(t1);
                        (ts2, xs1)
                    });
                    (Many(ts1), xs1)
                }
            }
        }
        (
            flatten(self.clone(), vector![]),
            Box::new(move |xs| recons(self.clone(), xs).0),
        )
    }

    // Perform a map over all elements in the tree.
    fn map(self, op: Arc<dyn Fn(T) -> T>) -> Tree<T> {
        match (self) {
            Zero => Zero,
            One(t) => One(op(t)),
            Many(ts) => Many(ts.into_iter().map(|t| t.map(op.clone())).collect()),
        }
    }
}

#[allow(dead_code)]
// Used by proptest for generating test instances of Tree<i32>.
fn proptest_integer_trees() -> impl Strategy<Value = Tree<i32>> {
    // https://proptest-rs.github.io/proptest/proptest/tutorial/enums.html
    // https://proptest-rs.github.io/proptest/proptest/tutorial/recursive.html
    let leaf = prop_oneof![Just(Tree::Zero), any::<i32>().prop_map(Tree::One),];

    leaf.prop_recursive(
        10,  // levels deep
        512, // Shoot for maximum size of 512 nodes
        20,  // We put up to 20 items per collection
        |inner| prop::collection::vec(inner.clone(), 0..20).prop_map(|vec| Tree::Many(vec.into())),
    )
}

proptest! {
    #[test]
    // Is tree.recons() isomorphic?
    fn tree_list_isomorphic_ints(tree in proptest_integer_trees()) {
        let (children,func) = tree.clone().list();
        let new_tree = func(children);
        prop_assert_eq!(new_tree,tree);
    }

    #[test]
    fn tree_map_add(tree in proptest_integer_trees(), diff in -100i32..100i32) {
        let new_tree = tree.clone().map(Arc::new(move |a| a+diff));
        let (old_children,_) = tree.list();
        let (new_children,_) = new_tree.list();

        for (old,new) in zip(old_children,new_children) {
            prop_assert_eq!(old+diff,new);
        }
    }
}
