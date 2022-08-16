use anyhow::*;
use std::mem::size_of;
use wgpu::{Extent3d, TextureDimension, TextureFormat};

pub struct FloatTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

pub trait Format {
    fn format() -> TextureFormat;
}

impl Format for f32 {
    fn format() -> TextureFormat {
        TextureFormat::R32Float
    }
}
impl Format for [f32; 2] {
    fn format() -> TextureFormat {
        TextureFormat::Rg32Float
    }
}
impl Format for [f32; 4] {
    fn format() -> TextureFormat {
        TextureFormat::Rgba32Float
    }
}

pub trait Dimensions {
    fn texture_dimension() -> TextureDimension;
    fn size(&self) -> Extent3d;
    fn row_length(&self) -> u32;
    fn row_count(&self) -> u32;
}

impl Dimensions for u32 {
    fn texture_dimension() -> TextureDimension {
        wgpu::TextureDimension::D1
    }

    fn size(&self) -> Extent3d {
        Extent3d {
            width: *self,
            height: 1,
            depth_or_array_layers: 1,
        }
    }

    fn row_length(&self) -> u32 {
        *self
    }

    fn row_count(&self) -> u32 {
        1
    }
}
impl Dimensions for (u32, u32) {
    fn texture_dimension() -> TextureDimension {
        wgpu::TextureDimension::D2
    }

    fn size(&self) -> Extent3d {
        Extent3d {
            width: self.0,
            height: self.1,
            depth_or_array_layers: 1,
        }
    }

    fn row_length(&self) -> u32 {
        self.0
    }

    fn row_count(&self) -> u32 {
        self.1
    }
}
impl Dimensions for (u32, u32, u32) {
    fn texture_dimension() -> TextureDimension {
        wgpu::TextureDimension::D3
    }

    fn size(&self) -> Extent3d {
        Extent3d {
            width: self.0,
            height: self.1,
            depth_or_array_layers: self.2,
        }
    }

    fn row_length(&self) -> u32 {
        self.0
    }

    fn row_count(&self) -> u32 {
        self.1 * self.2
    }
}

impl FloatTexture {
    pub fn from_f32<T: Format + bytemuck::Pod, U: Dimensions>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vec: &[T],
        dimensions: U,
        label: &str,
    ) -> Result<Self> {
        let size = dimensions.size();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: U::texture_dimension(),
            format: T::format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &bytemuck::cast_slice(&vec),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(
                    dimensions.row_length() * size_of::<T>() as u32,
                ),
                rows_per_image: std::num::NonZeroU32::new(dimensions.row_count()),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}
