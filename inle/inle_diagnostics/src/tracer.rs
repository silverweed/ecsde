use crate::prelude::Debug_Tracers;
use rayon::prelude::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread::ThreadId;
use std::time::{Duration, Instant};

// @Speed: currently I don't think we can do better than having a mutex wrapping the
// per-thread tracers, since:
//    a) they can be accessed by multiple threads (their own thread + the main thread
//       when it collects the results or adds hints to the console)
//    b) the Tracers map can be modified while a Scope_Trace is active, so we cannot
//       safely save just the naked pointer into it, as it could change.
// We could do better (e.g. pre-spawning ALL threads, using a short task system and
// ensuring all tracers are created in that moment and then access them without locking,
// e.g. putting them in an array indexed by number of thread or something) but it'd
// still not be trivial to aggregate their results; moreover, hopefully these mutexes
// are very rarely contended (but that's probably not true for the _outer_ mutex that guards DEBUG_TRACERS...)
pub type Tracers = HashMap<ThreadId, Arc<Mutex<Tracer>>>;

pub struct Tracer {
    // Tree of Tracer_Nodes representing the call tree.
    pub saved_traces: Vec<Tracer_Node>,

    // Latest pushed (and not-yet-popped) node index
    cur_active: Option<usize>,

    thread_id: ThreadId,
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
    pub start_t: Instant,
    pub end_t: Instant,
    pub tag: &'static str,
    pub tag_hash: u32,

    // These are only meaningful for collated traces
    pub n_calls: u32,
    pub tot_duration: Duration,
}

impl Scope_Trace_Info {
    #[inline(always)]
    pub fn duration(&self) -> Duration {
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
    tracer: Arc<Mutex<Tracer>>,
}

impl Scope_Trace {
    #[inline(always)]
    pub fn new(tracer: &Arc<Mutex<Tracer>>, tag: &'static str, tag_hash: u32) -> Self {
        tracer.lock().unwrap().push_scope_trace(tag, tag_hash);
        Self {
            tracer: tracer.clone(),
        }
    }
}

impl Drop for Scope_Trace {
    #[inline(always)]
    fn drop(&mut self) {
        self.tracer.lock().unwrap().pop_scope_trace();
    }
}

/// A trimmed-down version of Tracer_Node used to store data with lower memory footprint
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tracer_Node_Final {
    pub info: Scope_Trace_Info_Final,
    pub parent_idx: Option<u16>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scope_Trace_Info_Final {
    pub tag: &'static str,

    // High 24 bytes: n_calls (max value = 16'777'215)
    // Low 40 bytes: duration_nanos (max value = 1'099'511'627'775 =~ 1099 s)
    pub n_calls_and_tot_duration: u64,
}

impl Scope_Trace_Info_Final {
    #[inline(always)]
    pub fn new(tag: &'static str, n_calls: u32, tot_duration: Duration) -> Self {
        let n_calls = n_calls.min(1 << 24);
        let tot_duration_nanos = tot_duration.as_nanos().min(1 << 40) as u64;
        if tot_duration_nanos as u128 != tot_duration.as_nanos() {
            lwarn!(
                "Truncating duration nanos from {} to {}",
                tot_duration.as_nanos(),
                tot_duration_nanos
            );
        }
        Self {
            tag,
            n_calls_and_tot_duration: ((n_calls as u64) << 40)
                | (tot_duration_nanos & 0xFF_FFFF_FFFF),
        }
    }

    #[inline(always)]
    pub fn tot_duration(&self) -> Duration {
        let duration_nanos = self.n_calls_and_tot_duration & 0xFF_FFFF_FFFF;
        Duration::from_nanos(duration_nanos as _)
    }

    #[inline(always)]
    pub fn n_calls(&self) -> u32 {
        (self.n_calls_and_tot_duration >> 40) as _
    }
}

#[inline(always)]
pub fn debug_trace(tag: &'static str, tag_hash: u32, tracer: &Arc<Mutex<Tracer>>) -> Scope_Trace {
    Scope_Trace::new(tracer, tag, tag_hash)
}

#[inline(always)]
pub fn debug_trace_on_thread(
    tag: &'static str,
    tracers: &Debug_Tracers,
    thread_id: ThreadId,
    tag_hash: u32,
) -> Scope_Trace {
    let mut tracers = tracers.lock().unwrap();
    let tracer = tracers
        .entry(thread_id)
        .or_insert_with(|| Arc::new(Mutex::new(Tracer::new(thread_id))));

    debug_trace(tag, tag_hash, tracer)
}

#[derive(Clone, Debug)]
pub struct Trace_Tree<'a> {
    pub node: &'a Tracer_Node_Final,
    pub children: Vec<Trace_Tree<'a>>,
}

impl Trace_Tree<'_> {
    #[inline(always)]
    pub fn new(node: &Tracer_Node_Final) -> Trace_Tree {
        Trace_Tree {
            node,
            children: vec![],
        }
    }
}

impl Tracer {
    pub fn new(thread_id: ThreadId) -> Tracer {
        Tracer {
            saved_traces: Vec::with_capacity(2_048),
            cur_active: None,
            thread_id,
        }
    }

    // NOTE: don't do any kind of hard work here, or the tracing will
    // be too intrusive! Prefer delaying work until later, when processing
    // the traces.
    #[inline(always)]
    fn push_scope_trace(&mut self, tag: &'static str, tag_hash: u32) {
        let now = Instant::now();
        self.saved_traces.push(Tracer_Node {
            info: Scope_Trace_Info {
                start_t: now,
                end_t: now,
                tag,
                tag_hash,
                n_calls: 1,
                tot_duration: Duration::default(),
            },
            parent_idx: self.cur_active,
        });
        self.cur_active = Some(self.saved_traces.len() - 1);
    }

    #[inline(always)]
    fn pop_scope_trace(&mut self) {
        let now = Instant::now();
        let mut active_node = &mut self.saved_traces[self
            .cur_active
            .expect("[ ERROR ] Popped scope trace while none is active!")];
        active_node.info.end_t = now;
        self.cur_active = active_node.parent_idx;
    }

    pub fn start_frame(&mut self) {
        if let Some(cur_active) = self.cur_active {
            let active = &self.saved_traces[cur_active].info;
            // If start_t == end_t we've been called by another thread while we had a Scope_Trace
            // open. In this case keep the current node, otherwise we'd lose its data before we
            // close that scope.
            // @Audit: is this the proper solution?
            if active.start_t == active.end_t {
                self.saved_traces.swap(0, cur_active);
                self.saved_traces.truncate(1);
                self.cur_active = Some(0);
                return;
            }
        }
        self.saved_traces.clear();
        self.cur_active = None;
    }

    #[cold]
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

#[inline(always)]
pub fn total_traced_time(traces: &[Tracer_Node_Final]) -> Duration {
    // NOTE: this takes so little time (~1us) that's not worth parallelizing at all (it's also called rarely)
    traces
        .iter()
        .map(|node| {
            if node.parent_idx.is_none() {
                node.info.tot_duration()
            } else {
                Duration::default()
            }
        })
        .sum()
}

#[inline(always)]
pub fn sort_trace_trees(trees: &mut [Trace_Tree]) {
    #[inline(always)]
    fn sort_tree_internal(tree: &mut Trace_Tree) {
        tree.children
            .sort_by(|a, b| b.node.info.tot_duration().cmp(&a.node.info.tot_duration()));
        for c in &mut tree.children {
            sort_tree_internal(c);
        }
    }

    for tree in trees {
        sort_tree_internal(tree);
    }
}

struct Passthrough_Hasher(u64);

impl std::hash::Hasher for Passthrough_Hasher {
    #[cold]
    fn write(&mut self, _: &[u8]) {
        unimplemented!();
    }

    #[inline(always)]
    fn write_u32(&mut self, n: u32) {
        self.0 = u64::from(n);
    }

    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }
}

#[derive(Default)]
struct Passthrough_Build_Hasher;

impl std::hash::BuildHasher for Passthrough_Build_Hasher {
    type Hasher = Passthrough_Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        Passthrough_Hasher(0)
    }
}

/// Deduplicates tracer nodes and returns the final traces.
// @Incomplete: handle multiple threads in a sane way (right now tot_duration
// ends up being the sum of all threads, which may be ok, but should be made explicit
// in the debug overlay).
#[must_use]
pub fn collate_traces(saved_traces: &[Tracer_Node]) -> Vec<Tracer_Node_Final> {
    struct Tag_Map_Info {
        pub tag: &'static str,
        pub n_calls: u32,
        pub tot_duration: Duration,
        pub parent_idx: Option<usize>,
    }

    // Note: `hash` is computed from the entire call stack (we can't just use the tag,
    // or the trace will only show the call under the first caller).
    #[inline(always)]
    fn hash_node(nodes: &[Tracer_Node], node: &Tracer_Node) -> u32 {
        const FNV1A_PRIME32: u32 = 16_777_619;
        const FNV1A_START32: u32 = 2_166_136_261;

        let mut node = node;
        let mut x = FNV1A_START32;
        x ^= node.info.tag_hash;
        x = x.wrapping_mul(FNV1A_PRIME32);
        while let Some(parent_idx) = node.parent_idx {
            node = &nodes[parent_idx];
            x ^= node.info.tag_hash;
            x = x.wrapping_mul(FNV1A_PRIME32);
        }
        x
    }

    // Accumulate n_calls of all nodes with the same tag.
    // @Speed: this could use the frame_allocator.
    let t = Instant::now();
    let hashes = saved_traces
        .par_iter()
        .map(|node| hash_node(saved_traces, node))
        .collect::<Vec<_>>();
    ldebug!("hash: {:?}", t.elapsed());

    let t = Instant::now();
    // Used to iterate the tag_map in insertion order
    let mut tags_ordered: Vec<u32> = Vec::with_capacity(saved_traces.len() / 10);
    let mut tag_map = HashMap::with_hasher(Passthrough_Build_Hasher::default());
    let mut idx_map = HashMap::with_hasher(Passthrough_Build_Hasher::default());
    for (i, node) in saved_traces.iter().enumerate() {
        let hash = hashes[i];
        let entry = tag_map.entry(hash).or_insert_with(|| Tag_Map_Info {
            tag: node.info.tag,
            n_calls: 0,
            tot_duration: Duration::default(),
            parent_idx: node.parent_idx.map(|i| {
                let hash = hashes[i];
                idx_map[&hash]
            }),
        });
        entry.n_calls += 1;
        entry.tot_duration += node.info.duration();
        let is_new = entry.n_calls == 1;
        if is_new {
            idx_map.insert(hash, tags_ordered.len());
            tags_ordered.push(hash);
        }
    }
    ldebug!("insert: {:?}", t.elapsed());

    tags_ordered
        .iter()
        .map(|hash| {
            let info = &tag_map[hash];
            Tracer_Node_Final {
                info: Scope_Trace_Info_Final::new(info.tag, info.n_calls, info.tot_duration),
                parent_idx: info
                    .parent_idx
                    .map(|idx| idx.try_into().expect("parent_idx is too big to fit u16!")),
            }
        })
        .collect()
}

// Given some Tracer_Node_Final, merges all the ones with the same tag into a single one,
// accumulating duration and n_calls. All parent information is lost.
pub fn flatten_traces(
    traces: &[Tracer_Node_Final],
) -> impl Iterator<Item = Tracer_Node_Final> + '_ {
    let mut flat_traces = HashMap::new();
    for trace in traces {
        let accum = flat_traces
            .entry(&trace.info.tag)
            .or_insert_with(|| Tracer_Node_Final {
                info: Scope_Trace_Info_Final::new(trace.info.tag, 0, Duration::default()),
                parent_idx: None,
            });
        accum.info = Scope_Trace_Info_Final::new(
            accum.info.tag,
            accum.info.n_calls() + trace.info.n_calls(),
            accum.info.tot_duration() + trace.info.tot_duration(),
        );
    }
    flat_traces.into_iter().map(|(_, v)| v)
}

/// Construct a forest of Trace_Trees from the saved_traces array.
pub fn build_trace_trees(traces: &[Tracer_Node_Final]) -> Vec<Trace_Tree<'_>> {
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
        debug_assert!(node.info.n_calls() > 0);

        if let Some(p_idx) = node.parent_idx {
            trees[p_idx as usize]
                .as_mut()
                .unwrap()
                .children
                .push(trace_tree);
        } else {
            forest.push(trace_tree);
        }
    }

    forest
}
