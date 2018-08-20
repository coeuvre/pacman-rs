// //! Basic hello world example.

// extern crate ggez;

// use ggez::conf;
// use ggez::event;
// use ggez::graphics;
// use ggez::{Context, GameResult};
// use std::env;
// use std::path;

// // First we make a structure to contain the game's state
// struct MainState {
//     text: graphics::Text,
//     frames: usize,
// }

// impl MainState {
//     fn new(ctx: &mut Context) -> GameResult<MainState> {
//         // The ttf file will be in your resources directory. Later, we
//         // will mount that directory so we can omit it in the path here.
//         let font = graphics::Font::new(ctx, "/DejaVuSerif.ttf", 48)?;
//         let text = graphics::Text::new(ctx, "Hello world!", &font)?;

//         let s = MainState { text, frames: 0 };
//         Ok(s)
//     }
// }

// // Then we implement the `ggez:event::EventHandler` trait on it, which
// // requires callbacks for updating and drawing the game state each frame.
// //
// // The `EventHandler` trait also contains callbacks for event handling
// // that you can override if you wish, but the defaults are fine.
// impl event::EventHandler for MainState {
//     fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
//         Ok(())
//     }

//     fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
//         graphics::clear(ctx);

//         // Drawables are drawn from their top-left corner.
//         let dest_point = graphics::Point2::new(10.0, 10.0);
//         graphics::draw(ctx, &self.text, dest_point, 0.0)?;
//         graphics::present(ctx);

//         self.frames += 1;
//         if (self.frames % 100) == 0 {
//             println!("FPS: {}", ggez::timer::get_fps(ctx));
//         }

//         Ok(())
//     }
// }

// // Now our main function, which does three things:
// //
// // * First, create a new `ggez::conf::Conf`
// // object which contains configuration info on things such
// // as screen resolution and window title.
// // * Second, create a `ggez::game::Game` object which will
// // do the work of creating our MainState and running our game.
// // * Then, just call `game.run()` which runs the `Game` mainloop.
// pub fn main() {
//     let c = conf::Conf::new();
//     let ctx = &mut Context::load_from_conf("helloworld", "ggez", c).unwrap();

//     // We add the CARGO_MANIFEST_DIR/resources to the filesystem's path
//     // so that ggez will look in our cargo project directory for files.
//     if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
//         let mut path = path::PathBuf::from(manifest_dir);
//         path.pop();
//         path.pop();
//         path.push("resources");
//         ctx.filesystem.mount(&path, true);
//     }

//     let state = &mut MainState::new(ctx).unwrap();
//     if let Err(e) = event::run(ctx, state) {
//         println!("Error encountered: {}", e);
//     } else {
//         println!("Game exited cleanly.");
//     }
// }

extern crate failure;
extern crate libloading;

use std::fs;
use std::time::{Duration, SystemTime};

use failure::Error;
use libloading::{Library, Symbol};

const LIB_PATH: &str = "./target/debug/libgame.dylib";

struct Lib {
    path: String,
    lib: Option<Library>,
    modified: SystemTime,
}

impl Lib {
    pub fn load(path: &str) -> Result<Lib, Error> {
        let lib = Library::new(LIB_PATH)?;
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        Ok(Lib {
            path: path.to_string(),
            lib: Some(lib),
            modified,
        })
    }

    pub unsafe fn get<T>(&self, symbol: &[u8]) -> Result<Symbol<T>, Error> {
        Ok(self.lib.as_ref().unwrap().get(symbol)?)
    }

    pub fn is_modified(&self) -> Result<bool, Error> {
        let metadata = fs::metadata(&self.path)?;
        let modified = metadata.modified()?;

        Ok(modified > self.modified)
    }

    pub fn reload(&mut self) -> Result<(), Error> {
        let lib = self.lib.take().unwrap();
        std::mem::drop(lib);
        let lib = Library::new(&self.path)?;
        let metadata = fs::metadata(&self.path)?;
        let modified = metadata.modified()?;
        self.lib = Some(lib);
        self.modified = modified;
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let mut lib = Lib::load(LIB_PATH)?;

    loop {
        if lib.is_modified()? {
            println!("Lib changed.");

            // println!("Take snapshot...");
            // let snapshot = unsafe {
            //     let take_snapshot: Symbol<extern "C" fn() -> Snapshot> = lib.get(b"take_snapshot")?;
            //     take_snapshot()
            // };

            println!("Reload lib...");
            lib.reload()?;

            // println!("Restore snapshot...");
            // unsafe {
            //     let restore_snapshot: Symbol<extern "C" fn(Snapshot)> =
            //         lib.get(b"restore_snapshot")?;
            //     restore_snapshot(snapshot);

            //     let drop_snapshot: Symbol<extern "C" fn(Snapshot)> = lib.get(b"drop_snapshot")?;
            //     drop_snapshot(snapshot);
            // }
        }

        unsafe {
            let func: Symbol<fn(&str)> = lib.get(b"run")?;
            func("Main");
        }

        ::std::thread::sleep(Duration::from_millis(1000));
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Snapshot {
    buf: *mut u8,
    len: usize,
}
