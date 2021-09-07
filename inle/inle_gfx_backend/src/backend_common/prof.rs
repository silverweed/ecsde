#[cfg(feature = "gfx-gl")]
mod gl;

#[cfg(feature = "gfx-gl")]
use self::gl as backend;

pub struct Gpu_Profiler {
    query_group: backend::Gpu_Query_Group,
    old_query_group: Option<backend::Gpu_Query_Group>,
}

impl Gpu_Profiler {
    pub fn new() -> Self {
        Self {
            query_group: backend::create_gpu_query_group(),
            old_query_group: None,
        }
    }

    pub fn is_available() -> bool {
        backend::is_profiling_available()
    }

    pub fn start_gpu_frame(&mut self) {
        let ok = backend::start_gpu_query(&mut self.query_group);
        if !ok {
            let mut new_query_group = backend::create_gpu_query_group();
            std::mem::swap(&mut self.query_group, &mut new_query_group);
            self.old_query_group.replace(new_query_group);
        }
        let _ok = backend::start_gpu_query(&mut self.query_group);
        debug_assert!(_ok);
    }

    pub fn end_gpu_frame(&mut self) {
        backend::end_latest_gpu_query(&mut self.query_group);

        let mut must_delete_old_group = false;
        if let Some(group) = self.old_query_group.as_ref() {
            must_delete_old_group = backend::did_read_all_results(group);
        }

        if must_delete_old_group {
            backend::delete_query_group(self.old_query_group.take().unwrap());
        }
    }

    pub fn get_latest_result(&mut self) -> Option<std::time::Duration> {
        if let Some(group) = self.old_query_group.as_mut() {
            if !backend::did_read_all_results(group) {
                return backend::retrieve_next_query_result(group);
            }
        }
        backend::retrieve_next_query_result(&mut self.query_group)
    }
}
