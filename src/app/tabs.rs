use crate::pdf::{PageText, PdfDocument};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct SelectionRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone)]
pub struct Tab {
    pub id: usize,
    pub path: PathBuf,
    pub doc: Option<Arc<PdfDocument>>,
    pub page_count: usize,
    pub current_page: usize,
    pub zoom: f32,
    pub rotation: usize,
    pub outline_items: Option<Vec<crate::pdf::OutlineItem>>,
    pub page_image: Option<Arc<gpui::RenderImage>>,
    pub page_dimensions: Option<(u32, u32)>,
    pub page_text: Option<PageText>,
    // Text selection state
    pub selection_start: Option<(f32, f32)>,
    pub selection_end: Option<(f32, f32)>,
    pub selected_text: String,
    // Multiple selection regions for multi-line selection
    pub selection_regions: Vec<SelectionRegion>,
    // Image container offset in window coordinates (for coordinate conversion)
    #[allow(dead_code)]
    pub image_offset: Option<(f32, f32)>,
}

impl Tab {
    pub fn new(id: usize, path: PathBuf) -> Self {
        Self {
            id,
            path,
            doc: None,
            page_count: 0,
            current_page: 0,
            zoom: 1.0,
            rotation: 0,
            outline_items: None,
            page_image: None,
            page_dimensions: None,
            page_text: None,
            selection_start: None,
            selection_end: None,
            selected_text: String::new(),
            selection_regions: Vec::new(),
            image_offset: None,
        }
    }

    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("Untitled"))
    }
}

pub struct TabManager {
    tabs: Mutex<Vec<Tab>>,
    active_tab_id: Mutex<Option<usize>>,
    next_tab_id: Mutex<usize>,
}

impl TabManager {
    pub fn new() -> Self {
        Self {
            tabs: Mutex::new(Vec::new()),
            active_tab_id: Mutex::new(None),
            next_tab_id: Mutex::new(0),
        }
    }

    pub fn create_tab(&self, path: PathBuf) -> usize {
        let mut next_id = self.next_tab_id.lock().unwrap();
        let tab_id = *next_id;
        *next_id += 1;
        drop(next_id);

        let tab = Tab::new(tab_id, path);
        let mut tabs = self.tabs.lock().unwrap();
        tabs.push(tab);
        drop(tabs);

        self.set_active_tab(tab_id);
        tab_id
    }

    pub fn close_tab(&self, tab_id: usize) {
        let mut tabs = self.tabs.lock().unwrap();
        let index = tabs.iter().position(|t| t.id == tab_id);

        if let Some(index) = index {
            tabs.remove(index);

            let mut active_id = self.active_tab_id.lock().unwrap();
            if Some(tab_id) == *active_id {
                if !tabs.is_empty() {
                    let new_index = if index < tabs.len() {
                        index
                    } else {
                        tabs.len() - 1
                    };
                    *active_id = Some(tabs[new_index].id);
                } else {
                    *active_id = None;
                }
            }
        }
    }

    pub fn set_active_tab(&self, tab_id: usize) {
        let tabs = self.tabs.lock().unwrap();
        if tabs.iter().any(|t| t.id == tab_id) {
            let mut active_id = self.active_tab_id.lock().unwrap();
            *active_id = Some(tab_id);
        }
    }

    pub fn get_active_tab(&self) -> Option<usize> {
        *self.active_tab_id.lock().unwrap()
    }

    pub fn get_tab(&self, tab_id: usize) -> Option<Tab> {
        let tabs = self.tabs.lock().unwrap();
        tabs.iter().find(|t| t.id == tab_id).cloned()
    }

    pub fn update_tab<F>(&self, tab_id: usize, f: F)
    where
        F: FnOnce(&mut Tab),
    {
        let mut tabs = self.tabs.lock().unwrap();
        if let Some(tab) = tabs.iter_mut().find(|t| t.id == tab_id) {
            f(tab);
        }
    }

    pub fn get_all_tabs(&self) -> Vec<Tab> {
        self.tabs.lock().unwrap().clone()
    }
}
