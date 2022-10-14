use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Transport {
    Connect,
    Disconnect,
    Ping,
}
