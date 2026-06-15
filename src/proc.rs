pub static MAX_PROC: usize = 512;
static mut PROCESSES: [Option<PCB>; MAX_PROC] = [const { None }; MAX_PROC];

pub struct PCB {
    pub pid: u64,
    pub context: [u64; 32],
    pub sepc: u64,
    pub sstatus: u64,
}

impl PCB {
    fn new(pid: u64) -> PCB {
        PCB {
            pid,
            context: [0; 32],
            sepc: 0,
            sstatus: 0,
        }
    }

    fn from(pid: u64, context: *const [u64; 32], sepc: u64, sstatus: u64) -> PCB {
        unsafe {
            PCB {
                pid,
                context: **&context,
                sepc,
                sstatus,
            }
        }
    }
}

// pub fn proc_alloc(context: *const [u64;32], sepc: u64, sstatus: u64) -> Result<u64, &'static str> {
//     unsafe {
//         for (n, i) in PROCESSES.iter_mut().enumerate() {
//             if i.is_none() {
//                 *i = Some(PCB::from(n as u64, context, sepc, sstatus));
//                 return Ok(n as u64);
//             }
//         }
//     }
//
//     Err("proc count max")
// }
