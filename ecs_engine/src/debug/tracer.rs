use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::time;

pub struct Debug_Tracer {
    // Tree of Debug_Tracer_Nodes representing the call tree.
    saved_traces: Vec<Debug_Tracer_Node>,
    running_traces: Vec<Debug_Tracer_Node>,
}

/// Represents a traced scope with its info and a link to its parent.
#[derive(Debug)]
pub struct Debug_Tracer_Node {
    pub info: Debug_Scope_Trace_Info,
    pub parent_idx: Option<usize>,
}

/// The actual trace information for a single scope.
#[derive(Clone)]
pub struct Debug_Scope_Trace_Info {
    start_t: time::Instant,
    end_t: time::Instant,
    pub tag: &'static str,
}

impl Debug_Scope_Trace_Info {
    pub fn duration(&self) -> time::Duration {
        self.end_t.duration_since(self.start_t)
    }
}

impl Debug for Debug_Scope_Trace_Info {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}: {:?}]",
            self.tag,
            self.end_t.duration_since(self.start_t)
        )
    }
}

/// This is used to automatically add a Trace_Info to the Tracer via RAII.
pub struct Debug_Scope_Trace {
    tracer: Rc<RefCell<Debug_Tracer>>,
}

impl Drop for Debug_Scope_Trace {
    fn drop(&mut self) {
        self.tracer.borrow_mut().pop_scope_trace();
    }
}

#[derive(Clone, Debug)]
pub struct Trace_Tree<'a> {
    pub node: &'a Debug_Tracer_Node,
    pub children: Vec<Trace_Tree<'a>>,
}

impl Trace_Tree<'_> {
    pub fn new(node: &Debug_Tracer_Node) -> Trace_Tree {
        Trace_Tree {
            node,
            children: vec![],
        }
    }
}

impl Debug_Tracer {
    pub fn new() -> Debug_Tracer {
        Debug_Tracer {
            saved_traces: vec![],
            running_traces: vec![],
        }
    }

    pub fn push_scope_trace(&mut self, tag: &'static str) {
        let now = time::Instant::now();
        let cur_running_traces = self.running_traces.len();
        self.running_traces.push(Debug_Tracer_Node {
            info: Debug_Scope_Trace_Info {
                start_t: now,
                end_t: now,
                tag,
            },
            parent_idx: if cur_running_traces == 0 {
                None
            } else {
                Some(cur_running_traces - 1)
            },
        });
    }

    pub fn pop_scope_trace(&mut self) {
        let now = time::Instant::now();
        let mut active_node = self
            .running_traces
            .pop()
            .expect("Called pop_scope_trace without an active node!");
        active_node.info.end_t = now;
        self.saved_traces.push(active_node);
    }

    pub fn start_frame(&mut self) {
        self.running_traces.clear();
        self.saved_traces.clear();
    }

    pub fn debug_print(&self) {
        for node in &self.saved_traces {
            println!(
                "{:?} -> {:?}",
                node.info,
                node.parent_idx.map(|idx| self.saved_traces[idx].info.tag)
            );
        }
    }

    pub fn total_traced_time(&self) -> time::Duration {
        self.saved_traces
            .iter()
            .filter_map(|node| {
                if node.parent_idx.is_none() {
                    Some(node.info.end_t.duration_since(node.info.start_t))
                } else {
                    None
                }
            })
            .fold(time::Duration::default(), |acc, x| acc + x)
    }

    pub fn sort(&mut self) {
        self.saved_traces
            .sort_by(|a, b| b.info.duration().cmp(&a.info.duration()));
    }

    /// Construct a forest of Trace_Trees from the saved_traces array.
    pub fn get_trace_trees(&self) -> Vec<Trace_Tree<'_>> {
        let mut forest = vec![];

        if self.saved_traces.is_empty() {
            return forest;
        }
        println!("-----");
        for node in &self.saved_traces {
            println!("{:?}", node);
        }
        // Note: we exploit the fact that saved_traces elements are ordered as
        // a reversed tree. i.e.:
        //
        //      A
        //    /   \
        //   B     C
        //  / \     \
        // D   E     F
        //
        // saved_traces = [F, C, E, D, B, A]
        //

        // First, split saved_traces into multiple arrays, each containing one tree.
        let mut trees = vec![];
        {
            let mut i = 0;
            for node in &self.saved_traces {
                if trees.len() <= i {
                    trees.push(vec![]);
                }
                trees[i].push(Some(Trace_Tree::new(node)));
                if node.parent_idx.is_none() {
                    i += 1;
                }
            }
        }

        // Then, for each tree, fill the `children` vecs.
        {
            let mut traces_idx = 0;
            let mut tree_idx = 0;
            let mut subtree_idx = 0;

            loop {
                let tree = &mut trees[tree_idx];
                let tlen = tree.len();
                let node = &self.saved_traces[traces_idx];
                if let Some(p_idx) = node.parent_idx {
                    let subtree = tree[subtree_idx].take();
                    println!(
                        "taking [{}]([{}] ->  [{}])",
                        tree_idx,
                        subtree_idx,
                        tlen - 1 - p_idx
                    );
                    tree[tlen - 1 - p_idx]
                        .as_mut()
                        .unwrap()
                        .children
                        .push(subtree.unwrap());
                }

                if traces_idx == self.saved_traces.len() - 1 {
                    break;
                }

                traces_idx += 1;

                if subtree_idx == tree.len() - 1 {
                    assert!(tree_idx < trees.len() - 1);
                    tree_idx += 1;
                    subtree_idx = 0;
                } else {
                    subtree_idx += 1;
                }
            }
        }

        for mut tree in trees {
            let tlen = tree.len() - 1;
            forest.push(tree[tlen].take().unwrap());
        }

        forest
    }
}

#[inline]
pub fn debug_trace(tag: &'static str, tracer: Rc<RefCell<Debug_Tracer>>) -> Debug_Scope_Trace {
    tracer.borrow_mut().push_scope_trace(tag);
    Debug_Scope_Trace { tracer }
}

pub fn sort_trace_trees(trees: &mut [Trace_Tree]) {
    fn sort_tree_internal(tree: &mut Trace_Tree) {
        tree.children
            .sort_by(|a, b| b.node.info.duration().cmp(&a.node.info.duration()));
        for c in &mut tree.children {
            sort_tree_internal(c);
        }
    }

    for tree in trees {
        sort_tree_internal(tree);
    }
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! trace {
    ($tag: expr, $tracer: expr) => {
        let _trace_var = debug_trace($tag, $tracer.clone());
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! trace {
    ($tag: expr, $ng_state: expr) => {};
}
