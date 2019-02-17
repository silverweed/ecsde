use std::time::SystemTime;

pub struct Time {
	time: SystemTime,
	m_dt: f32
}

impl Time {
	pub fn new() -> Time {
		Time {
			time: SystemTime::now(),
			m_dt: 0f32
		}
	}

	pub fn update(&mut self) {
		let dtr = self.time.elapsed().unwrap();
		self.m_dt = dtr.as_secs() as f32 + dtr.subsec_nanos() as f32 * 1e-9;
		self.time = SystemTime::now();
	}

	pub fn dt(&self) -> f32 { self.m_dt }
}
