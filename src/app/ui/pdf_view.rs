use crate::app::state::{ScrollMode, SelectionMode};
use crate::app::tabs::SelectionRegion;
use crate::app::{STATUS_BAR_HEIGHT, TOOLBAR_HEIGHT};
use crate::theme::ThemeColors;
use crate::tr;
use gpui::*;
use std::sync::Arc;

use super::super::PdfReaderApp;

/// Convert window coordinates to image-relative coordinates
fn window_to_image_coords(
    window_x: f32,
    window_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    image_width: u32,
    image_height: u32,
    show_sidebar: bool,
) -> (f32, f32) {
    let sidebar_width = if show_sidebar {
        crate::app::SIDEBAR_WIDTH
    } else {
        0.0
    };
    let content_height = viewport_height - TOOLBAR_HEIGHT - STATUS_BAR_HEIGHT;

    // The container is divided into 3 equal parts, image is in the middle part
    let container_width = (viewport_width - sidebar_width) / 3.0;
    let container_start_x = sidebar_width;
    let offset_x = container_start_x + container_width;
    let offset_y = TOOLBAR_HEIGHT;

    // Calculate image offset within the container (centered)
    let img_offset_x = (container_width - image_width as f32) / 2.0;
    let img_offset_y = (content_height - image_height as f32) / 2.0;

    // Convert to image-relative coordinates
    let rel_x = window_x - offset_x - img_offset_x;
    let rel_y = window_y - offset_y - img_offset_y;

    (rel_x, rel_y)
}

impl PdfReaderApp {
    pub(super) fn render_pdf_view(
        &self,
        active_tab_id: Option<usize>,
        colors: ThemeColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let scroll_mode = self.state.get_scroll_mode();

        if active_tab_id.is_none() {
            return div()
                .flex_1()
                .h_full()
                .bg(colors.pdf_view)
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div().flex().flex_col().items_center().gap_3().child(
                        div()
                            .text_size(px(14.0))
                            .text_color(colors.text_secondary)
                            .child(tr!("welcome_message")),
                    ),
                )
                .into_any_element();
        }

        if let Some(tab_id) = active_tab_id {
            if let Some(tab) = self.state.tabs.get_tab(tab_id) {
                if let Some(image) = &tab.page_image {
                    let (width, height) = tab.page_dimensions.unwrap_or((800, 600));
                    let render_image = image.clone();
                    let selection_regions = tab.selection_regions.clone();

                    match scroll_mode {
                        ScrollMode::Page => {
                            return self
                                .render_page_view(
                                    tab_id,
                                    render_image,
                                    width,
                                    height,
                                    selection_regions,
                                    colors,
                                    cx,
                                )
                                .into_any_element();
                        }
                        ScrollMode::Smooth => {
                            return self
                                .render_smooth_view(
                                    tab_id,
                                    render_image,
                                    width,
                                    height,
                                    selection_regions,
                                    colors,
                                    cx,
                                )
                                .into_any_element();
                        }
                    }
                }
            }
        }

        div()
            .flex_1()
            .h_full()
            .bg(colors.pdf_view)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(colors.text_secondary)
                    .child(tr!("pdf.loading")),
            )
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_page_view(
        &self,
        _tab_id: usize,
        render_image: Arc<RenderImage>,
        width: u32,
        height: u32,
        selection_regions: Vec<SelectionRegion>,
        colors: ThemeColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selection_color = gpui::rgba(0x3399FF80);
        let selection_mode = self.state.get_selection_mode();
        let is_text_select_mode = selection_mode == SelectionMode::TextSelect;

        let image_width = width;
        let image_height = height;
        let show_sidebar = self.show_sidebar;

        let mut image_container = div()
            .relative()
            .id("pdf-page-container")
            .w(px(width as f32))
            .h(px(height as f32))
            .cursor(if is_text_select_mode {
                CursorStyle::IBeam
            } else {
                CursorStyle::PointingHand
            })
            .child(
                img(render_image.clone())
                    .block()
                    .w(px(width as f32))
                    .h(px(height as f32)),
            );

        // Render all selection regions
        for region in &selection_regions {
            image_container = image_container.child(
                div()
                    .absolute()
                    .left(px(region.x))
                    .top(px(region.y))
                    .w(px(region.width))
                    .h(px(region.height))
                    .bg(selection_color)
                    .border_1()
                    .border_color(gpui::rgb(0x3399FF)),
            );
        }

        if is_text_select_mode {
            image_container = image_container.child(
                div()
                    .absolute()
                    .inset_0()
                    .cursor(CursorStyle::IBeam)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                            this.is_selecting = true;
                            let x: f32 = event.position.x.into();
                            let y: f32 = event.position.y.into();

                            let viewport = window.viewport_size();
                            let viewport_width: f32 = viewport.width.into();
                            let viewport_height: f32 = viewport.height.into();

                            let (rel_x, rel_y) = window_to_image_coords(
                                x,
                                y,
                                viewport_width,
                                viewport_height,
                                image_width,
                                image_height,
                                show_sidebar,
                            );

                            this.selection_start = Some((rel_x, rel_y));
                            this.selection_end = Some((rel_x, rel_y));
                            this.clear_selection(cx);
                        }),
                    )
                    .on_mouse_move(
                        cx.listener(move |this, event: &MouseMoveEvent, window, cx| {
                            if this.is_selecting {
                                let x: f32 = event.position.x.into();
                                let y: f32 = event.position.y.into();

                                let viewport = window.viewport_size();
                                let viewport_width: f32 = viewport.width.into();
                                let viewport_height: f32 = viewport.height.into();

                                let (rel_x, rel_y) = window_to_image_coords(
                                    x,
                                    y,
                                    viewport_width,
                                    viewport_height,
                                    image_width,
                                    image_height,
                                    show_sidebar,
                                );

                                this.selection_end = Some((rel_x, rel_y));
                                if let (Some(start), Some(end)) =
                                    (this.selection_start, this.selection_end)
                                {
                                    this.update_text_selection(start.0, start.1, end.0, end.1, cx);
                                }
                            }
                        }),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                            if this.is_selecting {
                                this.is_selecting = false;
                                this.copy_selected_text(cx);
                            }
                        }),
                    ),
            );
        }

        div()
            .flex_1()
            .overflow_hidden()
            .bg(colors.pdf_view)
            .flex()
            .items_center()
            .justify_center()
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                match event.delta {
                    ScrollDelta::Pixels(delta) => {
                        if delta.y > px(10.0) {
                            this.next_page(cx);
                        } else if delta.y < px(-10.0) {
                            this.prev_page(cx);
                        }
                    }
                    ScrollDelta::Lines(delta) => {
                        if delta.y > 0.5 {
                            this.next_page(cx);
                        } else if delta.y < -0.5 {
                            this.prev_page(cx);
                        }
                    }
                }
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(image_container),
            )
    }

    #[allow(clippy::too_many_arguments)]
    fn render_smooth_view(
        &self,
        _tab_id: usize,
        render_image: Arc<RenderImage>,
        width: u32,
        _height: u32,
        selection_regions: Vec<SelectionRegion>,
        colors: ThemeColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selection_color = gpui::rgba(0x3399FF80);
        let selection_mode = self.state.get_selection_mode();
        let is_text_select_mode = selection_mode == SelectionMode::TextSelect;

        let mut image_container = div()
            .relative()
            .id("pdf-page-container")
            .cursor(if is_text_select_mode {
                CursorStyle::IBeam
            } else {
                CursorStyle::PointingHand
            })
            .child(img(render_image.clone()).block().max_w(px(width as f32)));

        // Render all selection regions
        for region in &selection_regions {
            image_container = image_container.child(
                div()
                    .absolute()
                    .left(px(region.x))
                    .top(px(region.y))
                    .w(px(region.width))
                    .h(px(region.height))
                    .bg(selection_color)
                    .border_1()
                    .border_color(gpui::rgb(0x3399FF)),
            );
        }

        if is_text_select_mode {
            image_container = image_container.child(
                div()
                    .absolute()
                    .inset_0()
                    .cursor(CursorStyle::IBeam)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, event: &MouseDownEvent, _window, cx| {
                            this.is_selecting = true;
                            let x: f32 = event.position.x.into();
                            let y: f32 = event.position.y.into();
                            this.selection_start = Some((x, y));
                            this.selection_end = Some((x, y));
                            this.clear_selection(cx);
                        }),
                    )
                    .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                        if this.is_selecting {
                            let x: f32 = event.position.x.into();
                            let y: f32 = event.position.y.into();
                            this.selection_end = Some((x, y));
                            if let (Some(start), Some(end)) =
                                (this.selection_start, this.selection_end)
                            {
                                this.update_text_selection(start.0, start.1, end.0, end.1, cx);
                            }
                        }
                    }))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                            if this.is_selecting {
                                this.is_selecting = false;
                                this.copy_selected_text(cx);
                            }
                        }),
                    ),
            );
        }

        div().flex_1().overflow_hidden().bg(colors.pdf_view).child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .p_4()
                .gap_4()
                .child(image_container),
        )
    }
}
