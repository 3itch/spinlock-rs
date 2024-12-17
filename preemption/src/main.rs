use crate::preemption::PreemptControl;
use core_affinity;
use std::{thread, time};

mod preemption;

fn main() {
    /*
        This is just a PoC. in a real kernel u would have actual critical
        section code here,  the critical section would be guarded by a spinlock.
        this is just a simulation spawning threads and using the preemption control
    */
    
    let preempt = PreemptControl::new();

    let guard = preempt.preempt_disable();
    let core_ids = core_affinity::get_core_ids().unwrap();

    thread::sleep(time::Duration::from_secs(5));

    let mut handles = vec![];

    for (i, core_id) in core_ids.into_iter().enumerate() {
        let handle = thread::spawn(move || {
            if core_affinity::set_for_current(core_id) {
                println!("[!] thread {} is pinned to {}", i, core_id.id);
            }

            thread::sleep(time::Duration::from_secs(1));
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Re-enable preemption by dropping `guard`
    // Enable interrupts/fast interrupts
    preempt.preempt_enable(guard);
}
