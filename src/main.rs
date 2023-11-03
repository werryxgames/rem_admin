use std::io::{Error, ErrorKind};

#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "controller-cli")]
pub mod controller_cli;

// Two secure-random generated 8-byte unsigned integers
// Should equals to client parts
static AUTH_PARTS: [u64; 2] = [0xf61388842cb1b921, 0x9a0c109ca878b305];
static VERSION: u64 = 0;
static MIN_SUPPORTED_VERSION: u64 = 0;
static MAX_SUPPORTED_VERSION: u64 = 0;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
enum ClientCodes {
    CAuth = 0x00,
    CEAuthPart = 0x01,
    CEAuthVersion = 0x02,
    CAuthOK = 0x03,
    RTestEcho = 0x04,
    ROK = 0x05,
    RFail = 0x06,
    RFailText = 0x07,
    ROKText = 0x08,
    RAborted = 0x09,
    RBool = 0x0A,
    RNotAborted = 0x0B,
    CControl = 0x70,
    CControlAll = 0x71,
    CControlList = 0x72,
    CControlOne = 0x73,
    CControlMany = 0x74,
}

impl TryFrom<u8> for ClientCodes {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::CAuth),
            0x01 => Ok(Self::CEAuthPart),
            0x02 => Ok(Self::CEAuthVersion),
            0x03 => Ok(Self::CAuthOK),
            0x04 => Ok(Self::RTestEcho),
            0x05 => Ok(Self::ROK),
            0x06 => Ok(Self::RFail),
            0x07 => Ok(Self::RFailText),
            0x08 => Ok(Self::ROKText),
            0x09 => Ok(Self::RAborted),
            0x0A => Ok(Self::RBool),
            0x0B => Ok(Self::RNotAborted),
            0x70 => Ok(Self::CControl),
            0x71 => Ok(Self::CControlAll),
            0x72 => Ok(Self::CControlList),
            0x73 => Ok(Self::CControlOne),
            0x74 => Ok(Self::CControlMany),
            _ => Err(Error::new(ErrorKind::InvalidData, "Code not in enum")),
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
enum ServerCodes {
    SAuth = 0x00,
    SEAuthPart = 0x01,
    SEAuthVersion = 0x02,
    MTest = 0x03,
    MGui = 0x04,
    MAbort = 0x05,
    MGuiYesNo = 0x06,
    MMoveCursor = 0x07,
    MMoveCursorRel = 0x08,
    MTypeKeyboard = 0x09,
    MClipboardGet = 0x0A,
    MClipboardSet = 0x0B,
    SControlOK = 0x70,
    SEControlOff = 0x71,
    SEControlPass = 0x72,
    SControlPacket = 0x73,
}

impl TryFrom<u8> for ServerCodes {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::SAuth),
            0x01 => Ok(Self::SEAuthPart),
            0x02 => Ok(Self::SEAuthVersion),
            0x03 => Ok(Self::MTest),
            0x04 => Ok(Self::MGui),
            0x05 => Ok(Self::MAbort),
            0x06 => Ok(Self::MGuiYesNo),
            0x07 => Ok(Self::MMoveCursor),
            0x08 => Ok(Self::MMoveCursorRel),
            0x09 => Ok(Self::MTypeKeyboard),
            0x0A => Ok(Self::MClipboardGet),
            0x0B => Ok(Self::MClipboardSet),
            0x70 => Ok(Self::SControlOK),
            0x71 => Ok(Self::SEControlOff),
            0x72 => Ok(Self::SEControlPass),
            0x73 => Ok(Self::SControlPacket),
            _ => Err(Error::new(ErrorKind::InvalidData, "Code not in enum")),
        }
    }
}

#[cfg(not(feature = "server"))]
#[cfg(not(feature = "client"))]
pub fn start() {
    println!("Required either `server` or `client` feature. Rerun with `--features server/client`");
}

#[cfg(feature = "server")]
#[cfg(not(feature = "client"))]
pub fn start() {
    server::start_server();
}

#[cfg(not(feature = "server"))]
#[cfg(feature = "client")]
pub fn start() {
    client::start_client();
}

#[cfg(feature = "server")]
#[cfg(feature = "client")]
pub fn start() {
    match std::env::var_os("REM_ADMIN_SERVER") {
        Some(value) => {
            if value == "1" {
                server::start_server();
                return;
            }
        }
        None => {}
    };
    client::start_client();
}

pub fn main() {
    start();
}
