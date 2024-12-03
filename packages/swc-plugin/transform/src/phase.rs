#[derive(PartialEq)]
pub enum ModulePhase {
    Register = 0,
    Runtime = 1,
}

impl From<u32> for ModulePhase {
    fn from(value: u32) -> Self {
        match value {
            0 => ModulePhase::Register,
            1 => ModulePhase::Runtime,
            _ => panic!("invalid u32 value for ModulePhase"),
        }
    }
}
