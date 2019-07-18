use serde::{Serialize, Deserialize};

// In this file described messages of the server side actors
pub mod storage {
    pub mod commands {

        #[derive(Serialize, Deserialize, Debug)]
        pub struct GetData { pub c: u32 }
    }

    pub mod responses {

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Data { pub value: String }
    }
}