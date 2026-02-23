use super::PdfReaderApp;
use crate::app::menu::{
    CloseTab, FirstPage, FitPage, FitWidth, FullScreen, LastPage, NextPage, OpenFile, PrevPage,
    Print, Quit, RefreshMenus, ResetZoom, RotateClockwise, RotateCounterClockwise, ToggleSidebar,
    ToggleTheme, ZoomIn, ZoomOut,
};
use gpui::{prelude::*, App, WindowHandle};

/// Helper to register window update actions
fn register_window_action<A, F>(cx: &mut App, window_handle: &WindowHandle<PdfReaderApp>, action: F)
where
    A: gpui::Action + 'static,
    F: Fn(&mut PdfReaderApp, &mut Context<PdfReaderApp>) + Clone + 'static,
{
    let window_handle = *window_handle;
    cx.on_action(move |_: &A, cx: &mut App| {
        window_handle
            .update(cx, |app: &mut PdfReaderApp, _window, cx| {
                action(app, cx);
            })
            .ok();
    });
}

/// Register all application actions
pub fn register_actions(cx: &mut App, window_handle: WindowHandle<PdfReaderApp>) {
    // Quit action
    cx.on_action(|_: &Quit, cx: &mut App| {
        cx.quit();
    });

    // Refresh menus action
    cx.on_action(|_: &RefreshMenus, cx: &mut App| {
        update_menus(cx);
    });

    // Open file action
    cx.on_action({
        move |_: &OpenFile, cx: &mut App| {
            window_handle
                .update(cx, |app: &mut PdfReaderApp, _window, cx| {
                    app.open_file_dialog(cx);
                })
                .ok();
        }
    });

    // Close tab action
    cx.on_action({
        move |_: &CloseTab, cx: &mut App| {
            window_handle
                .update(cx, |app: &mut PdfReaderApp, _window, cx| {
                    if let Some(tab_id) = app.state.get_active_tab_id() {
                        app.close_tab(tab_id, cx);
                    }
                })
                .ok();
        }
    });

    // Navigation actions
    register_window_action::<PrevPage, _>(cx, &window_handle, |app, cx| {
        app.prev_page(cx);
    });
    register_window_action::<NextPage, _>(cx, &window_handle, |app, cx| {
        app.next_page(cx);
    });
    register_window_action::<FirstPage, _>(cx, &window_handle, |app, cx| {
        app.first_page(cx);
    });
    register_window_action::<LastPage, _>(cx, &window_handle, |app, cx| {
        app.last_page(cx);
    });

    // Zoom actions
    register_window_action::<ZoomIn, _>(cx, &window_handle, |app, cx| {
        app.zoom_in(cx);
    });
    register_window_action::<ZoomOut, _>(cx, &window_handle, |app, cx| {
        app.zoom_out(cx);
    });
    register_window_action::<ResetZoom, _>(cx, &window_handle, |app, cx| {
        app.reset_zoom(cx);
    });

    // Fit actions
    register_window_action::<FitWidth, _>(cx, &window_handle, |app, cx| {
        app.fit_width(cx);
    });
    register_window_action::<FitPage, _>(cx, &window_handle, |app, cx| {
        app.fit_page(cx);
    });

    // Rotate actions
    register_window_action::<RotateClockwise, _>(cx, &window_handle, |app, cx| {
        app.rotate_clockwise(cx);
    });
    register_window_action::<RotateCounterClockwise, _>(cx, &window_handle, |app, cx| {
        app.rotate_counter_clockwise(cx);
    });

    // Toggle actions
    register_window_action::<ToggleSidebar, _>(cx, &window_handle, |app, cx| {
        app.show_sidebar = !app.show_sidebar;
        cx.notify();
    });
    register_window_action::<ToggleTheme, _>(cx, &window_handle, |app, cx| {
        app.toggle_theme(cx);
    });

    // Fullscreen action
    cx.on_action({
        move |_: &FullScreen, cx: &mut App| {
            window_handle
                .update(cx, |_app, window, _cx| {
                    window.toggle_fullscreen();
                })
                .ok();
        }
    });

    // Print action
    register_window_action::<Print, _>(cx, &window_handle, |app, cx| {
        app.print(cx);
    });
}

/// Update application menus based on current language
fn update_menus(cx: &mut App) {
    let menus = crate::app::menu::create_menus();

    let mut full_menus = vec![gpui::Menu {
        name: "LingPDF".into(),
        items: vec![
            gpui::MenuItem::os_submenu("Services", gpui::SystemMenuType::Services),
            gpui::MenuItem::separator(),
            gpui::MenuItem::action("Quit", Quit),
        ],
    }];
    full_menus.extend(menus);

    cx.set_menus(full_menus);
}
