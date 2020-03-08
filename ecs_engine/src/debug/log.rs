use super::tracer::Tracer_Node;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Debug_Log {
    pub cur_frame: u64,
    pub hist_len: u32,
    pub max_hist_len: u32,
    pub frames: VecDeque<Debug_Log_Frame>,
}

#[derive(Default, Debug)]
pub struct Debug_Log_Frame {
    pub traces: Vec<Tracer_Node>,
}

impl Debug_Log {
    pub fn with_hist_len(max_hist_len: u32) -> Self {
        Debug_Log {
            max_hist_len,
            frames: VecDeque::with_capacity(max_hist_len as usize),
            ..Default::default()
        }
    }

    pub fn start_frame(&mut self) {
        self.cur_frame += 1;
        if self.hist_len == self.max_hist_len {
            self.frames.pop_front();
        } else {
            self.hist_len += 1;
        }
        self.frames.push_back(Debug_Log_Frame::default());
    }

    pub fn get_frame(&self, frame_number: u64) -> Option<&Debug_Log_Frame> {
        if (self.cur_frame - self.hist_len as u64 + 1..=self.cur_frame).contains(&frame_number) {
            let idx = self.cur_frame - frame_number;
            let idx = self.hist_len as u64 - idx - 1;
            Some(&self.frames[idx as usize])
        } else {
            None
        }
    }

    pub fn push_trace(&mut self, trace: &[Tracer_Node]) {
        self.frames.back_mut().unwrap().traces = trace.to_vec();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug::tracer::*;
    use std::time::*;

    #[test]
    fn get_frames() {
        let mut log = Debug_Log::with_hist_len(10);

        log.start_frame();

        let info = Scope_Trace_Info {
            start_t: Instant::now(),
            end_t: Instant::now(),
            tag: "Test",
            n_calls: 1,
            tot_duration: Duration::default(),
        };
        let node = Tracer_Node {
            info: info.clone(),
            parent_idx: None,
        };

        log.push_trace(&[node.clone()]);

        {
            let n = &log.get_frame(0).unwrap().traces;
            assert_eq!(n, &[node.clone()]);
        }

        log.start_frame();

        let info2 = Scope_Trace_Info { n_calls: 2, ..info };
        let node2 = Tracer_Node {
            info: info2,
            ..node
        };

        log.push_trace(&[node2.clone()]);

        {
            let n = &log.get_frame(0).unwrap().traces;
            assert_eq!(n, &[node.clone()]);

            let n = &log.get_frame(1).unwrap().traces;
            assert_eq!(n, &[node2.clone()]);
        }

        log.start_frame();
        log.start_frame();
        log.start_frame();
        log.start_frame();
        log.start_frame();
        log.start_frame();
        log.start_frame();
        log.start_frame();
        log.start_frame();

        // At frame 11

        {
            let n = log.get_frame(0);
            assert!(n.is_none());

            let n = &log.get_frame(1).unwrap().traces;
            assert_eq!(n, &[node2.clone()]);
        }
    }
}
