use crate::app::tabs::{Tab, TabManager};
use crate::app::{MAX_ZOOM, MIN_ZOOM};
use crate::i18n::Language;
use crate::pdf::loader::PdfLoader;
use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScrollMode {
    #[default]
    Page,
    Smooth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SelectionMode {
    #[default]
    Hand, // Hand cursor - for navigation
    TextSelect, // IBeam cursor - for text selection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub recent_files: Vec<String>,
    pub default_zoom: f32,
    pub theme: Theme,
    pub language: Language,
    pub scroll_mode: ScrollMode,
    pub selection_mode: SelectionMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            recent_files: Vec::new(),
            default_zoom: 1.0,
            theme: Theme::Dark,
            language: Language::default(),
            scroll_mode: ScrollMode::default(),
            selection_mode: SelectionMode::default(),
        }
    }
}

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub tabs: Arc<TabManager>,
}

impl AppState {
    pub fn new() -> Self {
        let config = Self::load_config();

        Self {
            config: Mutex::new(config),
            tabs: Arc::new(TabManager::new()),
        }
    }

    pub fn open_file_new_tab(&self, path: PathBuf) -> anyhow::Result<usize> {
        let pdf_doc = PdfLoader::open(&path)?;
        let tab_id = self.tabs.create_tab(path.clone());

        let pdf_doc_arc = Arc::new(pdf_doc);
        let page_count = pdf_doc_arc.page_count();
        let outline = pdf_doc_arc.get_outline().ok();

        self.tabs.update_tab(tab_id, |tab| {
            tab.doc = Some(pdf_doc_arc.clone());
            tab.page_count = page_count;
            tab.outline_items = outline;
        });

        let mut config = self.config.lock().unwrap();
        let path_str = path.to_string_lossy().to_string();
        if !config.recent_files.contains(&path_str) {
            config.recent_files.insert(0, path_str);
            if config.recent_files.len() > 10 {
                config.recent_files.pop();
            }
        }

        self.save_config(&config);
        Ok(tab_id)
    }

    pub fn close_tab(&self, tab_id: usize) {
        self.tabs.close_tab(tab_id);
    }

    pub fn set_active_tab(&self, tab_id: usize) {
        self.tabs.set_active_tab(tab_id);
    }

    pub fn get_active_tab_id(&self) -> Option<usize> {
        self.tabs.get_active_tab()
    }

    pub fn update_active_tab<F>(&self, f: F)
    where
        F: FnOnce(&mut Tab),
    {
        if let Some(tab_id) = self.tabs.get_active_tab() {
            self.tabs.update_tab(tab_id, f);
        }
    }

    pub fn get_all_tabs(&self) -> Vec<Tab> {
        self.tabs.get_all_tabs()
    }

    pub fn navigate_to_page(&self, page: usize) -> anyhow::Result<()> {
        self.update_active_tab(|tab| {
            if page < tab.page_count {
                tab.current_page = page;
            }
        });
        Ok(())
    }

    pub fn next_page(&self) -> anyhow::Result<()> {
        self.update_active_tab(|tab| {
            if tab.current_page < tab.page_count - 1 {
                tab.current_page += 1;
            }
        });
        Ok(())
    }

    pub fn prev_page(&self) -> anyhow::Result<()> {
        self.update_active_tab(|tab| {
            if tab.current_page > 0 {
                tab.current_page -= 1;
            }
        });
        Ok(())
    }

    pub fn zoom_in(&self) {
        self.update_active_tab(|tab| {
            tab.zoom = (tab.zoom + 0.1).min(MAX_ZOOM);
        });
    }

    pub fn zoom_out(&self) {
        self.update_active_tab(|tab| {
            tab.zoom = (tab.zoom - 0.1).max(MIN_ZOOM);
        });
    }

    pub fn reset_zoom(&self) {
        self.update_active_tab(|tab| {
            tab.zoom = 1.0;
        });
    }

    pub fn rotate_clockwise(&self) {
        self.update_active_tab(|tab| {
            tab.rotation = (tab.rotation + 90) % 360;
        });
    }

    pub fn rotate_counter_clockwise(&self) {
        self.update_active_tab(|tab| {
            tab.rotation = (tab.rotation - 90 + 360) % 360;
        });
    }

    pub fn set_theme(&self, theme: Theme) {
        let mut config = self.config.lock().unwrap();
        config.theme = theme;
        self.save_config(&config);
    }

    pub fn get_theme(&self) -> Theme {
        self.config.lock().unwrap().theme
    }

    pub fn set_language(&self, language: Language) {
        let mut config = self.config.lock().unwrap();
        config.language = language;
        self.save_config(&config);
        crate::i18n::I18n::set_language(language);
    }

    pub fn get_language(&self) -> Language {
        self.config.lock().unwrap().language
    }

    pub fn set_scroll_mode(&self, scroll_mode: ScrollMode) {
        let mut config = self.config.lock().unwrap();
        config.scroll_mode = scroll_mode;
        self.save_config(&config);
    }

    pub fn get_scroll_mode(&self) -> ScrollMode {
        self.config.lock().unwrap().scroll_mode
    }

    #[allow(dead_code)]
    pub fn set_selection_mode(&self, selection_mode: SelectionMode) {
        let mut config = self.config.lock().unwrap();
        config.selection_mode = selection_mode;
        self.save_config(&config);
    }

    pub fn get_selection_mode(&self) -> SelectionMode {
        self.config.lock().unwrap().selection_mode
    }

    pub fn toggle_selection_mode(&self) -> SelectionMode {
        let mut config = self.config.lock().unwrap();
        config.selection_mode = match config.selection_mode {
            SelectionMode::Hand => SelectionMode::TextSelect,
            SelectionMode::TextSelect => SelectionMode::Hand,
        };
        let mode = config.selection_mode;
        self.save_config(&config);
        mode
    }

    pub fn get_recent_files(&self) -> Vec<String> {
        self.config.lock().unwrap().recent_files.clone()
    }

    pub fn remove_from_recent(&self, path: &str) {
        let mut config = self.config.lock().unwrap();
        config.recent_files.retain(|p| p != path);
        self.save_config(&config);
    }

    fn load_config() -> AppConfig {
        let config_path = crate::utils::path::get_config_path();
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        AppConfig::default()
    }

    fn save_config(&self, config: &AppConfig) {
        let config_path = crate::utils::path::get_config_path();
        if let Ok(content) = serde_json::to_string_pretty(config) {
            let _ = std::fs::write(config_path, content);
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
