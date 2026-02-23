use crate::print::show_print_dialog;
use crate::tr;
use gpui::*;
use image::RgbaImage;
use std::sync::Arc;

pub const FIT_WIDTH_TARGET: f32 = 800.0;
pub const FIT_PAGE_TARGET_WIDTH: f32 = 600.0;
pub const FIT_PAGE_TARGET_HEIGHT: f32 = 800.0;
pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 3.0;
pub const DPI_SCALE: f32 = 2.0;
pub const WINDOW_DEFAULT_WIDTH: f32 = 1200.0;
pub const WINDOW_DEFAULT_HEIGHT: f32 = 800.0;

// Layout constants - must match the actual UI layout
pub const TOOLBAR_HEIGHT: f32 = 32.0;
pub const STATUS_BAR_HEIGHT: f32 = 20.0;
pub const SIDEBAR_WIDTH: f32 = 200.0;

pub mod actions;
pub mod menu;
pub mod shortcuts;
pub mod state;
pub mod tabs;
pub mod text_selection;
pub mod ui;
pub mod widgets;

use state::AppState;

pub struct PdfReaderApp {
    pub state: Arc<AppState>,
    pub show_sidebar: bool,
    focus_handle: FocusHandle,
    // Text selection state
    pub is_selecting: bool,
    pub selection_start: Option<(f32, f32)>,
    pub selection_end: Option<(f32, f32)>,
}

impl PdfReaderApp {
    pub fn new(state: Arc<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        window.activate_window();
        window.set_window_title("LingPDF");

        let focus_handle = cx.focus_handle();
        focus_handle.focus(window);

        Self {
            state,
            show_sidebar: false,
            focus_handle,
            is_selecting: false,
            selection_start: None,
            selection_end: None,
        }
    }

    pub fn fit_width(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            if let Some(tab) = self.state.tabs.get_tab(tab_id) {
                if let Some(ref pdf_doc) = tab.doc {
                    let current_page = tab.current_page;
                    if let Ok((width, _)) = pdf_doc.get_page_size(current_page) {
                        let zoom = FIT_WIDTH_TARGET / width;
                        self.state.update_active_tab(|tab| {
                            tab.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
                        });
                        self.render_current_tab_page(tab_id, cx);
                        cx.notify();
                    }
                }
            }
        }
    }

    pub fn fit_page(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            if let Some(tab) = self.state.tabs.get_tab(tab_id) {
                if let Some(ref pdf_doc) = tab.doc {
                    let current_page = tab.current_page;
                    if let Ok((width, height)) = pdf_doc.get_page_size(current_page) {
                        let zoom_width = FIT_PAGE_TARGET_WIDTH / width;
                        let zoom_height = FIT_PAGE_TARGET_HEIGHT / height;
                        let zoom = zoom_width.min(zoom_height);
                        self.state.update_active_tab(|tab| {
                            tab.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
                        });
                        self.render_current_tab_page(tab_id, cx);
                        cx.notify();
                    }
                }
            }
        }
    }

    /// Fit page width to viewport with centered layout (optimized for landscape/PPT PDFs)
    pub fn fit_width_centered(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            if let Some(tab) = self.state.tabs.get_tab(tab_id) {
                if let Some(ref pdf_doc) = tab.doc {
                    let current_page = tab.current_page;
                    if let Ok((page_width, page_height)) = pdf_doc.get_page_size(current_page) {
                        let viewport = window.viewport_size();
                        let viewport_width: f32 = viewport.width.into();
                        let viewport_height: f32 = viewport.height.into();

                        // Calculate available content area
                        let sidebar_width = if self.show_sidebar {
                            SIDEBAR_WIDTH
                        } else {
                            0.0
                        };
                        let available_width = viewport_width - sidebar_width;
                        let available_height = viewport_height - TOOLBAR_HEIGHT - STATUS_BAR_HEIGHT;

                        // Calculate zoom to fit width with padding
                        let horizontal_padding = 40.0; // 20px padding on each side
                        let target_width = available_width - horizontal_padding;
                        let zoom_width = target_width / page_width;

                        // Calculate zoom to fit height with padding
                        let vertical_padding = 40.0; // 20px padding on top and bottom
                        let target_height = available_height - vertical_padding;
                        let zoom_height = target_height / page_height;

                        // Use the smaller zoom to ensure page fits both dimensions
                        let zoom = zoom_width.min(zoom_height).clamp(MIN_ZOOM, MAX_ZOOM);

                        self.state.update_active_tab(|tab| {
                            tab.zoom = zoom;
                        });
                        self.render_current_tab_page(tab_id, cx);
                        cx.notify();
                    }
                }
            }
        }
    }

    pub fn open_file_in_new_tab(&mut self, path: std::path::PathBuf, cx: &mut Context<Self>) {
        match self.state.open_file_new_tab(path) {
            Ok(tab_id) => {
                self.show_sidebar = true;
                self.render_current_tab_page(tab_id, cx);
                cx.notify();
            }
            Err(e) => {
                log::error!("Failed to open PDF: {}", e);
            }
        }
    }

    pub fn close_tab(&mut self, tab_id: usize, cx: &mut Context<Self>) {
        self.state.close_tab(tab_id);
        cx.notify();
    }

    pub fn switch_tab(&mut self, tab_id: usize, cx: &mut Context<Self>) {
        self.state.set_active_tab(tab_id);
        self.render_current_tab_page(tab_id, cx);
        cx.notify();
    }

    pub fn open_file_dialog(&mut self, cx: &mut Context<Self>) {
        let dialog_title = tr!("menu.open_file_dialog");
        cx.spawn(async move |this: WeakEntity<Self>, cx| {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("PDF Files", &["pdf"])
                .set_title(&dialog_title)
                .pick_file()
                .await;

            if let Some(file) = file {
                let path = file.path().to_path_buf();
                this.update(cx, |this: &mut Self, cx: &mut Context<Self>| {
                    this.open_file_in_new_tab(path, cx);
                })
                .ok();
            }
        })
        .detach();
    }

    pub fn render_current_tab_page(&mut self, tab_id: usize, _cx: &mut Context<Self>) {
        if let Some(tab) = self.state.tabs.get_tab(tab_id) {
            if let Some(ref pdf_doc) = tab.doc {
                let current_page = tab.current_page;
                let zoom = tab.zoom;
                let rotation = tab.rotation;

                // Extract text from page
                // Clear selection when rendering a new page
                self.clear_selection(_cx);

                match pdf_doc.extract_page_text(current_page) {
                    Ok(page_text) => {
                        self.state.tabs.update_tab(tab_id, |tab| {
                            tab.page_text = Some(page_text);
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to extract text from page: {}", e);
                    }
                }

                match pdf_doc.render_page(current_page, zoom) {
                    Ok((data, pixmap_width, pixmap_height)) => {
                        let mut scaled_width = pixmap_width;
                        let mut scaled_height = pixmap_height;

                        let mut rgba_image = RgbaImage::from_raw(scaled_width, scaled_height, data);

                        if let Some(ref mut rgba) = rgba_image {
                            match rotation {
                                90 => {
                                    *rgba = image::imageops::rotate90(rgba);
                                    std::mem::swap(&mut scaled_width, &mut scaled_height);
                                }
                                180 => {
                                    *rgba = image::imageops::rotate180(rgba);
                                }
                                270 => {
                                    *rgba = image::imageops::rotate270(rgba);
                                    std::mem::swap(&mut scaled_width, &mut scaled_height);
                                }
                                _ => {}
                            }

                            let display_width = (scaled_width as f32 / DPI_SCALE) as u32;
                            let display_height = (scaled_height as f32 / DPI_SCALE) as u32;
                            let page_dimensions = Some((display_width, display_height));
                            let frame = image::Frame::new(rgba.clone());
                            let render_image = RenderImage::new([frame]);
                            let page_image = Some(Arc::new(render_image));

                            self.state.tabs.update_tab(tab_id, |tab| {
                                tab.page_dimensions = page_dimensions;
                                tab.page_image = page_image;
                            });
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to render page: {}", e);
                    }
                }
            }
        }
    }

    pub fn next_page(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            let _ = self.state.next_page();
            self.render_current_tab_page(tab_id, cx);
            cx.notify();
        }
    }

    pub fn prev_page(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            let _ = self.state.prev_page();
            self.render_current_tab_page(tab_id, cx);
            cx.notify();
        }
    }

    pub fn zoom_in(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            self.state.zoom_in();
            self.render_current_tab_page(tab_id, cx);
            cx.notify();
        }
    }

    pub fn zoom_out(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            self.state.zoom_out();
            self.render_current_tab_page(tab_id, cx);
            cx.notify();
        }
    }

    pub fn reset_zoom(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            self.state.reset_zoom();
            self.render_current_tab_page(tab_id, cx);
            cx.notify();
        }
    }

    pub fn rotate_clockwise(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            self.state.rotate_clockwise();
            self.render_current_tab_page(tab_id, cx);
            cx.notify();
        }
    }

    pub fn rotate_counter_clockwise(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            self.state.rotate_counter_clockwise();
            self.render_current_tab_page(tab_id, cx);
            cx.notify();
        }
    }

    pub fn first_page(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            self.state.update_active_tab(|tab| {
                tab.current_page = 0;
            });
            self.render_current_tab_page(tab_id, cx);
            cx.notify();
        }
    }

    pub fn last_page(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            self.state.update_active_tab(|tab| {
                tab.current_page = tab.page_count.saturating_sub(1);
            });
            self.render_current_tab_page(tab_id, cx);
            cx.notify();
        }
    }

    pub fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        let current_theme = self.state.get_theme();
        let new_theme = match current_theme {
            crate::theme::Theme::Light => crate::theme::Theme::Dark,
            crate::theme::Theme::Dark => crate::theme::Theme::Light,
        };
        self.state.set_theme(new_theme);
        cx.notify();
    }

    pub fn print(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            if let Some(tab) = self.state.tabs.get_tab(tab_id) {
                if let Some(ref _pdf_doc) = tab.doc {
                    let file_path = tab.path.clone();

                    cx.spawn(async move |_this: WeakEntity<Self>, _cx| {
                        match show_print_dialog(&file_path) {
                            Ok(_) => {
                                log::info!("Print dialog shown successfully");
                            }
                            Err(e) => {
                                log::error!("Failed to show print dialog: {}", e);
                            }
                        }
                    })
                    .detach();
                }
            }
        }
    }

    /// Copy selected text to clipboard
    pub fn copy_selected_text(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            if let Some(tab) = self.state.tabs.get_tab(tab_id) {
                let selected_text = tab.selected_text.clone();
                if !selected_text.is_empty() {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(selected_text));
                }
            }
        }
    }

    /// Update text selection based on mouse coordinates
    pub fn update_text_selection(
        &mut self,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            if let Some(tab) = self.state.tabs.get_tab(tab_id) {
                if let Some(ref page_text) = tab.page_text {
                    if let (Some(doc), Some((page_width, page_height))) =
                        (&tab.doc, tab.page_dimensions)
                    {
                        // Get the actual page size in PDF points
                        if let Ok((pdf_width, pdf_height)) = doc.get_page_size(tab.current_page) {
                            let (selected_text, selection_regions) =
                                text_selection::calculate_text_selection(
                                    page_text,
                                    pdf_width,
                                    pdf_height,
                                    page_width,
                                    page_height,
                                    start_x,
                                    start_y,
                                    end_x,
                                    end_y,
                                );

                            self.state.tabs.update_tab(tab_id, |tab| {
                                tab.selection_regions = selection_regions;
                                tab.selected_text = selected_text;
                            });

                            cx.notify();
                        }
                    }
                }
            }
        }
    }

    /// Clear text selection
    pub fn clear_selection(&mut self, cx: &mut Context<Self>) {
        if let Some(tab_id) = self.state.get_active_tab_id() {
            self.state.tabs.update_tab(tab_id, |tab| {
                tab.selection_start = None;
                tab.selection_end = None;
                tab.selected_text = String::new();
                tab.selection_regions.clear();
            });
            cx.notify();
        }
    }
}

impl Render for PdfReaderApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_ui(window, cx)
    }
}
