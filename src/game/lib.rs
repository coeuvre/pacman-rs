extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate serde;

use failure::Error;

// pub enum Evnet {}

// pub struct World {}

// pub struct EntityId(u32);

// pub struct Entity {
//     pub id: EntityId,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct PacMan {
    count: u32,
}

impl PacMan {
    pub fn new() -> Result<PacMan, Error> {
        Ok(PacMan { count: 0 })
    }
}

#[repr(C)]
pub struct Snapshot {
    buf: *mut u8,
    len: usize,
}

#[no_mangle]
pub extern "C" fn take_snapshot() -> Snapshot {
    let game = PacMan::new().unwrap();
    let encoded = bincode::serialize(&game).unwrap();
    let mut slice = encoded.into_boxed_slice();
    let len = slice.len();
    let buf = slice.as_mut_ptr();
    std::mem::forget(slice);
    Snapshot { buf, len }
}

#[no_mangle]
pub unsafe extern "C" fn drop_snapshot(snapshot: Snapshot) {
    let slice = std::slice::from_raw_parts_mut(snapshot.buf, snapshot.len);
    Box::from_raw(slice);
}

#[no_mangle]
pub unsafe extern "C" fn restore_snapshot(snapshot: Snapshot) {
    let slice = std::slice::from_raw_parts_mut(snapshot.buf, snapshot.len);
    let game: PacMan = bincode::deserialize(slice).unwrap();
    println!("{:?}", game)
}

#[no_mangle]
pub fn run(msg: &str) {
    println!("{}, game 1", msg);
}
