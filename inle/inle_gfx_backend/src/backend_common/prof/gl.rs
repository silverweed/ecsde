use gl::types::*;
use std::time::Duration;
use super::super::misc::check_gl_err;

macro_rules! glcheck {
    ($expr: expr) => {{
        let res = $expr;
        check_gl_err();
        res
    }};
}

const N_QUERIES_PER_GROUP: usize = 128;

pub struct Gpu_Query_Group {
    query_ids: [GLuint; N_QUERIES_PER_GROUP],
    results: [Option<Duration>; N_QUERIES_PER_GROUP],
    n_started: usize,
    n_results_queried: usize,
}

pub fn is_profiling_available() -> bool {
    gl::GenQueries::is_loaded()
}

pub fn create_gpu_query_group() -> Gpu_Query_Group {
    let mut group = Gpu_Query_Group {
        query_ids: [0; N_QUERIES_PER_GROUP],
        results: [None; N_QUERIES_PER_GROUP],
        n_started: 0,
        n_results_queried: 0
    };

    unsafe {
        glcheck!(gl::GenQueries(N_QUERIES_PER_GROUP as GLsizei, group.query_ids.as_mut_ptr()));
    }

    group
}

pub fn start_gpu_query(query_group: &mut Gpu_Query_Group) -> bool {
    if query_group.n_started < query_group.query_ids.len() - 1 {
        query_group.n_started += 1;
        unsafe {
            glcheck!(gl::BeginQuery(gl::TIME_ELAPSED, query_group.query_ids[query_group.n_started]));
        }
        true
    } else {
        false
    }
}

pub fn end_latest_gpu_query(_query_group: &mut Gpu_Query_Group) {
    unsafe {
        glcheck!(gl::EndQuery(gl::TIME_ELAPSED));
    }
}

pub fn retrieve_next_query_result(query_group: &mut Gpu_Query_Group) -> Option<Duration> {
    debug_assert!(query_group.n_results_queried <= query_group.n_started);

    let result_idx = query_group.n_results_queried;
    if result_idx < query_group.results.len() - 1 {
        let id = query_group.query_ids[result_idx];
        let mut available = 0;
        unsafe {
            glcheck!(gl::GetQueryObjectiv(id, gl::QUERY_RESULT_AVAILABLE, &mut available));
        }
        if available != 0 {
            let mut time = 0;
            unsafe {
                glcheck!(gl::GetQueryObjectui64v(id, gl::QUERY_RESULT, &mut time));
            }
            query_group.results[result_idx] = Some(Duration::from_nanos(time));
            query_group.n_results_queried += 1;
        }
    }

    query_group.results[result_idx]
}

pub fn did_read_all_results(group: &Gpu_Query_Group) -> bool {
    group.n_results_queried == group.results.len() - 1
}

pub fn delete_query_group(group: Gpu_Query_Group) {
    unsafe {
        glcheck!(gl::DeleteQueries(group.query_ids.len() as GLsizei, group.query_ids.as_ptr()));
    }
}
