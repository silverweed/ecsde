use crate::prelude::Debug_Tracer;
use std::fmt::Debug;
use std::time;

pub struct Tracer {
    // Tree of Tracer_Nodes representing the call tree.
    pub saved_traces: Vec<Tracer_Node>,
    // Latest pushed (and not-yet-popped) node index
    cur_active: Option<usize>,
}

/// Represents a traced scope with its info and a link to its parent.
#[derive(Debug, Clone, PartialEq)]
pub struct Tracer_Node {
    pub info: Scope_Trace_Info,
    pub parent_idx: Option<usize>,
}

/// The actual trace information for a single scope.
#[derive(Clone, PartialEq)]
pub struct Scope_Trace_Info {
    pub start_t: time::Instant,
    pub end_t: time::Instant,
    pub tag: &'static str,

    // These are only meaningful for collated traces
    pub n_calls: usize,
    pub tot_duration: time::Duration,
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
    tracer: Debug_Tracer,
}

impl Scope_Trace {
    pub fn new(tracer: Debug_Tracer, tag: &'static str) -> Self {
        tracer.lock().unwrap().push_scope_trace(tag);
        Self { tracer }
    }
}

impl Drop for Scope_Trace {
    fn drop(&mut self) {
        self.tracer.lock().unwrap().pop_scope_trace();
    }
}

#[inline]
pub fn debug_trace(tag: &'static str, tracer: Debug_Tracer) -> Option<Scope_Trace> {
    Some(Scope_Trace::new(tracer, tag))
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
                tot_duration: time::Duration::default(),
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
}

pub fn total_traced_time(traces: &[Tracer_Node]) -> time::Duration {
    traces
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

pub fn sort_trace_trees(trees: &mut [Trace_Tree]) {
    fn sort_tree_internal(tree: &mut Trace_Tree) {
        tree.children
            .sort_by(|a, b| b.node.info.tot_duration.cmp(&a.node.info.tot_duration));
        for c in &mut tree.children {
            sort_tree_internal(c);
        }
    }

    for tree in trees {
        sort_tree_internal(tree);
    }
}

/// Deduplicates tracer nodes and returns the final traces.
// @Incomplete: handle multiple threads in a sane way (right now tot_duration
// ends up being the sum of all threads, which may be ok, but should be made explicit
// in the debug overlay).
pub fn collate_traces(saved_traces: &mut Vec<Tracer_Node>) {
    use std::collections::hash_map::Entry;
    use std::collections::HashMap;

    #[derive(Copy, Clone)]
    struct Tag_Map_Info {
        pub idx_into_saved_traces: usize,
        pub tot_n_calls: usize,
        pub tot_duration: time::Duration,
    }

    // where `hash` is computed from the entire call stack (we can't just use the tag,
    // or the trace will only show the call under the first caller).
    let mut tag_map: HashMap<u32, Tag_Map_Info> = HashMap::new();

    fn hash_node(nodes: &[Tracer_Node], node: &Tracer_Node) -> u32 {
        use crate::common::stringid::{FNV1A_PRIME32, FNV1A_START32};

        let mut result = FNV1A_START32;
        let mut node = node;
        loop {
            let tag = node.info.tag;
            for b in tag.bytes() {
                result ^= u32::from(b);
                result = result.wrapping_mul(FNV1A_PRIME32);
            }
            if let Some(parent_idx) = node.parent_idx {
                node = &nodes[parent_idx];
            } else {
                break;
            }
        }
        result
    }

    // Accumulate n_calls of all nodes with the same tag in the first one found,
    // and leave all others with n_calls = 0.
    // @Speed: this could use the frame_allocator.
    let hashes = saved_traces
        .iter()
        .map(|node| hash_node(saved_traces, node))
        .collect::<Vec<_>>();
    for (i, node) in saved_traces.iter_mut().enumerate() {
        match tag_map.entry(hashes[i]) {
            Entry::Vacant(v) => {
                v.insert(Tag_Map_Info {
                    idx_into_saved_traces: i,
                    tot_n_calls: 1,
                    tot_duration: node.info.duration(),
                });
            }
            Entry::Occupied(mut o) => {
                let tag_map_info = *o.get();
                o.insert(Tag_Map_Info {
                    tot_n_calls: tag_map_info.tot_n_calls + 1,
                    tot_duration: tag_map_info.tot_duration + node.info.duration(),
                    ..tag_map_info
                });
                node.info.n_calls = 0;
            }
        }
    }

    for (
        _,
        Tag_Map_Info {
            idx_into_saved_traces,
            tot_n_calls,
            tot_duration,
        },
    ) in tag_map
    {
        saved_traces[idx_into_saved_traces].info.n_calls = tot_n_calls;
        saved_traces[idx_into_saved_traces].info.tot_duration = tot_duration;
    }
}

/// Construct a forest of Trace_Trees from the saved_traces array.
// @Audit @Soundness: verify this function is actually working, after the collate_traces change.
pub fn build_trace_trees(traces: &[Tracer_Node]) -> Vec<Trace_Tree<'_>> {
    let mut forest = vec![];

    if traces.is_empty() {
        return forest;
    }

    // Note: we exploit the fact that saved_traces elements are ordered as
    // a tree. i.e.:
    //
    //      A
    //    /   \
    //   B     C
    //  / \     \
    // D   E     F
    //
    // saved_traces = [A, B, D, E, C, F]
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
