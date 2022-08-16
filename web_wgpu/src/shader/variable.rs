use wgpu::{BindGroupEntry, BindGroupLayoutEntry};

pub trait Variable {
    fn entry(&self, index: u32) -> Vec<BindGroupEntry>;
    fn layout_entry(&self, index: u32) -> Vec<BindGroupLayoutEntry>;
}
