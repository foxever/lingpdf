pub mod loader;
pub mod renderer;

use std::path::Path;
use thiserror::Error;

const DPI_SCALE: f32 = 2.0;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("Failed to open PDF: {0}")]
    OpenError(String),

    #[error("Failed to render page: {0}")]
    RenderError(String),

    #[error("Invalid page number: {0}")]
    InvalidPage(usize),

    #[error("PDF is password protected")]
    #[allow(dead_code)]
    PasswordProtected,
}

pub type Result<T> = std::result::Result<T, PdfError>;

use pdfium_render::prelude::*;

/// Represents a single text character with its position and bounds
#[derive(Debug, Clone)]
pub struct TextChar {
    pub char: char,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[allow(dead_code)]
    pub font_size: f32,
}

/// Represents text content extracted from a page
#[derive(Debug, Clone)]
pub struct PageText {
    #[allow(dead_code)]
    pub text: String,
    pub chars: Vec<TextChar>,
}

pub struct PdfDocument {
    path: std::path::PathBuf,
    page_count: usize,
}

impl PdfDocument {
    fn get_pdfium() -> Result<Pdfium> {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let lib_path = Pdfium::pdfium_platform_library_name_at_path(exe_dir);
                if lib_path.exists() {
                    return Pdfium::bind_to_library(lib_path)
                        .map(Pdfium::new)
                        .map_err(|e| {
                            PdfError::OpenError(format!("Failed to bind to Pdfium library: {}", e))
                        });
                }
            }
        }

        Pdfium::bind_to_system_library()
            .map(Pdfium::new)
            .map_err(|e| PdfError::OpenError(format!("Failed to bind to Pdfium library: {}", e)))
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let path_str = path.to_string_lossy().to_string();

        let pdfium = Self::get_pdfium()?;

        let doc = pdfium
            .load_pdf_from_file(&path_str, None)
            .map_err(|e| PdfError::OpenError(format!("Failed to load PDF: {}", e)))?;

        let page_count = doc.pages().len() as usize;

        Ok(Self { path, page_count })
    }

    pub fn page_count(&self) -> usize {
        self.page_count
    }

    pub fn render_page(&self, page_num: usize, zoom: f32) -> Result<(Vec<u8>, u32, u32)> {
        if page_num >= self.page_count {
            return Err(PdfError::InvalidPage(page_num));
        }

        let path_str = self.path.to_string_lossy().to_string();

        let pdfium = Self::get_pdfium()?;
        let doc = pdfium
            .load_pdf_from_file(&path_str, None)
            .map_err(|e| PdfError::RenderError(format!("Failed to load PDF: {}", e)))?;

        let page = doc.pages().get(page_num as PdfPageIndex).map_err(|e| {
            PdfError::RenderError(format!("Failed to load page {}: {}", page_num, e))
        })?;

        let size = page.page_size();
        let height = size.height().value;

        let render_config = PdfRenderConfig::new()
            .set_target_height((height * zoom * DPI_SCALE) as i32)
            .render_annotations(true)
            .render_form_data(true)
            .set_reverse_byte_order(true);

        let bitmap = page
            .render_with_config(&render_config)
            .map_err(|e| PdfError::RenderError(format!("Failed to render page: {}", e)))?;

        let data = bitmap.as_rgba_bytes().to_vec();

        let width = bitmap.width() as u32;
        let height = bitmap.height() as u32;

        let mut rgba_data = Vec::with_capacity(data.len());
        for chunk in data.chunks_exact(4) {
            let b = chunk[0];
            let g = chunk[1];
            let r = chunk[2];
            let a = chunk[3];
            rgba_data.push(r);
            rgba_data.push(g);
            rgba_data.push(b);
            rgba_data.push(a);
        }

        Ok((rgba_data, width, height))
    }

    pub fn get_page_size(&self, page_num: usize) -> Result<(f32, f32)> {
        if page_num >= self.page_count {
            return Err(PdfError::InvalidPage(page_num));
        }

        let path_str = self.path.to_string_lossy().to_string();

        let pdfium = Self::get_pdfium()?;
        let doc = pdfium
            .load_pdf_from_file(&path_str, None)
            .map_err(|e| PdfError::RenderError(format!("Failed to load PDF: {}", e)))?;

        let page = doc.pages().get(page_num as PdfPageIndex).map_err(|e| {
            PdfError::RenderError(format!("Failed to load page {}: {}", page_num, e))
        })?;

        let size = page.page_size();
        Ok((size.width().value, size.height().value))
    }

    /// Get document outline (bookmarks/table of contents)
    pub fn get_outline(&self) -> Result<Vec<OutlineItem>> {
        let path_str = self.path.to_string_lossy().to_string();

        let pdfium = Self::get_pdfium()?;
        let doc = pdfium
            .load_pdf_from_file(&path_str, None)
            .map_err(|e| PdfError::OpenError(format!("Failed to load PDF: {}", e)))?;

        let bookmarks = doc.bookmarks();

        fn convert_bookmarks<'a>(bookmark: &PdfBookmark<'a>) -> OutlineItem {
            let title = bookmark.title().unwrap_or_else(|| String::from(""));
            let page = bookmark
                .destination()
                .and_then(|dest| dest.page_index().ok())
                .map(|idx| idx as usize)
                .unwrap_or(0);

            let mut children = Vec::new();
            let mut child = bookmark.first_child();
            while let Some(c) = child {
                children.push(convert_bookmarks(&c));
                child = c.next_sibling();
            }

            OutlineItem {
                title,
                page,
                children,
            }
        }

        let mut items = Vec::new();
        let mut bookmark = bookmarks.root();
        while let Some(b) = bookmark {
            items.push(convert_bookmarks(&b));
            bookmark = b.next_sibling();
        }

        Ok(items)
    }

    /// Extract text from a specific page with character positions
    pub fn extract_page_text(&self, page_num: usize) -> Result<PageText> {
        if page_num >= self.page_count {
            return Err(PdfError::InvalidPage(page_num));
        }

        let path_str = self.path.to_string_lossy().to_string();

        let pdfium = Self::get_pdfium()?;
        let doc = pdfium
            .load_pdf_from_file(&path_str, None)
            .map_err(|e| PdfError::RenderError(format!("Failed to load PDF: {}", e)))?;

        let page = doc.pages().get(page_num as PdfPageIndex).map_err(|e| {
            PdfError::RenderError(format!("Failed to load page {}: {}", page_num, e))
        })?;

        let page_text = page.text().map_err(|e| {
            PdfError::RenderError(format!(
                "Failed to extract text from page {}: {}",
                page_num, e
            ))
        })?;

        let text = page_text.all();
        let mut chars = Vec::new();

        // Extract character positions
        for char_info in page_text.chars().iter() {
            if let Some(ch) = char_info.unicode_char() {
                if let Ok(bounds) = char_info.loose_bounds() {
                    chars.push(TextChar {
                        char: ch,
                        x: bounds.left().value,
                        y: bounds.bottom().value, // Store bottom Y for consistent comparisons
                        width: bounds.width().value,
                        height: bounds.height().value,
                        font_size: char_info.scaled_font_size().value,
                    });
                }
            }
        }

        Ok(PageText { text, chars })
    }

    /// Extract text from a specific region of a page
    #[allow(dead_code)]
    pub fn extract_text_in_rect(
        &self,
        page_num: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<String> {
        if page_num >= self.page_count {
            return Err(PdfError::InvalidPage(page_num));
        }

        let path_str = self.path.to_string_lossy().to_string();

        let pdfium = Self::get_pdfium()?;
        let doc = pdfium
            .load_pdf_from_file(&path_str, None)
            .map_err(|e| PdfError::RenderError(format!("Failed to load PDF: {}", e)))?;

        let page = doc.pages().get(page_num as PdfPageIndex).map_err(|e| {
            PdfError::RenderError(format!("Failed to load page {}: {}", page_num, e))
        })?;

        let page_text = page.text().map_err(|e| {
            PdfError::RenderError(format!(
                "Failed to extract text from page {}: {}",
                page_num, e
            ))
        })?;

        let rect = PdfRect::new(
            PdfPoints::new(y),
            PdfPoints::new(x + width),
            PdfPoints::new(y + height),
            PdfPoints::new(x),
        );

        let text = page_text.inside_rect(rect);
        Ok(text)
    }
}

#[derive(Debug, Clone)]
pub struct OutlineItem {
    pub title: String,
    pub page: usize,
    pub children: Vec<OutlineItem>,
}
