use prometheus::{self, Encoder, IntCounterVec, TextEncoder};

lazy_static! {
    static ref FILE_COUNTER: IntCounterVec =
        register_int_counter_vec!("file_total", "Files procecssed", &["stage"]).unwrap();
}
