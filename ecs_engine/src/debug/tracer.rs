use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use std::time;

pub struct Tracer {
    // Tree of Tracer_Nodes representing the call tree.
    saved_traces: Vec<Tracer_Node>,
    // Latest pushed (and not-yet-popped) node index
    cur_active: Option<usize>,
}

/// Represents a traced scope with its info and a link to its parent.
#[derive(Debug)]
pub struct Tracer_Node {
    pub info: Scope_Trace_Info,
    pub parent_idx: Option<usize>,
}

/// The actual trace information for a single scope.
#[derive(Clone)]
pub struct Scope_Trace_Info {
    pub start_t: time::Instant,
    pub end_t: time::Instant,
    pub tag: &'static str,
    pub n_calls: usize,
}

impl Scope_Trace_Info {
    pub fn duration(&self) -> time::Duration {
        self.end_t.duration_since(self.start_t)
    }
}

impl Debug for Scope_Trace_Info {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}: {:?} (x{})]",
            self.tag,
            self.duration(),
            self.n_calls
        )
    }
}

/// This is used to automatically add a Trace_Info to the Tracer via RAII.
pub struct Scope_Trace {
    tracer: Rc<RefCell<Tracer>>,
}

impl Drop for Scope_Trace {
    fn drop(&mut self) {
        self.tracer.borrow_mut().pop_scope_trace();
    }
}

#[derive(Clone, Debug)]
pub struct Trace_Tree<'a> {
    pub node: &'a Tracer_Node,
    pub children: Vec<Trace_Tree<'a>>,
}

impl Trace_Tree<'_> {
    pub fn new(node: &Tracer_Node) -> Trace_Tree {
        Trace_Tree {
            node,
            children: vec![],
        }
    }
}

impl Tracer {
    pub fn new() -> Tracer {
        Tracer {
            saved_traces: vec![],
            cur_active: None,
        }
    }

    fn push_scope_trace(&mut self, tag: &'static str) {
        let now = time::Instant::now();
        self.saved_traces.push(Tracer_Node {
            info: Scope_Trace_Info {
                start_t: now,
                end_t: now,
                tag,
                n_calls: 1,
            },
            parent_idx: self.cur_active,
        });
        self.cur_active = Some(self.saved_traces.len() - 1);
    }

    fn pop_scope_trace(&mut self) {
        let now = time::Instant::now();
        let mut active_node = &mut self.saved_traces[self
            .cur_active
            .expect("[ ERROR ] Popped scope trace while none is active!")];
        active_node.info.end_t = now;
        self.cur_active = active_node.parent_idx;
    }

    /// Deduplicates tracer nodes and returns a reference to the final traces.
    pub fn collate_traces(&mut self) -> &[Tracer_Node] {
        use std::collections::hash_map::Entry;
        use std::collections::HashMap;

        // Map { tag => (index_into_saved_traces, tot_n_calls) }
        let mut tag_map: HashMap<&'static str, (usize, usize)> = HashMap::new();

        // Accumulate n_calls of all nodes with the same tag in the first one found,
        // and leave all others with n_calls = 0.
        for (i, node) in self.saved_traces.iter_mut().enumerate() {
            match tag_map.entry(&node.info.tag) {
                Entry::Vacant(v) => {
                    v.insert((i, 1));
                }
                Entry::Occupied(mut o) => {
                    let (idx, n_calls) = *o.get();
                    o.insert((idx, n_calls + 1));
                    node.info.n_calls = 0;
                }
            }
        }

        for (_, (idx, tot_calls)) in tag_map {
            self.saved_traces[idx].info.n_calls = tot_calls;
        }

        &self.saved_traces
    }

    pub fn start_frame(&mut self) {
        self.saved_traces.clear();
        self.cur_active = None;
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
}

#[inline]
pub fn debug_trace(tag: &'static str, tracer: Rc<RefCell<Tracer>>) -> Scope_Trace {
    tracer.borrow_mut().push_scope_trace(tag);
    Scope_Trace { tracer }
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

/// Construct a forest of Trace_Trees from the saved_traces array.
pub fn build_trace_trees(traces: &[Tracer_Node]) -> Vec<Trace_Tree<'_>> {
    let mut forest = vec![];

    if traces.is_empty() {
        return forest;
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

    let mut trees: Vec<Option<Trace_Tree>> = traces
        .iter()
        .map(|node| Some(Trace_Tree::new(node)))
        .collect();

    // Fill the `children` vecs.
    // Here we iterate in reverse on both saved_traces and trees, and during each iteration
    // we take() one tree out of the `trees` array. Since the nodes are ordered, we always
    // remove children before their parent, so we never try to unwrap() an already-taken node.
    for (i, node) in traces.iter().enumerate().rev() {
        let trace_tree = trees[i].take().unwrap();
        if node.info.n_calls == 0 {
            continue;
        }
        if let Some(p_idx) = node.parent_idx {
            trees[p_idx].as_mut().unwrap().children.push(trace_tree);
        } else {
            forest.push(trace_tree);
        }
    }

    forest
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
