use super::Selector;

#[derive(Debug, Clone)]
pub struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Bounds {
    pub fn center(&self) -> (i32, i32) {
        ((self.left + self.right) / 2, (self.top + self.bottom) / 2)
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

#[derive(Debug, Clone)]
pub struct UiNode {
    pub index: usize,

    pub text: Option<String>,

    pub resource_id: Option<String>,

    pub class_name: Option<String>,

    pub package: Option<String>,

    pub content_desc: Option<String>,

    pub clickable: bool,

    pub enabled: bool,

    pub bounds: Bounds,
}

impl UiNode {
    pub fn center(&self) -> (i32, i32) {
        self.bounds.center()
    }
}

#[derive(Debug, Clone)]
pub struct UiTree {
    pub nodes: Vec<UiNode>,
}

impl UiTree {
    pub fn new(nodes: Vec<UiNode>) -> Self {
        Self { nodes }
    }

    pub fn find(&self, selector: &Selector) -> Option<&UiNode> {
        self.nodes.iter().find(|node| selector.matches(node))
    }

    pub fn find_all(&self, selector: &Selector) -> Vec<&UiNode> {
        self.nodes
            .iter()
            .filter(|node| selector.matches(node))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
