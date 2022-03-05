use {crate::tracer::Tracers, std::sync::Mutex};

pub type Debug_Tracers = Mutex<Tracers>;

lazy_static! {
    pub static ref DEBUG_TRACERS: Debug_Tracers = Mutex::new(Tracers::new());
}

#[macro_export]
macro_rules! trace {
    ($tag: expr) => {
        const fn fnv1a(bytes: &[u8]) -> u32 {
            const FNV1A_PRIME32: u32 = 16_777_619;
            const FNV1A_START32: u32 = 2_166_136_261;

            let mut result = FNV1A_START32;
            let mut i = 0;
            while i < bytes.len() {
                let b = bytes[i];
                result ^= b as u32;
                result = result.wrapping_mul(FNV1A_PRIME32);
                i += 1;
            }
            result
        }
        const HASH: u32 = fnv1a($tag.as_bytes());

        let _trace_var = $crate::tracer::debug_trace_on_thread(
            $tag,
            &$crate::prelude::DEBUG_TRACERS,
            std::thread::current().id(),
            HASH,
        );
    };
}
