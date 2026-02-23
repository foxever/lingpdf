use crate::app::SIDEBAR_WIDTH;
use crate::pdf::OutlineItem;
use crate::theme::ThemeColors;
use crate::tr;
use gpui::*;

use super::super::PdfReaderApp;

impl PdfReaderApp {
    pub(super) fn render_sidebar(
        &self,
        active_tab_id: Option<usize>,
        colors: ThemeColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let outline = active_tab_id.and_then(|id| {
            self.state
                .tabs
                .get_tab(id)
                .and_then(|t| t.outline_items.clone())
        });
        let has_doc = active_tab_id.is_some();

        div()
            .w(px(SIDEBAR_WIDTH))
            .h_full()
            .flex()
            .flex_col()
            .bg(colors.background_secondary)
            .border_r_1()
            .border_color(colors.border)
            .child(
                div()
                    .h(px(24.0))
                    .w_full()
                    .flex()
                    .items_center()
                    .px_2()
                    .bg(colors.background)
                    .border_b_1()
                    .border_color(colors.border)
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(colors.text)
                            .child(if has_doc {
                                tr!("sidebar.outline")
                            } else {
                                tr!("sidebar.recent_files")
                            }),
                    ),
            )
            .child(div().flex_1().p_1().child(if has_doc {
                match outline {
                    Some(items) if !items.is_empty() => self
                        .render_outline_items(&items, colors, cx, 0)
                        .into_any_element(),
                    _ => self.render_page_list(colors, cx).into_any_element(),
                }
            } else {
                self.render_recent_files(colors, cx).into_any_element()
            }))
    }

    fn render_recent_files(&self, colors: ThemeColors, cx: &mut Context<Self>) -> impl IntoElement {
        let recent_files = self.state.get_recent_files();

        if recent_files.is_empty() {
            return div()
                .text_size(px(10.0))
                .text_color(colors.text_secondary)
                .child(tr!("sidebar.no_recent_files"))
                .into_any_element();
        }

        let mut container = div().flex().flex_col();

        for file_path in recent_files {
            let file_name = std::path::Path::new(&file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&file_path)
                .to_string();
            let path_clone = file_path.clone();

            container = container.child(
                div()
                    .px_2()
                    .py(px(4.0))
                    .cursor_pointer()
                    .hover(|this| this.bg(colors.background_tertiary))
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(colors.text)
                            .child(file_name),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event, _window, cx| {
                            let path = std::path::PathBuf::from(&path_clone);
                            if path.exists() {
                                this.open_file_in_new_tab(path, cx);
                            } else {
                                this.state.remove_from_recent(&path_clone);
                                cx.notify();
                            }
                        }),
                    ),
            );
        }

        container.into_any_element()
    }

    fn render_outline_items(
        &self,
        items: &[OutlineItem],
        colors: ThemeColors,
        cx: &mut Context<Self>,
        level: usize,
    ) -> impl IntoElement {
        let mut container = div().flex().flex_col();

        for item in items {
            let page_num = item.page;
            container = container.child(
                div()
                    .px(px(level as f32 * 12.0 + 8.0))
                    .py(px(4.0))
                    .cursor_pointer()
                    .hover(|this| this.bg(colors.background_tertiary))
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(colors.text)
                            .child(item.title.clone()),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener({
                            let page = page_num;
                            move |this, _event, _window, cx| {
                                if let Some(tab_id) = this.state.get_active_tab_id() {
                                    let _ = this.state.navigate_to_page(page);
                                    this.render_current_tab_page(tab_id, cx);
                                    cx.notify();
                                }
                            }
                        }),
                    ),
            );

            if !item.children.is_empty() {
                container = container.child(self.render_outline_items(
                    &item.children,
                    colors,
                    cx,
                    level + 1,
                ));
            }
        }

        container
    }

    fn render_page_list(&self, colors: ThemeColors, cx: &mut Context<Self>) -> impl IntoElement {
        let page_count = self
            .state
            .get_active_tab_id()
            .and_then(|id| self.state.tabs.get_tab(id))
            .map(|t| t.page_count)
            .unwrap_or(0);

        if page_count == 0 {
            return div()
                .text_size(px(10.0))
                .text_color(colors.text_secondary)
                .child(tr!("pdf.no_outline"))
                .into_any_element();
        }

        let mut container = div().flex().flex_col();

        for page_num in 0..page_count {
            let page_num_clone = page_num;
            container = container.child(
                div()
                    .px_2()
                    .py(px(4.0))
                    .cursor_pointer()
                    .hover(|this| this.bg(colors.background_tertiary))
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(colors.text)
                            .child(format!("{} {}", tr!("page.label"), page_num_clone + 1)),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event, _window, cx| {
                            if let Some(tab_id) = this.state.get_active_tab_id() {
                                let _ = this.state.navigate_to_page(page_num_clone);
                                this.render_current_tab_page(tab_id, cx);
                                cx.notify();
                            }
                        }),
                    ),
            );
        }

        container.into_any_element()
    }
}
