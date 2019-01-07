use std::sync::Mutex;
use lazy_static::lazy_static;

use crate::tree::*;

pub fn get_performance_counter() -> u64 {
    unsafe { sdl2_sys::SDL_GetPerformanceCounter() }
}

pub fn get_performance_frequency() -> u64 {
    unsafe { sdl2_sys::SDL_GetPerformanceFrequency() }
}

lazy_static! {
    pub static ref PROFILER: Mutex<Profiler> = Mutex::new(Profiler::new());
}

pub struct Profiler {
    count: u64,
    frequency: u64,
    frames: Vec<Frame>,
    next_frame_index: usize,
    current_frame_index: Option<usize>,
}

impl Profiler {
    pub fn new() -> Self {
        Profiler {
            count: 0,
            frequency: get_performance_frequency(),
            frames: Vec::with_capacity(1024),
            next_frame_index: 0,
            current_frame_index: None,
        }
    }

    pub fn begin_frame(&mut self) {
        assert!(self.current_frame_index.is_none());

        self.count = self.count + 1;

        if self.frames.len() < self.frames.capacity() {
            self.frames.push(Frame::new());
            self.current_frame_index = Some(self.frames.len() - 1);
        } else {
            self.current_frame_index = Some(self.next_frame_index);
        };
        self.next_frame_index = (self.next_frame_index + 1) % self.frames.capacity();
        let current_frame = &mut self.frames[self.current_frame_index.unwrap()];
        current_frame.frequency = self.frequency;
        current_frame.count = self.count;
        current_frame.begin();
    }

    pub fn end_frame(&mut self) {
        assert!(self.current_frame_index.is_some());

        let current_frame = &mut self.frames[self.current_frame_index.unwrap()];
        current_frame.end();
        self.current_frame_index = None;
    }

    pub fn current_frame_mut(&mut self) -> Option<&mut Frame> {
        if let Some(index) = self.current_frame_index {
            self.frames.get_mut(index)
        } else {
            None
        }
    }

    pub fn current_frame(&self) -> Option<&Frame> {
        if let Some(index) = self.current_frame_index {
            self.frames.get(index)
        } else {
            None
        }
    }

    pub fn last_frame(&self) -> Option<&Frame> {
        let current_index = if let Some(current_frame_index) = self.current_frame_index {
            current_frame_index
        } else {
            self.next_frame_index
        };

        if current_index > 0 || self.frames.len() > 0 {
            self.frames.get((current_index + self.frames.capacity() - 1) % self.frames.capacity())
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct Frame {
    count: u64,
    frequency: u64,
    block_tree: Option<Tree<Block>>,
    current_block_id: usize,
}

fn root_block(frequency: u64, current_counter: u64) -> Block {
    Block {
        file: "",
        line: 0,
        name: "".to_string(),
        frequency,
        begin: current_counter,
        end: current_counter,
    }
}

impl Frame {
    fn new() -> Self {
        Frame {
            count: 0,
            frequency: 0,
            block_tree: None,
            current_block_id: 0,
        }
    }

    fn begin(&mut self) {
        assert_eq!(self.current_block_id, 0);
        let current_counter = get_performance_counter();
        let root_block = root_block(self.frequency, current_counter);
        if let Some(ref mut block_tree) = self.block_tree {
            block_tree.clear(root_block);
        } else {
            self.block_tree = Some(Tree::new(root_block));
        }
    }

    fn end(&mut self) {
        assert_eq!(self.current_block_id, 0);
        let current_counter = get_performance_counter();
        let block_tree = self.block_tree.as_mut().unwrap();
        let mut root_block = block_tree.get_mut(0).unwrap();
        root_block.data().end = current_counter;
    }

    pub fn begin_counter(&self) -> u64 {
        let block_tree = self.block_tree.as_ref().unwrap();
        let root_block_node = block_tree.get(0).unwrap();
        root_block_node.data().begin
    }

    pub fn delta(&self) -> f32 {
        let block_tree = self.block_tree.as_ref().unwrap();
        let root_block_node = block_tree.get(0).unwrap();
        root_block_node.data().delta()
    }

    pub fn dfs_block_iter(&self) -> Dfs<Block> {
        self.block_tree.as_ref().unwrap().dfs_iter()
    }

    pub fn open_block<N>(&mut self, file: &'static str, line: u32, name: N) where N: Into<String> {
        let block_tree = self.block_tree.as_mut().unwrap();

        let current_counter = get_performance_counter();
        let new_block = Block {
            file,
            line,
            name: name.into(),
            frequency: self.frequency,
            begin: current_counter,
            end: current_counter,
        };
        self.current_block_id = block_tree.push(self.current_block_id, new_block).unwrap();
    }

    pub fn close_block(&mut self) {
        let block_tree = self.block_tree.as_mut().unwrap();
        let current_counter = get_performance_counter();
        let mut current_block_node = block_tree.get_mut(self.current_block_id).unwrap();
        current_block_node.data().end = current_counter;
        self.current_block_id = current_block_node.parent().unwrap().id();
    }
}

#[derive(Clone)]
pub struct Block {
    file: &'static str,
    line: u32,
    name: String,
    frequency: u64,
    begin: u64,
    end: u64,
}

impl Block {
    pub fn delta(&self) -> f32 {
        (self.end - self.begin) as f32 / self.frequency as f32
    }

    pub fn file(&self) -> &str {
        self.file
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

macro_rules! profile_block {
    ($block:block) => {
        profile_block!("", $block);
    };

    ($name:ident, $block:block) => {
        profile_block!(stringify!($name), $block);
    };

    ($name:expr, $block:block) => {
        PROFILER.lock().unwrap().current_frame_mut().unwrap().open_block(file!(), line!(), $name);
        $block;
        PROFILER.lock().unwrap().current_frame_mut().unwrap().close_block();
    }
}
