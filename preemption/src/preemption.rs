use std::arch::asm;
use std::thread;

pub struct Spinlock {
    locked: *mut i32,  // ptr to atomic integer ( 0 = unlocked, 1 = locked )
}

impl Spinlock {
    pub fn new() -> Self {
        // spinlock with a i32 initialized to 0
        let locked = Box::into_raw(Box::new(0));
        Spinlock { locked }
    }

    pub fn lock(&self) {
        loop {
            unsafe {
                let mut prev_val: i32;
                // LDREX and STREX in loop 4 exclusive access
                asm!(
                    "ldrex {0}, [{1}]",      // loads the value from the addr
                    "cmp {0}, #0",           // cmp if the value is 0 ( lock is free )
                    "bne 1f",                // != 0 ( already locked ), jump to retry
                    "strex {0}, {2}, [{1}]", // store new value
                    "cmp {0}, #0",           // did strex succeed?!! ( return 0 if so )
                    "bne 1f",                // strex failed?!!, retry!
                    "dmb sy",                // memory orderingggg ( for exclusive access )
                    "1:",                    // label 4 entry
                    out(reg) prev_val,
                    in(reg) self.locked,
                    in(reg) 1,               // stored value
                    options(nostack, preserves_flags)
                );

                // prev_val was 0 and STREX returned 0, break out of loop.
                if prev_val == 0 {
                    break;
                }
            }

            thread::yield_now();
        }
    }

    pub fn unlock(&self) {
        unsafe {
            // release lock by writing 0 to the memory loc
            asm!(
                "strex x0, {0}, [{1}]",  // write 0 to the memory loc to release the lock
                in(reg) 0,
                in(reg) self.locked,
                out("x0") _,
                options(nostack, preserves_flags)
            );
        }
    }
}

pub struct PreemptControl {
    spinlock: Spinlock,
}

impl PreemptControl {
    pub fn new() -> Self {
        PreemptControl {
            spinlock: Spinlock::new(),
        }
    }

    pub fn preempt_disable(&self) -> PreemptGuard {
        unsafe {
            asm!("CPSID I");
            asm!("CPSID F");

            self.spinlock.lock();
        }

        PreemptGuard {
            spinlock: &self.spinlock,
        }
    }

    pub fn preempt_enable(&self, guard: PreemptGuard) {
        unsafe {
            asm!("CPSIE I");
            asm!("CPSIE F");
        }

        drop(guard);
    }
}

pub struct PreemptGuard<'a> {
    spinlock: &'a Spinlock,
}

impl<'a> Drop for PreemptGuard<'a> {
    fn drop(&mut self) {
        self.spinlock.unlock();
    }
}
