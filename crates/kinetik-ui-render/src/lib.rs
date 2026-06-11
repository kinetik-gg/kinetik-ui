//! Backend-independent renderer contract for Kinetik UI.
//!
//! This crate owns frame submission types, resource registries, image payloads,
//! and renderer diagnostics that are shared by renderer backends. Concrete
//! backends such as Vello consume this contract and keep backend-specific
//! encoding details in their own crates.

use std::collections::HashMap;

use kinetik_ui_core::{ImageId, Primitive, Size, TextLayoutId, TextureId, ViewportInfo};
use kinetik_ui_text::{ShapedTextLayout, StoredTextLayout};

/// Static image resource known by a renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageResource {
    /// Image handle from core primitives.
    pub id: ImageId,
    /// Image size in physical pixels.
    pub size: Size,
    /// Optional CPU pixel data to draw.
    pub pixels: Option<RenderImage>,
}

/// Dynamic texture resource known by a renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct TextureResource {
    /// Texture handle from core primitives.
    pub id: TextureId,
    /// Texture size in physical pixels.
    pub size: Size,
    /// Optional CPU snapshot for renderers that consume image data.
    pub snapshot: Option<RenderImage>,
}

/// CPU image data accepted by renderer boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderImage {
    /// Pixel width.
    pub width: u32,
    /// Pixel height.
    pub height: u32,
    /// Pixel bytes.
    pub data: Vec<u8>,
    /// Pixel format.
    pub format: RenderImageFormat,
    /// Alpha representation.
    pub alpha: RenderImageAlpha,
}

impl RenderImage {
    /// Creates an RGBA8 image after validating byte length.
    #[must_use]
    pub fn rgba8(width: u32, height: u32, data: Vec<u8>) -> Option<Self> {
        Self::new(
            width,
            height,
            data,
            RenderImageFormat::Rgba8,
            RenderImageAlpha::Alpha,
        )
    }

    /// Creates a BGRA8 image after validating byte length.
    #[must_use]
    pub fn bgra8(width: u32, height: u32, data: Vec<u8>) -> Option<Self> {
        Self::new(
            width,
            height,
            data,
            RenderImageFormat::Bgra8,
            RenderImageAlpha::Alpha,
        )
    }

    /// Creates image data after validating byte length.
    #[must_use]
    pub fn new(
        width: u32,
        height: u32,
        data: Vec<u8>,
        format: RenderImageFormat,
        alpha: RenderImageAlpha,
    ) -> Option<Self> {
        let expected_len = format.byte_len(width, height)?;
        (data.len() == expected_len).then_some(Self {
            width,
            height,
            data,
            format,
            alpha,
        })
    }
}

/// CPU image pixel format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderImageFormat {
    /// 32-bit RGBA with 8-bit channels.
    Rgba8,
    /// 32-bit BGRA with 8-bit channels.
    Bgra8,
}

impl RenderImageFormat {
    fn byte_len(self, width: u32, height: u32) -> Option<usize> {
        match self {
            Self::Rgba8 | Self::Bgra8 => 4_usize
                .checked_mul(usize::try_from(width).ok()?)
                .and_then(|bytes| bytes.checked_mul(usize::try_from(height).ok()?)),
        }
    }
}

/// CPU image alpha representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderImageAlpha {
    /// Straight alpha.
    Alpha,
    /// Premultiplied alpha.
    Premultiplied,
}

/// Shaped text layout resource known by a renderer.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLayoutResource {
    /// Text layout handle from core primitives.
    pub id: TextLayoutId,
    /// Owned shaped text layout.
    pub layout: ShapedTextLayout,
}

/// Resource registry used during frame translation and encoding.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderResources {
    images: HashMap<ImageId, ImageResource>,
    textures: HashMap<TextureId, TextureResource>,
    text_layouts: HashMap<TextLayoutId, ShapedTextLayout>,
}

impl RenderResources {
    /// Creates an empty resource registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an image resource.
    pub fn register_image(&mut self, image: ImageResource) {
        self.images.insert(image.id, image);
    }

    /// Registers a texture resource.
    pub fn register_texture(&mut self, texture: TextureResource) {
        self.textures.insert(texture.id, texture);
    }

    /// Registers a shaped text layout resource.
    pub fn register_text_layout(&mut self, text: TextLayoutResource) {
        self.text_layouts.insert(text.id, text.layout);
    }

    /// Registers a borrowed shaped text layout resource.
    pub fn register_text_layout_ref(&mut self, id: TextLayoutId, layout: &ShapedTextLayout) {
        self.text_layouts.insert(id, layout.clone());
    }

    /// Registers shaped text layouts exported by a text layout store.
    pub fn register_text_layouts<'a>(
        &mut self,
        layouts: impl IntoIterator<Item = StoredTextLayout<'a>>,
    ) {
        for layout in layouts {
            self.register_text_layout_ref(layout.id, layout.layout);
        }
    }

    /// Returns true when an image is registered.
    #[must_use]
    pub fn has_image(&self, image: ImageId) -> bool {
        self.images.contains_key(&image)
    }

    /// Returns true when a texture is registered.
    #[must_use]
    pub fn has_texture(&self, texture: TextureId) -> bool {
        self.textures.contains_key(&texture)
    }

    /// Returns a registered image resource.
    #[must_use]
    pub fn image(&self, image: ImageId) -> Option<&ImageResource> {
        self.images.get(&image)
    }

    /// Returns a registered texture resource.
    #[must_use]
    pub fn texture(&self, texture: TextureId) -> Option<&TextureResource> {
        self.textures.get(&texture)
    }

    /// Returns true when a shaped text layout is registered.
    #[must_use]
    pub fn has_text_layout(&self, layout: TextLayoutId) -> bool {
        self.text_layouts.contains_key(&layout)
    }

    /// Returns a registered shaped text layout.
    #[must_use]
    pub fn text_layout(&self, layout: TextLayoutId) -> Option<&ShapedTextLayout> {
        self.text_layouts.get(&layout)
    }
}

/// Input submitted to a renderer for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderFrameInput<'a> {
    /// Viewport for the frame.
    pub viewport: ViewportInfo,
    /// Primitive sequence to draw in order.
    pub primitives: &'a [Primitive],
    /// Image, texture, and text resources available to this frame.
    pub resources: &'a RenderResources,
}

/// Output produced by renderer frame submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFrameOutput {
    /// Number of primitives submitted.
    pub primitive_count: usize,
    /// Recoverable renderer diagnostics.
    pub diagnostics: Vec<RenderDiagnostic>,
}

/// Backend-neutral renderer contract.
///
/// Fatal submission failures are returned as `Self::Error`; recoverable issues
/// such as missing optional resources should be reported through
/// [`RenderFrameOutput::diagnostics`].
pub trait RendererBackend {
    /// Fatal renderer submission error.
    type Error;

    /// Submits one frame to the renderer backend.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when the backend cannot submit the frame at all.
    /// Recoverable primitive/resource issues should be returned as diagnostics.
    fn render_frame(
        &mut self,
        input: RenderFrameInput<'_>,
    ) -> Result<RenderFrameOutput, Self::Error>;
}

/// Renderer diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderDiagnostic {
    /// Text layout resource was referenced but not registered.
    MissingTextLayout(TextLayoutId),
    /// Image resource was referenced but not registered.
    MissingImage(ImageId),
    /// Image resource was registered but does not include drawable pixels.
    MissingImagePixels(ImageId),
    /// Texture resource was referenced but not registered.
    MissingTexture(TextureId),
    /// Texture resource was registered but does not include a drawable snapshot.
    MissingTextureSnapshot(TextureId),
    /// Primitive kind is intentionally represented but not yet translated.
    UnsupportedPrimitive(&'static str),
    /// Primitive contained non-finite or non-positive geometry and was sanitized or skipped.
    InvalidGeometry(&'static str),
}

/// Result of deterministic primitive translation.
#[derive(Debug, Clone, PartialEq)]
pub struct Translation<T> {
    /// Deterministic backend command stream.
    pub commands: Vec<T>,
    /// Translation diagnostics.
    pub diagnostics: Vec<RenderDiagnostic>,
}

/// Returns the crate name.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-ui-render"
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::{
        ImageResource, RenderDiagnostic, RenderFrameInput, RenderFrameOutput, RenderImage,
        RenderResources, RendererBackend, TextLayoutResource, TextureResource,
    };
    use kinetik_ui_core::{
        ImageId, PhysicalSize, ScaleFactor, Size, TextLayoutId, TextureId, ViewportInfo,
    };
    use kinetik_ui_text::{CosmicTextEngine, TextLayoutKey, TextStyle};

    #[derive(Default)]
    struct RecordingRenderer {
        submitted_frames: usize,
    }

    impl RendererBackend for RecordingRenderer {
        type Error = Infallible;

        fn render_frame(
            &mut self,
            input: RenderFrameInput<'_>,
        ) -> Result<RenderFrameOutput, Self::Error> {
            self.submitted_frames += 1;
            Ok(RenderFrameOutput {
                primitive_count: input.primitives.len(),
                diagnostics: vec![RenderDiagnostic::MissingTexture(TextureId::from_raw(7))],
            })
        }
    }

    fn render_once(
        renderer: &mut impl RendererBackend<Error = Infallible>,
        input: RenderFrameInput<'_>,
    ) -> RenderFrameOutput {
        match renderer.render_frame(input) {
            Ok(output) => output,
            Err(error) => match error {},
        }
    }

    #[test]
    fn render_image_validates_pixel_byte_length() {
        assert!(RenderImage::rgba8(2, 2, vec![0; 16]).is_some());
        assert!(RenderImage::rgba8(2, 2, vec![0; 15]).is_none());
    }

    #[test]
    fn resources_register_images_textures_and_text_layouts() {
        let mut resources = RenderResources::new();
        let image = ImageId::from_raw(1);
        let texture = TextureId::from_raw(2);
        let text = TextLayoutId::from_raw(3);
        let mut engine = CosmicTextEngine::new();
        let layout = engine.shape_text(&TextLayoutKey::new(
            "Label",
            TextStyle::new("sans-serif", 12.0, 16.0),
            200.0,
            false,
        ));

        resources.register_image(ImageResource {
            id: image,
            size: Size::new(1.0, 1.0),
            pixels: None,
        });
        resources.register_texture(TextureResource {
            id: texture,
            size: Size::new(1.0, 1.0),
            snapshot: None,
        });
        resources.register_text_layout(TextLayoutResource { id: text, layout });

        assert!(resources.has_image(image));
        assert!(resources.has_texture(texture));
        assert!(resources.has_text_layout(text));
        assert!(resources.image(image).is_some());
        assert!(resources.texture(texture).is_some());
        assert!(resources.text_layout(text).is_some());
    }

    #[test]
    fn frame_input_and_diagnostics_are_backend_neutral() {
        let resources = RenderResources::new();
        let primitives = [];
        let input = RenderFrameInput {
            viewport: ViewportInfo::new(
                Size::new(100.0, 50.0),
                PhysicalSize::new(200, 100),
                ScaleFactor::new(2.0),
            ),
            primitives: &primitives,
            resources: &resources,
        };

        assert_eq!(input.primitives.len(), 0);
        assert_eq!(
            RenderDiagnostic::MissingImage(ImageId::from_raw(9)),
            RenderDiagnostic::MissingImage(ImageId::from_raw(9))
        );
        assert_eq!(
            RenderDiagnostic::MissingImagePixels(ImageId::from_raw(9)),
            RenderDiagnostic::MissingImagePixels(ImageId::from_raw(9))
        );
        assert_eq!(
            RenderDiagnostic::MissingTextureSnapshot(TextureId::from_raw(8)),
            RenderDiagnostic::MissingTextureSnapshot(TextureId::from_raw(8))
        );
    }

    #[test]
    fn renderer_backend_contract_separates_output_diagnostics_from_fatal_errors() {
        let resources = RenderResources::new();
        let primitives = [];
        let input = RenderFrameInput {
            viewport: ViewportInfo::new(
                Size::new(100.0, 50.0),
                PhysicalSize::new(200, 100),
                ScaleFactor::new(2.0),
            ),
            primitives: &primitives,
            resources: &resources,
        };
        let mut renderer = RecordingRenderer::default();

        let output = render_once(&mut renderer, input);

        assert_eq!(renderer.submitted_frames, 1);
        assert_eq!(output.primitive_count, 0);
        assert_eq!(
            output.diagnostics,
            vec![RenderDiagnostic::MissingTexture(TextureId::from_raw(7))]
        );
    }
}
