const INTERRUPT_MASK: i64 = 1 << 63;
const U_MODE_SOFTWARE_INTERRUPT: i64 = INTERRUPT_MASK | 0;
const S_MODE_SOFTWARE_INTERRUPT: i64 = INTERRUPT_MASK | 1;
const M_MODE_SOFTWARE_INTERRUPT: i64 = INTERRUPT_MASK | 3;
const USER_TIMER_INTERRUPT: i64 = INTERRUPT_MASK | 4;
const SUPERVISOR_TIMER_INTERRUPT: i64 = INTERRUPT_MASK | 5;
const MACHINE_TIMER_INTERRUPT: i64 = INTERRUPT_MASK | 7;
const USER_EXTERNAL_INTERRUPT: i64 = INTERRUPT_MASK | 8;
const SUPERVISOR_EXTERNAL_INTERRUPT: i64 = INTERRUPT_MASK | 9;
const MACHINE_EXTERNAL_INTERRUPT: i64 = INTERRUPT_MASK | 11;

const INSTRUCTION_ADDRESS_MISALIGNED: i64 = 0;
const INSTRUCTION_ACCESS_FAULT: i64 = 1;
const ILLEGAL_INSTRUCTION: i64 = 2;
const BREAKPOINT: i64 = 3;
const LOAD_ADDRESS_MISALIGNED: i64 = 4;
const LOAD_ACCESS_FAULT: i64 = 5;
const STORE_ADDRESS_MISALIGNED: i64 = 6;
const STORE_ACCESS_FAULT: i64 = 7;
const U_MODE_ECALL: i64 = 8;
const S_MODE_ECALL: i64 = 9;
const M_MODE_ECALL: i64 = 11;
const INSTRUCTION_PAGE_FAULT: i64 = 12;
const LOAD_PAGE_FAULT: i64 = 13;
const STORE_PAGE_FAULT: i64 = 15;

#[derive(Clone, Copy, Debug)]
pub enum Interrupt {
    UModeSoftwareInterrupt,
    SModeSoftwareInterrupt,
    MModeSoftwareInterrupt,
    UserTimerInterrupt,
    SupervisorTimerInterrupt,
    MachineTimerInterrupt,
    UserExternalInterrupt,
    SupervisorExternalInterrupt,
    MachineExternalInterrupt,
    Reserved(usize),
}

impl From<i64> for Interrupt {
    fn from(i: i64) -> Interrupt {
        match i {
            U_MODE_SOFTWARE_INTERRUPT => Interrupt::UModeSoftwareInterrupt,
            S_MODE_SOFTWARE_INTERRUPT => Interrupt::SModeSoftwareInterrupt,
            M_MODE_SOFTWARE_INTERRUPT => Interrupt::MModeSoftwareInterrupt,
            USER_TIMER_INTERRUPT => Interrupt::UserTimerInterrupt,
            SUPERVISOR_TIMER_INTERRUPT => Interrupt::SupervisorTimerInterrupt,
            MACHINE_TIMER_INTERRUPT => Interrupt::MachineTimerInterrupt,
            USER_EXTERNAL_INTERRUPT => Interrupt::UserExternalInterrupt,
            SUPERVISOR_EXTERNAL_INTERRUPT => Interrupt::SupervisorExternalInterrupt,
            MACHINE_EXTERNAL_INTERRUPT => Interrupt::MachineExternalInterrupt,
            _ => Interrupt::Reserved((i & !INTERRUPT_MASK) as usize),
        }
    }
}

impl From<Interrupt> for i64 {
    fn from(i: Interrupt) -> i64 {
        match i {
            Interrupt::UModeSoftwareInterrupt => U_MODE_SOFTWARE_INTERRUPT,
            Interrupt::SModeSoftwareInterrupt => S_MODE_SOFTWARE_INTERRUPT,
            Interrupt::MModeSoftwareInterrupt => M_MODE_SOFTWARE_INTERRUPT,
            Interrupt::UserTimerInterrupt => USER_TIMER_INTERRUPT,
            Interrupt::SupervisorTimerInterrupt => SUPERVISOR_TIMER_INTERRUPT,
            Interrupt::MachineTimerInterrupt => MACHINE_TIMER_INTERRUPT,
            Interrupt::UserExternalInterrupt => USER_EXTERNAL_INTERRUPT,
            Interrupt::SupervisorExternalInterrupt => SUPERVISOR_EXTERNAL_INTERRUPT,
            Interrupt::MachineExternalInterrupt => MACHINE_EXTERNAL_INTERRUPT,
            Interrupt::Reserved(n) => n as i64 | INTERRUPT_MASK,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Exception {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault,
    StoreAddressMisaligned,
    StoreAccessFault,
    UModeEcall,
    SModeEcall,
    MModeEcall,
    InstructionPageFault,
    LoadPageFault,
    StorePageFault,
    Reserved(usize),
}

impl From<i64> for Exception {
    fn from(i: i64) -> Exception {
        match i {
            INSTRUCTION_ADDRESS_MISALIGNED => Exception::InstructionAddressMisaligned,
            INSTRUCTION_ACCESS_FAULT => Exception::InstructionAccessFault,
            ILLEGAL_INSTRUCTION => Exception::IllegalInstruction,
            BREAKPOINT => Exception::Breakpoint,
            LOAD_ADDRESS_MISALIGNED => Exception::LoadAddressMisaligned,
            LOAD_ACCESS_FAULT => Exception::LoadAccessFault,
            STORE_ADDRESS_MISALIGNED => Exception::StoreAddressMisaligned,
            STORE_ACCESS_FAULT => Exception::StoreAccessFault,
            U_MODE_ECALL => Exception::UModeEcall,
            S_MODE_ECALL => Exception::SModeEcall,
            M_MODE_ECALL => Exception::MModeEcall,
            INSTRUCTION_PAGE_FAULT => Exception::InstructionPageFault,
            LOAD_PAGE_FAULT => Exception::LoadPageFault,
            STORE_PAGE_FAULT => Exception::StorePageFault,
            _ => Exception::Reserved(i as usize),
        }
    }
}

impl From<Exception> for i64 {
    fn from(value: Exception) -> Self {
        match value {
            Exception::InstructionAddressMisaligned => INSTRUCTION_ADDRESS_MISALIGNED,
            Exception::InstructionAccessFault => INSTRUCTION_ACCESS_FAULT,
            Exception::IllegalInstruction => ILLEGAL_INSTRUCTION,
            Exception::Breakpoint => BREAKPOINT,
            Exception::LoadAddressMisaligned => LOAD_ADDRESS_MISALIGNED,
            Exception::LoadAccessFault => LOAD_ACCESS_FAULT,
            Exception::StoreAddressMisaligned => STORE_ADDRESS_MISALIGNED,
            Exception::StoreAccessFault => STORE_ACCESS_FAULT,
            Exception::UModeEcall => U_MODE_ECALL,
            Exception::SModeEcall => S_MODE_ECALL,
            Exception::MModeEcall => M_MODE_ECALL,
            Exception::InstructionPageFault => INSTRUCTION_PAGE_FAULT,
            Exception::LoadPageFault => LOAD_PAGE_FAULT,
            Exception::StorePageFault => STORE_PAGE_FAULT,
            Exception::Reserved(i) => i as i64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Trap {
    Exception(Exception),
    Interrupt(Interrupt),
}

impl From<i64> for Trap {
    fn from(value: i64) -> Trap {
        if value > 0 {
            Trap::Exception(Exception::from(value))
        } else {
            Trap::Interrupt(Interrupt::from(value))
        }
    }
}

impl From<Exception> for Trap {
    fn from(value: Exception) -> Trap {
        Trap::Exception(value)
    }
}

impl From<Interrupt> for Trap {
    fn from(value: Interrupt) -> Trap {
        Trap::Interrupt(value)
    }
}

impl From<Trap> for i64 {
    fn from(value: Trap) -> i64 {
        match value {
            Trap::Exception(i) => i.into(),
            Trap::Interrupt(i) => i.into(),
        }
    }
}
