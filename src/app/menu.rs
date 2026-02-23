use crate::tr;
use gpui::*;

actions!(
    lingpdf,
    [
        OpenFile,
        CloseTab,
        SaveAs,
        Print,
        Quit,
        ZoomIn,
        ZoomOut,
        ResetZoom,
        FitWidth,
        FitWidthCentered,
        FitPage,
        RotateClockwise,
        RotateCounterClockwise,
        FullScreen,
        ToggleSidebar,
        PrevPage,
        NextPage,
        FirstPage,
        LastPage,
        GoToPage,
        AddToFavorites,
        ToggleTheme,
        About,
        RefreshMenus
    ]
);

pub fn create_menus() -> Vec<Menu> {
    vec![
        Menu {
            name: tr!("menu.file").into(),
            items: vec![
                MenuItem::action(tr!("menu.open"), OpenFile),
                MenuItem::separator(),
                MenuItem::action(tr!("menu.close_tab"), CloseTab),
                MenuItem::action(tr!("menu.save_as"), SaveAs),
                MenuItem::separator(),
                MenuItem::action(tr!("menu.print"), Print),
                MenuItem::separator(),
                MenuItem::action(tr!("menu.quit"), Quit),
            ],
        },
        Menu {
            name: tr!("menu.view").into(),
            items: vec![
                MenuItem::action(tr!("menu.zoom_in"), ZoomIn),
                MenuItem::action(tr!("menu.zoom_out"), ZoomOut),
                MenuItem::action(tr!("menu.reset_zoom"), ResetZoom),
                MenuItem::separator(),
                MenuItem::action(tr!("menu.fit_width"), FitWidth),
                MenuItem::action(tr!("menu.fit_width_centered"), FitWidthCentered),
                MenuItem::action(tr!("menu.fit_page"), FitPage),
                MenuItem::separator(),
                MenuItem::action(tr!("menu.rotate_clockwise"), RotateClockwise),
                MenuItem::action(tr!("menu.rotate_counter_clockwise"), RotateCounterClockwise),
                MenuItem::separator(),
                MenuItem::action(tr!("menu.full_screen"), FullScreen),
                MenuItem::action(tr!("menu.toggle_sidebar"), ToggleSidebar),
                MenuItem::action(tr!("menu.toggle_theme"), ToggleTheme),
            ],
        },
        Menu {
            name: tr!("menu.go").into(),
            items: vec![
                MenuItem::action(tr!("menu.prev_page"), PrevPage),
                MenuItem::action(tr!("menu.next_page"), NextPage),
                MenuItem::separator(),
                MenuItem::action(tr!("menu.first_page"), FirstPage),
                MenuItem::action(tr!("menu.last_page"), LastPage),
                MenuItem::separator(),
                MenuItem::action(tr!("menu.go_to_page"), GoToPage),
            ],
        },
        Menu {
            name: tr!("menu.favorites").into(),
            items: vec![MenuItem::action(
                tr!("menu.add_to_favorites"),
                AddToFavorites,
            )],
        },
        Menu {
            name: tr!("menu.help").into(),
            items: vec![MenuItem::action(tr!("menu.about"), About)],
        },
    ]
}
