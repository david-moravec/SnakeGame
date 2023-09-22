#![deny(missing_docs)]

//! A Gfx texture representation that works nicely with Piston libraries.

extern crate gfx;
extern crate gfx_core;
extern crate texture;
extern crate image;

pub use texture::*;

use std::path::Path;
use image::{
    DynamicImage,
    ImageError,
    RgbaImage,
};
use gfx::format::{Srgba8, R8_G8_B8_A8};

/// Context required to create and update textures.
pub struct TextureContext<F, R, C>
    where F: gfx::Factory<R>,
          R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
{
    /// A factory to create textures.
    pub factory: F,
    /// An encoder to update textures.
    pub encoder: gfx::Encoder<R, C>,
}

/// Create creation or update error.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// An error when creating texture.
    Create(gfx::CombinedError),
    /// An error when updating texture.
    Update(gfx::UpdateError<[u16; 3]>),
    /// An error when performing an image operation.
    Image(String),
}

impl From<gfx::UpdateError<[u16; 3]>> for Error {
    fn from(val: gfx::UpdateError<[u16; 3]>) -> Error {
        Error::Update(val)
    }
}

impl From<gfx::texture::CreationError> for Error {
    fn from(val: gfx::texture::CreationError) -> Error {
        Error::Create(val.into())
    }
}

impl From<ImageError> for Error {
    fn from(val: ImageError) -> Error {
        Error::Image(val.to_string())
    }
}

impl From<gfx::ResourceViewError> for Error {
    fn from(val: gfx::ResourceViewError) -> Error {
        Error::Create(val.into())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        use std::fmt::{Display, Debug};

        match *self {
            Error::Create(ref err) => Display::fmt(err, w),
            Error::Update(ref err) => Debug::fmt(err, w),
            Error::Image(ref err) => Display::fmt(err, w),
        }
    }
}

impl std::error::Error for Error {}

/// Flip settings.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Flip {
    /// Does not flip.
    None,
    /// Flips image vertically.
    Vertical,
    /// Flips image horizontally.
    Horizontal,
    /// Flips image both vertically and horizontally.
    Both,
}

/// Represents a texture.
#[derive(Clone, Debug, PartialEq)]
pub struct Texture<R> where R: gfx::Resources {
    /// Pixel storage for texture.
    pub surface: gfx::handle::Texture<R, R8_G8_B8_A8>,
    /// Sampler for texture.
    pub sampler: gfx::handle::Sampler<R>,
    /// View used by shader.
    pub view: gfx::handle::ShaderResourceView<R, [f32; 4]>
}

impl<R: gfx::Resources> Texture<R> {
    /// Returns empty texture.
    pub fn empty<F, C>(context: &mut TextureContext<F, R, C>) -> Result<Self, Error>
        where F: gfx::Factory<R>,
              C: gfx::CommandBuffer<R>,
    {
        CreateTexture::create(context, Format::Rgba8, &[0u8; 4], [1, 1],
                              &TextureSettings::new())
    }

    /// Creates a texture from path.
    pub fn from_path<F, C, P>(
        context: &mut TextureContext<F, R, C>,
        path: P,
        flip: Flip,
        settings: &TextureSettings,
    ) -> Result<Self, Error>
        where F: gfx::Factory<R>,
              C: gfx::CommandBuffer<R>,
              P: AsRef<Path>
    {
        let img = image::open(path)?;

        let img = match img {
            DynamicImage::ImageRgba8(img) => img,
            img => img.to_rgba8()
        };

        let img = match flip {
            Flip::Vertical => image::imageops::flip_vertical(&img),
            Flip::Horizontal => image::imageops::flip_horizontal(&img),
            Flip::Both => {
                let img = image::imageops::flip_vertical(&img);
                image::imageops::flip_horizontal(&img)
            }
            Flip::None => img,
        };

        Texture::from_image(context, &img, settings)
    }

    /// Creates a texture from image.
    pub fn from_image<F, C>(
        context: &mut TextureContext<F, R, C>,
        img: &RgbaImage,
        settings: &TextureSettings
    ) -> Result<Self, Error>
        where F: gfx::Factory<R>,
              C: gfx::CommandBuffer<R>,
    {
        let (width, height) = img.dimensions();
        CreateTexture::create(context, Format::Rgba8,
                              img, [width, height], settings)
    }

    /// Creates texture from memory alpha.
    pub fn from_memory_alpha<F, C>(
        context: &mut TextureContext<F, R, C>,
        buffer: &[u8],
        width: u32,
        height: u32,
        settings: &TextureSettings
    ) -> Result<Self, Error>
        where F: gfx::Factory<R>,
              C: gfx::CommandBuffer<R>,
    {
        if width == 0 || height == 0 {
            return Texture::empty(context);
        }

        let size = [width, height];
        let buffer = texture::ops::alpha_to_rgba8(buffer, size);
        CreateTexture::create(context, Format::Rgba8, &buffer, size, settings)
    }

    /// Updates the texture with an image.
    pub fn update<F, C>(
        &mut self,
        context: &mut TextureContext<F, R, C>,
        img: &RgbaImage
    ) -> Result<(), Error>
        where F: gfx::Factory<R>,
              C: gfx::CommandBuffer<R>
    {
        let (width, height) = img.dimensions();
        let offset = [0, 0];
        let size = [width, height];
        UpdateTexture::update(self, context, Format::Rgba8, img, offset, size)
    }
}

impl<F, R> TextureOp<F> for Texture<R> where R: gfx::Resources {
    type Error = Error;
}

impl<F, R, C> CreateTexture<TextureContext<F, R, C>> for Texture<R>
    where F: gfx::Factory<R>,
          R: gfx::Resources,
          C: gfx::CommandBuffer<R>,
{
    fn create<S: Into<[u32; 2]>>(
        context: &mut TextureContext<F, R, C>,
        _format: Format,
        memory: &[u8],
        size: S,
        settings: &TextureSettings
    ) -> Result<Self, Self::Error> {
        let factory = &mut context.factory;
        // Modified `Factory::create_texture_immutable_u8` for dynamic texture.
        fn create_texture<T, F, R>(
            factory: &mut F,
            kind: gfx::texture::Kind,
            data: &[&[u8]]
        ) -> Result<(
            gfx::handle::Texture<R, T::Surface>,
            gfx::handle::ShaderResourceView<R, T::View>
        ), Error>
            where F: gfx::Factory<R>,
                  R: gfx::Resources,
                  T: gfx::format::TextureFormat
        {
            use gfx::{format, texture};
            use gfx::memory::{Usage, Bind};
            use gfx_core::memory::Typed;
            use gfx_core::texture::Mipmap;

            let surface = <T::Surface as format::SurfaceTyped>::get_surface_type();
            let num_slices = kind.get_num_slices().unwrap_or(1) as usize;
            let num_faces = if kind.is_cube() {6} else {1};
            let desc = texture::Info {
                kind: kind,
                levels: (data.len() / (num_slices * num_faces)) as texture::Level,
                format: surface,
                bind: Bind::SHADER_RESOURCE,
                usage: Usage::Dynamic,
            };
            let cty = <T::Channel as format::ChannelTyped>::get_channel_type();
            let raw = factory.create_texture_raw(desc, Some(cty), Some((data, Mipmap::Provided)))?;
            let levels = (0, raw.get_info().levels - 1);
            let tex = Typed::new(raw);
            let view = factory.view_texture_as_shader_resource::<T>(
                &tex, levels, format::Swizzle::new())?;
            Ok((tex, view))
        }

        let size = size.into();
        let (width, height) = (size[0] as u16, size[1] as u16);
        let tex_kind = gfx::texture::Kind::D2(width, height,
            gfx::texture::AaMode::Single);

        // FIXME Use get_min too. gfx has only one filter setting for both.
        let filter_method = match settings.get_mag() {
            texture::Filter::Nearest => gfx::texture::FilterMethod::Scale,
            texture::Filter::Linear => gfx::texture::FilterMethod::Bilinear,
        };

        let wrap_mode_u = match settings.get_wrap_u() {
            Wrap::ClampToEdge => gfx::texture::WrapMode::Clamp,
            Wrap::ClampToBorder => gfx::texture::WrapMode::Border,
            Wrap::Repeat => gfx::texture::WrapMode::Tile,
            Wrap::MirroredRepeat => gfx::texture::WrapMode::Mirror,
        };

        let wrap_mode_v = match settings.get_wrap_v() {
            Wrap::ClampToEdge => gfx::texture::WrapMode::Clamp,
            Wrap::ClampToBorder => gfx::texture::WrapMode::Border,
            Wrap::Repeat => gfx::texture::WrapMode::Tile,
            Wrap::MirroredRepeat => gfx::texture::WrapMode::Mirror,
        };

        let mut sampler_info = gfx::texture::SamplerInfo::new(
            filter_method,
            wrap_mode_u
        );
        sampler_info.wrap_mode.1 = wrap_mode_v;
        sampler_info.border = settings.get_border_color().into();

        let (surface, view) = create_texture::<Srgba8, F, R>(
            factory, tex_kind, &[memory])?;
        let sampler = factory.create_sampler(sampler_info);
        Ok(Texture { surface: surface, sampler: sampler, view: view })
    }
}

impl<F, R, C> UpdateTexture<TextureContext<F, R, C>> for Texture<R>
    where F: gfx::Factory<R>,
          R: gfx::Resources,
          C: gfx::CommandBuffer<R>
{
    fn update<O, S>(
        &mut self,
        context: &mut TextureContext<F, R, C>,
        format: Format,
        memory: &[u8],
        offset: O,
        size: S,
    ) -> Result<(), Self::Error>
        where O: Into<[u32; 2]>,
              S: Into<[u32; 2]>,
    {
        let encoder = &mut context.encoder;
        let offset = offset.into();
        let size = size.into();
        let tex = &self.surface;
        let face = None;
        let img_info = gfx::texture::ImageInfoCommon {
            xoffset: offset[0] as u16,
            yoffset: offset[1] as u16,
            zoffset: 0,
            width: size[0] as u16,
            height: size[1] as u16,
            depth: 0,
            format: (),
            mipmap: 0,
        };
        let data = gfx::memory::cast_slice(memory);

        match format {
            Format::Rgba8 => {
                use gfx::format::Rgba8;
                encoder.update_texture::<_, Rgba8>(tex, face, img_info, data).map_err(Into::into)
            },
        }
    }
}

impl<R> ImageSize for Texture<R> where R: gfx::Resources {
    #[inline(always)]
    fn get_size(&self) -> (u32, u32) {
        let (w, h, _, _) = self.surface.get_info().kind.get_dimensions();
        (w as u32, h as u32)
    }
}
