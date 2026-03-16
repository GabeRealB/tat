#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    X86_64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cpu {
    pub arch: Arch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsTag {
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Os {
    tag: OsTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiTag {
    Msvc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Api {
    pub tag: ApiTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbiTag {
    Win64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Abi {
    pub tag: AbiTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Target {
    pub cpu: Cpu,
    pub os: Os,
    pub api: Api,
    pub abi: Abi,
}

impl Target {
    pub fn get_host_target() -> Self {
        // TODO: Run target detection
        Target {
            cpu: Cpu { arch: Arch::X86_64 },
            os: Os {
                tag: OsTag::Windows,
            },
            api: Api { tag: ApiTag::Msvc },
            abi: Abi { tag: AbiTag::Win64 },
        }
    }

    pub fn get_max_align(&self) -> u16 {
        match self.cpu.arch {
            Arch::X86_64 => 16,
        }
    }

    pub fn get_pointer_bit_width(&self) -> u16 {
        match self.cpu.arch {
            Arch::X86_64 => 64,
        }
    }
}
