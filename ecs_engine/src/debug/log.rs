use super::tracer::Tracer_Node;
use std::collections::VecDeque;

#[derive(Default)]
pub struct Debug_Log {
    pub cur_frame: usize,
    pub hist_len: usize,
    pub max_hist_len: usize,
    pub frames: VecDeque<Debug_Log_Frame>,
}

#[derive(Default)]
pub struct Debug_Log_Frame {
    pub traces: Vec<Tracer_Node>,
}

impl Debug_Log {
    pub fn with_hist_len(max_hist_len: usize) -> Self {
        Debug_Log {
            max_hist_len,
            frames: VecDeque::with_capacity(max_hist_len),
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

    pub fn get_frame(&self, frame_number: usize) -> Option<&Debug_Log_Frame> {
        if (self.cur_frame - self.hist_len..self.cur_frame).contains(&frame_number) {
            let idx = self.cur_frame - frame_number;
            let idx = self.hist_len - idx - 1;
            Some(&self.frames[idx])
        } else {
            None
        }
    }

    pub fn push_trace(&mut self, trace: &[Tracer_Node]) {
        self.frames.back_mut().unwrap().traces = trace.to_vec();
    }
}
