use crate::numeric;

#[repr(C)]
pub struct Elf32Ehdr {
    pub e_ident: EIdent,
    pub e_type: EType,
    pub e_machine: EMachine,
    pub e_version: u32,
    pub e_entry: u32,
    pub e_phoff: u32,
    pub e_shoff: u32,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}
#[repr(C)]
pub struct Elf64Ehdr {
    pub e_ident: EIdent,
    pub e_type: EType,
    pub e_machine: EMachine,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

numeric! {
    pub enum EType : u16 {
        None = 0,
        Rel = 1,
        Exec = 2,
        Dyn = 3,
        Core = 4,
        Loos = 0xfe00,
        Hios = 0xfeff,
        Loproc = 0xff00,
        Hiproc = 0xffff,
    }
}

numeric! {
    @fallback
    pub enum EMachine : u16 {
        None = 0,               // No machine
        M32 = 1,                // AT&T WE 32100
        Sparc = 2,              // SPARC
        Intel386 = 3,           // Intel 80386
        Motorola68K = 4,        // Motorola 68000
        Motorola88K = 5,        // Motorola 88000
        Intel860 = 7,           // Intel 80860
        Mips = 8,               // MIPS I Architecture
        S370 = 9,               // IBM System/370 Processor
        MipsRs3Le = 10,         // MIPS RS3000 Little-endian
        PaRisc = 15,            // Hewlett-Packard PA-RISC
        VPP500 = 17,            // Fujitsu VPP500
        SPARC32PLUS = 18,       // Enhanced instruction set SPARC
        Intel960 = 19,          // Intel 80960
        PowerPC = 20,           // PowerPC
        PPC64 = 21,             // 64-bit PowerPC
        S390 = 22,              //IBM System/390 Processor
        V800 = 36,              // NEC V800
        FR20 = 37,              // Fujitsu FR20
        RH32 = 38,              // TRW RH-32
        Rce = 39,               // Motorola RCE
        Arm = 40,               // Advanced RISC Machines ARM
        Alpha = 41,             // Digital Alpha
        SH = 42,                // Hitachi SH
        SPARCV9 = 43,           // SPARC Version 9
        TroCore = 44,           // Siemens TriCore embedded processor
        Arc = 45,               // Argonaut RISC Core, Argonaut Technologies Inc.
        H8_300 = 46,            // Hitachi H8/300
        H8_300H = 47,           // Hitachi H8/300H
        H8S = 48,               // Hitachi H8S
        H8_500 = 49,            // Hitachi H8/500
        Ia64 = 50,              // Intel IA-64 processor architecture
        MipsX = 51,             // Stanford MIPS-X
        ColdFire = 52,          // Motorola ColdFire
        Motorola68HC12 = 53,    // Motorola M68HC12
        Mma = 54,               // Fujitsu MMA Multimedia Accelerator
        Pcp = 55,               // Siemens PCP
        Ncpu = 56,              // Sony nCPU embedded RISC processor
        NDR1 = 57,              // Denso NDR1 microprocessor
        StarCore = 58,          // Motorola Star*Core processor
        ME16 = 59,              // Toyota ME16 processor
        ST100 = 60,             // STMicroelectronics ST100 processor
        TinyJ = 61,             // Advanced Logic Corp. TinyJ embedded processor family
        X86_64 = 62,            // AMD x86-64 architecture
        Pdsp = 63,              // Sony DSP Processor
        PDP10 = 64,             // Digital Equipment Corp. PDP-10
        PDP11 = 65,             // Digital Equipment Corp. PDP-11
        FX66 = 66,              // Siemens FX66 microcontroller
        ST9PLUS = 67,           // STMicroelectronics ST9+ 8/16 bit microcontroller
        ST7 = 68,               // STMicroelectronics ST7 8-bit microcontroller
        Motorola68HC16 = 69,    // Motorola MC68HC16 Microcontroller
        Motorola68HC11 = 70,    // Motorola MC68HC11 Microcontroller
        Motorola68HC08 = 71,    // Motorola MC68HC08 Microcontroller
        Motorola68HC05 = 72,    // Motorola MC68HC05 Microcontroller
        SVx = 73,               // Silicon Graphics SVx
        ST19 = 74,              // STMicroelectronics ST19 8-bit microcontroller
        Vax = 75,               // Digital VAX
        Cris = 76,              // Axis Communications 32-bit embedded processor
        Javelin = 77,           // Infineon Technologies 32-bit embedded processor
        FirePath = 78,          // Element 14 64-bit DSP Processor
        Zsp = 79,               // LSI Logic 16-bit DSP Processor
        Mmix = 80,              // Donald Knuth's educational 64-bit processor
        Huany = 81,             // Harvard University machine-independent object files
        Prism = 82,             // SiTera Prism
        Avr = 83,               // Atmel AVR 8-bit microcontroller
        FR30 = 84,              // Fujitsu FR30
        D10V = 85,              // Mitsubishi D10V
        D30V = 86,              // Mitsubishi D30V
        V850 = 87,              // NEC v850
        M32R = 88,              // Mitsubishi M32R
        MN10300 = 89,           // Matsushita MN10300
        MN10200 = 90,           // Matsushita MN10200
        PicoJava = 91,          // picoJava
        OpenRisc = 92,          // OpenRISC 32-bit embedded processor
        ArcA5 = 93,             // ARC Cores Tangent-A5
        Xtensa = 94,            // Tensilica Xtensa Architecture
        VideoCore = 95,         // Alphamosaic VideoCore processor
        TmmGpp = 96,            // Thompson Multimedia General Purpose Processor
        NS32K = 97,             // National Semiconductor 32000 series
        Tpc = 98,               // Tenor Network TPC processor
        SNP1K = 99,             // Trebia SNP 1000 processor
        ST200 = 100,            // STMicroelectronics (www.st.com) ST200 microcontroller
    }
}

numeric! {
    pub enum EVersion : u32 {
        None	= 0,	// Invalid version
        Current = 1,	// Current version
    }
}

#[repr(transparent)]
pub struct EIdent(pub [u8; 16]);

impl EIdent {
    pub fn is_elf(&self) -> bool {
        self.0[0] == 0x7f && self.0[1] == b'E' && self.0[2] == b'L' && self.0[3] == b'F'
    }

    pub fn class(&self) -> Result<Class, u8> {
        self.0[4].try_into()
    }

    pub fn set_class(&mut self, class: Class) {
        self.0[4] = class.into();
    }

    pub fn data(&self) -> Result<Endianess, u8> {
        self.0[5].try_into()
    }

    pub fn set_data(&mut self, data: Endianess) {
        self.0[5] = data.into();
    }

    pub fn version(&self) -> u8 {
        self.0[6]
    }

    pub fn set_version(&mut self, version: u8) {
        self.0[6] = version;
    }

    pub fn os_abi(&self) -> OsAbi {
        self.0[7].into()
    }

    pub fn set_os_abi(&mut self, os_abi: OsAbi) {
        self.0[7] = os_abi.into();
    }

    pub fn abi_version(&self) -> u8 {
        self.0[8]
    }

    pub fn set_abi_version(&mut self, version: u8) {
        self.0[8] = version;
    }

    pub fn pad(&self) -> u8 {
        self.0[9]
    }

    pub fn set_pad(&mut self, pad: u8) {
        self.0[9] = pad;
    }
    pub fn n_ident(&self) -> u8 {
        self.0[15]
    }

    pub fn set_n_ident(&mut self, n_ident: u8) {
        self.0[15] = n_ident;
    }
}

numeric! {
    pub enum Class : u8 {
        None = 0,	    // Invalid class
        Class32	= 1,	// 32-bit objects
        Class64	= 2,	// 64-bit objects
    }
}

numeric! {
    @fallback
    pub enum OsAbi : u8 {
        None = 0,
        HpUx = 1,
        NetBSD = 2,
        Linux = 3,
        Solaris = 6,
        Aix = 7,
        Iris = 8,
        FreeBSD = 9,
        TRU64 = 10,
        Modesto = 11,
        OpenBSD = 12,
        OpenVMS = 13,
        Nsk = 14,
    }
}

numeric! {
    pub enum Endianess: u8 {
        None = 0,
        Lsb = 1,
        Msb = 2,
    }
}
