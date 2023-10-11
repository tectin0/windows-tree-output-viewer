use std::{cell::RefCell, rc::Rc};

use eframe::{
    egui::{self, RichText},
    CreationContext,
};

use anyhow::{Context, Result};

use crate::tree::{Tree, TreeInfo, TreeNode};

pub(crate) trait Show {
    fn show(&mut self, ctx: &egui::Context, ui: &mut egui::Ui);
}

struct App {
    tree: Tree,
    tree_info: TreeInfo,
}

impl App {
    pub fn new(tree: Tree, tree_info: TreeInfo) -> Self {
        Self { tree, tree_info }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(RichText::new(format!(
                "Volume Name: {}",
                self.tree_info.volume_name
            )));

            ui.label(RichText::new(format!(
                "Volume Serial Number: {}",
                self.tree_info.volume_serial_number
            )));

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.tree.borrow_mut().show(ctx, ui);
                });
        });
    }
}

pub(crate) fn start_gui(tree: Tree, tree_info: TreeInfo) -> Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 800.0)),
        ..Default::default()
    };

    eframe::run_native(
        "tree-viewer",
        options,
        Box::new(|_cc| Box::new(App::new(tree, tree_info))),
    )
    .map_err(|_| anyhow::anyhow!("Failed to start GUI"))
}
