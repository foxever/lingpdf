use crate::app::state::{ScrollMode, SelectionMode};
use crate::app::widgets::{toolbar_btn, toolbar_btn_enabled, toolbar_btn_with_color};
use crate::app::TOOLBAR_HEIGHT;
use crate::i18n::Language;
use crate::theme::{Theme, ThemeColors};
use gpui::*;

use super::super::PdfReaderApp;

impl PdfReaderApp {
    pub(super) fn render_toolbar(
        &self,
        has_doc: bool,
        colors: ThemeColors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = self.state.get_theme();
        let (theme_emoji, theme_color) = match theme {
            Theme::Light => ("🌙", colors.moon_color),
            Theme::Dark => ("☀️", colors.sun_color),
        };

        let language = self.state.get_language();
        let lang_flag = match language {
            Language::English => "🇺🇸",
            Language::Chinese => "🇨🇳",
            Language::Spanish => "🇪🇸",
        };

        let sidebar_emoji = if self.show_sidebar { "📑" } else { "📖" };

        let scroll_mode = self.state.get_scroll_mode();
        let scroll_emoji = match scroll_mode {
            ScrollMode::Page => "📄",
            ScrollMode::Smooth => "📜",
        };

        let selection_mode = self.state.get_selection_mode();
        let selection_emoji = match selection_mode {
            SelectionMode::Hand => "👋",
            SelectionMode::TextSelect => "🖱️",
        };

        div()
            .h(px(TOOLBAR_HEIGHT))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .px_2()
            .gap_1()
            .bg(colors.toolbar)
            .border_b_1()
            .border_color(colors.border)
            .child({
                let this = cx.entity().clone();
                toolbar_btn("📂", colors, move |_event, _window, cx| {
                    this.update(cx, |this, cx| {
                        this.open_file_dialog(cx);
                    });
                })
            })
            .child(div().w(px(4.0)))
            .child(toolbar_btn_enabled(
                "◀",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.prev_page(cx);
                }),
            ))
            .child(toolbar_btn_enabled(
                "▶",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.next_page(cx);
                }),
            ))
            .child(div().w(px(4.0)))
            .child(toolbar_btn_enabled(
                "⏮",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.first_page(cx);
                }),
            ))
            .child(toolbar_btn_enabled(
                "⏭",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.last_page(cx);
                }),
            ))
            .child(div().w(px(4.0)))
            .child(toolbar_btn_enabled(
                "−",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.zoom_out(cx);
                }),
            ))
            .child(toolbar_btn_enabled(
                "+",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.zoom_in(cx);
                }),
            ))
            .child(toolbar_btn_enabled(
                "1:1",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.reset_zoom(cx);
                }),
            ))
            .child(toolbar_btn_enabled(
                "↔",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.fit_width(cx);
                }),
            ))
            .child(toolbar_btn_enabled(
                "⊡",
                has_doc,
                colors,
                cx.listener(|this, _event, window, cx| {
                    this.fit_width_centered(window, cx);
                }),
            ))
            .child(toolbar_btn_enabled(
                "□",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.fit_page(cx);
                }),
            ))
            .child(div().w(px(4.0)))
            .child(toolbar_btn_enabled(
                "↻",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.rotate_clockwise(cx);
                }),
            ))
            .child(toolbar_btn_enabled(
                "↺",
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.rotate_counter_clockwise(cx);
                }),
            ))
            .child(div().w(px(4.0)))
            .child(toolbar_btn_enabled(
                sidebar_emoji,
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.show_sidebar = !this.show_sidebar;
                    cx.notify();
                }),
            ))
            .child(div().w(px(4.0)))
            .child({
                let this = cx.entity().clone();
                if has_doc {
                    toolbar_btn("🖨️", colors, move |_event, _window, cx| {
                        this.update(cx, |this, cx| {
                            this.print(cx);
                        });
                    })
                    .into_any_element()
                } else {
                    div()
                        .px_2()
                        .py(px(2.0))
                        .rounded_sm()
                        .child(div().text_size(px(12.0)).child("🖨️".to_string()))
                        .bg(colors.background_secondary)
                        .text_color(colors.text_secondary)
                        .into_any_element()
                }
            })
            .child(div().w(px(4.0)))
            .child(toolbar_btn_enabled(
                scroll_emoji,
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    let current_mode = this.state.get_scroll_mode();
                    let next_mode = match current_mode {
                        ScrollMode::Page => ScrollMode::Smooth,
                        ScrollMode::Smooth => ScrollMode::Page,
                    };
                    this.state.set_scroll_mode(next_mode);
                    cx.notify();
                }),
            ))
            .child(div().w(px(4.0)))
            .child(toolbar_btn_enabled(
                selection_emoji,
                has_doc,
                colors,
                cx.listener(|this, _event, _window, cx| {
                    this.clear_selection(cx);
                    this.state.toggle_selection_mode();
                    cx.notify();
                }),
            ))
            .child(div().flex_1())
            .child(toolbar_btn(
                lang_flag,
                colors,
                cx.listener(|this, _event, window, cx| {
                    let current_lang = this.state.get_language();
                    let next_lang = match current_lang {
                        Language::English => Language::Chinese,
                        Language::Chinese => Language::Spanish,
                        Language::Spanish => Language::English,
                    };
                    this.state.set_language(next_lang);
                    window.dispatch_action(Box::new(crate::app::menu::RefreshMenus), cx);
                    cx.notify();
                }),
            ))
            .child(toolbar_btn_with_color(
                theme_emoji,
                colors,
                theme_color,
                cx.listener(|this, _event, _window, cx| {
                    this.toggle_theme(cx);
                }),
            ))
    }
}
