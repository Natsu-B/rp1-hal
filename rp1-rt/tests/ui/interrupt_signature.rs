mod abi {
    #[rp1_rt::interrupt(proc0)]
    extern "C" fn TIMER_0() {}
}

mod argument {
    #[rp1_rt::interrupt(proc1)]
    fn TIMER_1(value: u32) {
        let _ = value;
    }
}

mod asynchronous {
    #[rp1_rt::interrupt(proc0)]
    async fn TIMER_0() {}
}

mod constant {
    #[rp1_rt::interrupt(proc0)]
    const fn TIMER_0() {}
}

mod generic {
    #[rp1_rt::interrupt(proc0)]
    fn TIMER_0<T>() {}
}

mod return_value {
    #[rp1_rt::interrupt(proc0)]
    fn TIMER_0() -> u32 {
        0
    }
}

mod unsafe_handler {
    #[rp1_rt::interrupt(proc0)]
    unsafe fn TIMER_0() {}
}

fn main() {}
