use bevy_asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};
use bevy_ecs::prelude::{FromWorld, World};
use thiserror::Error;

use crate::{
    renderer::RenderDevice,
    texture::{Image, ImageFormat, ImageType, TextureError},
};

use super::{CompressedImageFormats, ImageSampler};
use serde::{Deserialize, Serialize};

/// Loader for images that can be read by the `image` crate.
#[derive(Clone)]
pub struct ImageLoader {
    supported_compressed_formats: CompressedImageFormats,
}

pub(crate) const IMG_FILE_EXTENSIONS: &[&str] = &[
    "basis",
    "dds",
    "ktx2",
];

#[derive(Serialize, Deserialize, Default, Debug)]
pub enum ImageFormatSetting {
    #[default]
    FromExtension,
    Format(ImageFormat),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageLoaderSettings {
    pub format: ImageFormatSetting,
    pub is_srgb: bool,
    pub sampler: ImageSampler,
}

impl Default for ImageLoaderSettings {
    fn default() -> Self {
        Self {
            format: ImageFormatSetting::default(),
            is_srgb: true,
            sampler: ImageSampler::Default,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ImageLoaderError {
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not load texture file: {0}")]
    FileTexture(#[from] FileTextureError),
}

impl AssetLoader for ImageLoader {
    type Asset = Image;
    type Settings = ImageLoaderSettings;
    type Error = ImageLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a ImageLoaderSettings,
        load_context: &'a mut LoadContext,
    ) -> bevy_utils::BoxedFuture<'a, Result<Image, Self::Error>> {
        Box::pin(async move {
            // use the file extension for the image type
            let ext = load_context.path().extension().unwrap().to_str().unwrap();

            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let image_type = match settings.format {
                ImageFormatSetting::FromExtension => ImageType::Extension(ext),
                ImageFormatSetting::Format(format) => ImageType::Format(format),
            };
            Ok(Image::from_buffer(
                &bytes,
                image_type,
                self.supported_compressed_formats,
                settings.is_srgb,
                settings.sampler.clone(),
            )
            .map_err(|err| FileTextureError {
                error: err,
                path: format!("{}", load_context.path().display()),
            })?)
        })
    }

    fn extensions(&self) -> &[&str] {
        IMG_FILE_EXTENSIONS
    }
}


/// An error that occurs when loading a texture from a file.
#[derive(Error, Debug)]
pub struct FileTextureError {
    error: TextureError,
    path: String,
}
impl std::fmt::Display for FileTextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Error reading image file {}: {}, this is an error in `bevy_render`.",
            self.path, self.error
        )
    }
}
