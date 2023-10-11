use anyhow::Result;
use tree::load_tree;
use ui::start_gui;

mod tree;
mod ui;

const FILE_PATH: &str = "H:/content.txt";

fn main() -> Result<()> {
    let (tree, tree_info) = load_tree()?;

    // fs::write("output.txt", format!("{}", tree.borrow()))?;

    start_gui(tree, tree_info)?;

    Ok(())
}
