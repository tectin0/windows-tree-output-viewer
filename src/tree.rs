use std::cell::{RefCell, RefMut};
use std::fmt::Display;
use std::io::BufRead;
use std::rc::Rc;

use anyhow::{Context, Result};
use eframe::egui::RichText;

use crate::ui::Show;
use crate::FILE_PATH;

#[derive(PartialEq, Default)]
pub(crate) struct UiInfo {
    pub(crate) visible: bool,
}

pub(crate) struct TreeInfo {
    pub(crate) volume_name: String,
    pub(crate) volume_serial_number: String,
    pub(crate) volume_tag: String,
}

#[derive(PartialEq)]
pub(crate) struct TreeNode {
    pub(crate) value: Option<String>,
    pub(crate) depth: usize,
    pub(crate) children: Vec<Rc<RefCell<TreeNode>>>,
    pub(crate) parent: Option<Rc<RefCell<TreeNode>>>,
    pub(crate) ui_info: UiInfo,
}

impl TreeNode {
    pub(crate) fn new(value: Option<String>, depth: usize) -> Self {
        Self {
            value,
            depth,
            children: Vec::new(),
            parent: None,
            ui_info: UiInfo::default(),
        }
    }

    pub(crate) fn add_child(&mut self, child: Rc<RefCell<TreeNode>>) {
        self.children.push(child);
    }

    pub(crate) fn get_child(&self, value: String) -> Option<Rc<RefCell<TreeNode>>> {
        for child in &self.children {
            match &child.borrow().value {
                Some(child_value) => {
                    if child_value == &value {
                        return Some(child.clone());
                    }
                }
                None => {}
            }
        }
        None
    }

    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        self.children.len()
    }
}

impl Display for TreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        if self.value.is_some() {
            result.push_str(
                format!(
                    "+{} {}\n",
                    "--".repeat(self.depth),
                    self.value.as_ref().unwrap()
                )
                .as_str(),
            );
        }

        if self.children.len() > 0 {
            for child in &self.children {
                result.push_str(format!("{}", child.borrow()).as_str());
            }
        }

        write!(f, "{}", result)
    }
}

impl Show for RefMut<'_, TreeNode> {
    fn show(&mut self, ctx: &eframe::egui::Context, ui: &mut eframe::egui::Ui) {
        let node = self;

        let node_value = node.value.clone();
        let node_depth = node.depth;

        match node_value {
            Some(value) => {
                ui.horizontal(|ui| {
                    ui.add_space(node_depth as f32 * 10.0);
                    ui.toggle_value(
                        &mut node.ui_info.visible,
                        RichText::new(format!("{}", value)).size(20.0),
                    );
                });

                if node.children.len() > 0 && node.ui_info.visible {
                    for child in &node.children {
                        child.borrow_mut().show(ctx, ui);
                    }
                }
            }
            None => {}
        }
    }
}

pub(crate) type Tree = Rc<RefCell<TreeNode>>;

pub(crate) fn load_tree() -> Result<(Tree, TreeInfo)> {
    let file = std::fs::File::open(FILE_PATH).unwrap();

    // Example File Structure:
    /*
       Folder PATH listing for volume X
       Volume serial number is Y Y
       X:\
       +---folder1
       |   \---folder1a
       +---folder2
       |   \---folder2a
       +---folder3
       +---folder4
       |   +---folder4a
       |   |   +---folder4aa
    */

    let mut lines = std::io::BufReader::new(file).lines();

    let volume_name = lines.next().unwrap().unwrap();
    let volume_name = volume_name
        .split("Folder PATH listing for volume ")
        .collect::<Vec<&str>>()[1];

    let volume_serial_number = lines.next().unwrap().unwrap();
    let volume_serial_number = volume_serial_number
        .split("Volume serial number is ")
        .collect::<Vec<&str>>()[1];

    let volume_tag = lines.next().unwrap().unwrap();

    let tree_info = TreeInfo {
        volume_name: volume_name.to_string(),
        volume_serial_number: volume_serial_number.to_string(),
        volume_tag: volume_tag.to_string(),
    };

    let root = Rc::new(RefCell::new(TreeNode::new(Some(volume_tag), 0)));

    let mut current_node = root.clone();

    let mut current_depth = 0;

    for (line_number, line) in lines.into_iter().enumerate() {
        let line_number = line_number + 4;

        let mut line = line.unwrap();

        let mut depth = 0;

        let mut continue_loop = true;

        while continue_loop {
            line = line[4..].to_string();
            depth += 1;

            continue_loop = match line.get(0..4) {
                Some(value) => match value {
                    "+---" => true,
                    "|   " => true,
                    "    " => true,
                    "\\---" => true,
                    _ => false,
                },
                None => false,
            };
        }

        let depth_difference: i32 = depth - current_depth;

        match depth_difference {
            1 => {
                let child = Rc::new(RefCell::new(TreeNode::new(
                    Some(line),
                    usize::try_from(depth).unwrap(),
                )));

                child.borrow_mut().parent = Some(current_node.clone());

                current_node.borrow_mut().add_child(child);
            }
            2 => {
                let last_child = current_node
                    .borrow()
                    .children
                    .last()
                    .context(format!(
                        "Failed to get parent of line {} for {}",
                        line_number, line
                    ))
                    .unwrap()
                    .clone();

                current_node = last_child;
                current_depth = depth - 1;

                let child = Rc::new(RefCell::new(TreeNode::new(
                    Some(line),
                    usize::try_from(depth).unwrap(),
                )));

                child.borrow_mut().parent = Some(current_node.clone());

                current_node.borrow_mut().add_child(child);
            }
            0 => {
                let parent = current_node.borrow().parent.clone();
                current_node = parent.unwrap();

                current_depth = depth - 1;

                let child = Rc::new(RefCell::new(TreeNode::new(
                    Some(line),
                    usize::try_from(depth).unwrap(),
                )));

                child.borrow_mut().parent = Some(current_node.clone());

                current_node.borrow_mut().add_child(child);
            }
            _ => {
                loop {
                    let parent = current_node.borrow().parent.clone();
                    current_node = parent
                        .context(format!(
                            "Failed to get parent of line {} for {}",
                            line_number, line
                        ))
                        .unwrap();

                    current_depth -= 1;

                    if current_depth == depth - 1 {
                        break;
                    }
                }

                let child = Rc::new(RefCell::new(TreeNode::new(
                    Some(line),
                    usize::try_from(depth).unwrap(),
                )));

                child.borrow_mut().parent = Some(current_node.clone());

                current_node.borrow_mut().add_child(child);
            }
        }
    }

    Ok((root, tree_info))
}
