use core::arch::naked_asm;

#[repr(C)]
 #[derive(Debug, Default, Clone)]
 pub struct GeneralRegisters {
     pub ra: usize,
     pub sp: usize,
     pub gp: usize, // only valid for user traps
     pub tp: usize, // only valid for user traps
     pub t0: usize,
     pub t1: usize,
     pub t2: usize,
     pub s0: usize,
     pub s1: usize,
     pub a0: usize,
     pub a1: usize,
     pub a2: usize,
     pub a3: usize,
     pub a4: usize,
     pub a5: usize,
     pub a6: usize,
     pub a7: usize,
     pub s2: usize,
     pub s3: usize,
     pub s4: usize,
     pub s5: usize,
     pub s6: usize,
     pub s7: usize,
     pub s8: usize,
     pub s9: usize,
     pub s10: usize,
     pub s11: usize,
     pub t3: usize,
     pub t4: usize,
     pub t5: usize,
     pub t6: usize,
 }
 
 #[repr(C)]
 #[derive(Debug, Default, Clone)]
 pub struct TrapFrame {
     /// All general registers.
     pub regs: GeneralRegisters,
     /// Supervisor Exception Program Counter.
     pub sepc: usize,
     /// Supervisor Status Register.
     pub sstatus: usize,
 }

#[repr(C)]
#[derive(Debug, Default)]
pub struct TaskContext {
    pub ra: usize, // return address (x1)
    pub sp: usize, // stack pointer (x2)

    pub s0: usize, // x8-x9
    pub s1: usize,

    pub s2: usize, // x18-x27
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
}

impl TaskContext {
    pub const fn new() -> Self {
        unsafe { core::mem::MaybeUninit::zeroed().assume_init() }
    }

    pub fn init(&mut self, entry: usize, kstack_top: usize) {
        self.sp = kstack_top;
        self.ra = entry;
    }

    pub fn switch_to(&mut self, next_ctx: &Self) {
        unsafe { context_switch(self, next_ctx) }
    }
}

#[naked]
unsafe extern "C" fn context_switch(_current_task: &mut TaskContext, _next_task: &TaskContext) {
    unsafe {
        naked_asm!(
            "
        // save old context (callee-saved registers)
        sd     ra, 0*8(a0)
        sd     sp, 1*8(a0)
        sd     s0, 2*8(a0)
        sd     s1, 3*8(a0)
        sd     s2, 4*8(a0)
        sd     s3, 5*8(a0)
        sd     s4, 6*8(a0)
        sd     s5, 7*8(a0)
        sd     s6, 8*8(a0)
        sd     s7, 9*8(a0)
        sd     s8, 10*8(a0)
        sd     s9, 11*8(a0)
        sd     s10, 12*8(a0)
        sd     s11, 13*8(a0)

        // restore new context
        ld     s11, 13*8(a1)
        ld     s10, 12*8(a1)
        ld     s9, 11*8(a1)
        ld     s8, 10*8(a1)
        ld     s7,  9*8(a1)
        ld     s6,  8*8(a1)
        ld     s5,  7*8(a1)
        ld     s4,  6*8(a1)
        ld     s3,  5*8(a1)
        ld     s2,  4*8(a1)
        ld     s1,  3*8(a1)
        ld     s0,  2*8(a1)
        ld     sp,  1*8(a1)
        ld     ra,  0*8(a1)

        ret",
        )
    }
}
