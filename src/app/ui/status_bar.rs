use crate::app::STATUS_BAR_HEIGHT;
use crate::theme::ThemeColors;
use crate::tr;
use gpui::*;

use super::super::PdfReaderApp;

impl PdfReaderApp {
    pub(super) fn render_status_bar(
        &self,
        active_tab_id: Option<usize>,
        colors: ThemeColors,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (current_page, page_count, zoom_info, file_name) = if let Some(tab_id) = active_tab_id {
            if let Some(tab) = self.state.tabs.get_tab(tab_id) {
                let file_name = tab.file_name();
                (
                    tab.current_page + 1,
                    tab.page_count,
                    format!("{:.0}%", tab.zoom * 100.0),
                    file_name,
                )
            } else {
                (0, 0, String::new(), String::new())
            }
        } else {
            (0, 0, String::new(), String::new())
        };

        let has_doc = page_count > 0;
        let current_page_clone = current_page;
        let page_count_clone = page_count;

        div()
            .h(px(STATUS_BAR_HEIGHT))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .px_2()
            .gap_3()
            .bg(colors.status_bar)
            .border_t_1()
            .border_color(colors.border)
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(colors.text)
                    .child(if has_doc {
                        file_name
                    } else {
                        tr!("status.ready")
                    }),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(colors.text)
                    .child(if has_doc {
                        format!("{} / {}", current_page_clone, page_count_clone)
                    } else {
                        String::new()
                    }),
            )
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(colors.text)
                    .child(if has_doc { zoom_info } else { String::new() }),
            )
    }
}
