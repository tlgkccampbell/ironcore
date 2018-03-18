use std::io;
use std::ffi;
use std::env;

pub type IronCoreResult<T> = Result<T, IronCoreError>;

#[derive(Debug)]
pub enum IronCoreError {
    IoError(io::Error),
    NulError(ffi::NulError),
    VarError(env::VarError),
    HresultError(HRESULT),
    InvalidExePath,
}

impl From<io::Error> for IronCoreError {
    fn from(e: io::Error) -> IronCoreError {
        IronCoreError::IoError(e)
    }
}

impl From<ffi::NulError> for IronCoreError {
    fn from(e: ffi::NulError) -> IronCoreError {
        IronCoreError::NulError(e)
    }
}

impl From<env::VarError> for IronCoreError {
    fn from(e: env::VarError) -> IronCoreError {
        IronCoreError::VarError(e)
    }
}

#[derive(Debug)]
pub enum HRESULT {
    Ok,
    FileNotFound,
    CorETypeLoad,
    CorEEntryPointNotFound,
    CorEDLLNotFound,
    Unknown(u32),
}

impl HRESULT {
    pub fn succeeded(&self) -> bool {
        match *self {
            HRESULT::Ok => true,
            HRESULT::FileNotFound => false,
            HRESULT::CorETypeLoad => false,
            HRESULT::CorEEntryPointNotFound => false,
            HRESULT::CorEDLLNotFound => false,
            HRESULT::Unknown(hr) => (hr as i32) >= 0,
        }
    }
    pub fn failed(&self) -> bool {
        match *self {
            HRESULT::Ok => false,
            HRESULT::FileNotFound => true,
            HRESULT::CorETypeLoad => true,
            HRESULT::CorEEntryPointNotFound => true,
            HRESULT::CorEDLLNotFound => true,
            HRESULT::Unknown(hr) => (hr as i32) < 0,
        }
    }
    pub fn check(self) -> IronCoreResult<()> {
        if self.failed() {
            return Err(IronCoreError::HresultError(self));
        }
        return Ok(());
    }
}

impl From<u32> for HRESULT {
    fn from(hr: u32) -> HRESULT {
        match hr {
            0 => HRESULT::Ok,
            0x80070002 => HRESULT::FileNotFound,
            0x80131522 => HRESULT::CorETypeLoad,
            0x80131523 => HRESULT::CorEEntryPointNotFound,
            0x80131524 => HRESULT::CorEDLLNotFound,
            _ => HRESULT::Unknown(hr),
        }
    }
}
