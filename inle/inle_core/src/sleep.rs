use std::time::Duration;

type Sleep_Init_Result = Result<Duration, Box<dyn std::error::Error>>;

/// Returns Ok(granularity) or Err
pub fn init_sleep() -> Sleep_Init_Result {
    #[cfg(target_os = "windows")]
    {
        win32::init_sleep_internal()
    }
    #[cfg(target_os = "linux")]
    {
        unix::init_sleep_internal()
    }
    #[cfg(target_os = "macos")]
    {
        // @Incomplete
        Ok(Duration::from_millis(1))
    }
}

pub fn sleep(time: Duration) {
    #[cfg(not(target_os = "windows"))]
    {
        unix::sleep_internal(time);
    }
    #[cfg(target_os = "windows")]
    {
        win32::sleep_internal(time);
    }
}

#[cfg(target_os = "windows")]
#[allow(clippy::upper_case_acronyms)]
mod win32 {
    use std::borrow::Cow;
    use std::os::raw::*;
    use std::time::Duration;

    type UINT = c_uint;
    type LPTIMECAPS = *mut TIMECAPS;
    type MMRESULT = c_uint;
    type DWORD = c_ulong;

    const MMRESULT_NOERROR: MMRESULT = 0;
    const TIMERR_NOCANDO: MMRESULT = 97;

    #[allow(non_snake_case)]
    #[repr(C)]
    struct TIMECAPS {
        pub wPeriodMin: UINT,
        pub wPeriodMax: UINT,
    }

    #[link(name = "winmm")]
    extern "system" {
        fn timeGetDevCaps(time_caps: LPTIMECAPS, sizeof_time_caps: UINT) -> MMRESULT;

        fn timeBeginPeriod(period: UINT) -> MMRESULT;
    }

    #[link(name = "Kernel32")]
    extern "system" {
        fn Sleep(milliseconds: DWORD);
    }

    struct Sleep_Init_Error {
        code: MMRESULT,
    }

    impl std::fmt::Display for Sleep_Init_Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            let msg = match self.code {
                0 => Cow::Borrowed("MMSYSERR_NOERROR"),
                1 => Cow::Borrowed("MMSYSERR_ERROR"),
                TIMERR_NOCANDO => Cow::Borrowed("TIMERR_NOCANDO"),
                _ => Cow::Owned(format!("Unknown MMERROR {}", self.code)),
            };
            write!(f, "{}", msg)
        }
    }

    impl std::fmt::Debug for Sleep_Init_Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self)
        }
    }

    impl std::error::Error for Sleep_Init_Error {}

    pub(super) fn init_sleep_internal() -> super::Sleep_Init_Result {
        use std::mem::MaybeUninit;

        let mut tc = MaybeUninit::uninit();
        unsafe {
            let res = timeGetDevCaps(tc.as_mut_ptr(), std::mem::size_of::<TIMECAPS>() as UINT);
            if res != MMRESULT_NOERROR {
                return Err(Box::new(Sleep_Init_Error { code: res }));
            }

            let tc = tc.assume_init();
            let res = timeBeginPeriod(tc.wPeriodMin);
            if res != MMRESULT_NOERROR {
                return Err(Box::new(Sleep_Init_Error { code: res }));
            }
            Ok(Duration::from_millis(tc.wPeriodMin as u64))
        }
    }

    pub(super) fn sleep_internal(time: Duration) {
        unsafe {
            Sleep(time.as_millis() as UINT);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod unix {
    use std::borrow::Cow;
    use std::os::raw::c_int;
    use std::time::Duration;

    struct Sleep_Init_Error {
        code: c_int,
    }

    impl std::fmt::Display for Sleep_Init_Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            let msg = match self.code {
                libc::EFAULT => {
                    Cow::Borrowed("EFAULT: tp points outside the accessible address space.")
                }
                libc::EINVAL => {
                    Cow::Borrowed("EINVAL: The clk_id specified is not supported on this system.")
                }
                _ => Cow::Owned(format!("Unknown code {}", self.code)),
            };
            write!(f, "{}", msg)
        }
    }

    impl std::fmt::Debug for Sleep_Init_Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self)
        }
    }

    impl std::error::Error for Sleep_Init_Error {}

    pub(super) fn init_sleep_internal() -> super::Sleep_Init_Result {
        let mut ts = std::mem::MaybeUninit::uninit();
        let clk_id = libc::CLOCK_MONOTONIC;

        unsafe {
            let res = libc::clock_getres(clk_id, ts.as_mut_ptr());
            if res == 0 {
                let ts = ts.assume_init();
                Ok(Duration::from_secs(ts.tv_sec as u64) + Duration::from_nanos(ts.tv_nsec as u64))
            } else {
                Err(Box::new(Sleep_Init_Error { code: res }))
            }
        }
    }

    pub(super) fn sleep_internal(time: Duration) {
        use std::io::Error;

        let usecs = time.as_micros() as libc::time_t;
        let mut ti = libc::timespec {
            tv_nsec: (usecs % 1_000_000) * 1000,
            tv_sec: usecs / 1_000_000,
        };
        unsafe {
            // (From SFML/System/Unix/SleepImpl)
            // If nanosleep returns -1, we check errno. If it is EINTR
            // nanosleep was interrupted and has set ti to the remaining
            // duration. We continue sleeping until the complete duration
            // has passed. We stop sleeping if it was due to an error.
            while libc::nanosleep(&ti, &mut ti) == -1
                && Error::last_os_error().raw_os_error().unwrap() == libc::EINTR
            {}
        }
    }
}
