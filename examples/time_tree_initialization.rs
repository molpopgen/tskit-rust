use std::time::Instant;

use clap::Parser;
use tskit::prelude::*;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    treefile: String,
    #[clap(short, long, value_parser, default_value = "10")]
    stepsize: u64,
}

fn compare(tree: u64, name: &str, left: &[NodeId], right: &[NodeId]) {
    for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        if *l != *r {
            panic!(
                "tree {}, array: {}, index {}, left {}, right {}",
                tree, name, i, *l, *r
            );
        }
    }
}

fn main() {
    let args = Args::parse();

    assert!(args.stepsize > 0);

    let treeseq = tskit::TreeSequence::load(&args.treefile).unwrap();
    let num_trees: u64 = treeseq.num_trees().into();
    let flags = tskit::TreeFlags::SAMPLE_LISTS;
    let indexes = tskit::TreesIndex::new(&treeseq).unwrap();

    for i in (0..num_trees).step_by(args.stepsize as usize) {
        assert!(i < num_trees);
        let now = Instant::now();
        let tree_at = treeseq
            .tree_iterator_at_index(i.into(), &indexes, flags)
            .unwrap();
        let duration = now.elapsed();
        let now = Instant::now();
        let tree_at_lib = treeseq
            .tree_iterator_at_index_lib(i.into(), &indexes, flags)
            .unwrap();
        let duration_lib = now.elapsed();
        println!(
            "{} {:?} {:?}",
            i,
            duration.as_micros(),
            duration_lib.as_micros()
        );

        compare(
            i,
            "parent",
            tree_at.parent_array(),
            tree_at_lib.parent_array(),
        );
        compare(
            i,
            "left_child",
            tree_at.left_child_array(),
            tree_at_lib.left_child_array(),
        );
        compare(
            i,
            "right_child",
            tree_at.right_child_array(),
            tree_at_lib.right_child_array(),
        );
        compare(
            i,
            "left_sib",
            tree_at.left_sib_array(),
            tree_at_lib.left_sib_array(),
        );
        compare(
            i,
            "right_sib",
            tree_at.right_sib_array(),
            tree_at_lib.right_sib_array(),
        );
    }
}
