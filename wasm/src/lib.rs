use game::{Board, DEPTH};
use std::time::Instant;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn ai_response(req: &str) -> String {
    let req = serde_json::from_str(req).unwrap();
    let state = Board::from_request(req);
    let start = Instant::now();
    let op = state.min_max(DEPTH);
    eprintln!("{}\n{:?}", state, op);
    eprintln!("{}ms", start.elapsed().as_millis());
    serde_json::to_string(&op).unwrap()
}
