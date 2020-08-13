#[derive(Copy, Clone, Debug)]
pub enum Align {
    Begin,
    Middle,
    End,
}

impl Default for Align {
    fn default() -> Self {
        Align::Begin
    }
}

impl Align {
    pub fn aligned_pos(self, pos: f32, width: f32) -> f32 {
        match self {
            Align::Begin => pos,
            Align::Middle => 0.5 * (pos - width),
            Align::End => -(pos + width),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::ApproxEq;

    #[test]
    fn test_align() {
        {
            let r = Align::Begin.aligned_pos(0.0, 100.0);
            let ex = 0.0;
            assert!(
                r.approx_eq(ex, (0.0, 2)),
                format!("expected_approx: {}, got: {}", ex, r)
            );
        }

        {
            let r = Align::Middle.aligned_pos(-10.0, 100.0);
            let ex = -55.0;
            assert!(
                r.approx_eq(ex, (0.0, 2)),
                format!("expected_approx: {}, got: {}", ex, r)
            );
        }

        {
            let r = Align::End.aligned_pos(10.0, 100.0);
            let ex = -110.0;
            assert!(
                r.approx_eq(ex, (0.0, 2)),
                format!("expected_approx: {}, got: {}", ex, r)
            );
        }
    }
}
