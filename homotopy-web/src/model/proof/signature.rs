use std::{collections::VecDeque, fmt, ops::Deref, str::FromStr};

use homotopy_common::tree::{Node, Tree};
use homotopy_core::{common::Generator, diagram::NewDiagramError, Diagram, DiagramN};
use palette::Srgb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub(crate) Srgb<u8>);

pub const COLORS: &[&str] = &[
    "#2980b9", // belize blue
    "#c0392b", // pomegranate
    "#f39c12", // orange
    "#8e44ad", // wisteria
    "#27ae60", // nephritis
    "#f1c40f", // sunflower
    "#ffffff", // white
    "#000000", // black
];

impl Deref for Color {
    type Target = Srgb<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (r, g, b) = self.into_components();
        write!(f, "#{:02x}{:02x}{:02x}", r, g, b)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorInfo {
    pub generator: Generator,
    pub name: String,
    pub color: Color,
    pub diagram: Diagram,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureItem {
    Folder(String, bool),
    Item(GeneratorInfo),
}

impl Default for SignatureItem {
    fn default() -> Self {
        Self::Folder("".to_owned(), true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureItemEdit {
    Rename(String),
    Recolor(Color),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureEdit {
    Edit(Node, SignatureItemEdit),
    MoveBefore(Node, Node),
    MoveInto(Node, Node),
    ToggleFolder(Node),
    NewFolder(Node),
    Remove(Node),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Signature(Tree<SignatureItem>);

impl Signature {
    pub fn iter(&self) -> impl Iterator<Item = &GeneratorInfo> {
        self.0.iter().filter_map(|(_, data)| match data.inner() {
            SignatureItem::Item(info) => Some(info),
            &SignatureItem::Folder(_, _) => None,
        })
    }

    pub fn generator_info(&self, generator: Generator) -> Option<&GeneratorInfo> {
        self.iter().find(|info| info.generator == generator)
    }

    fn next_generator_id(&self) -> usize {
        self.iter()
            .map(|info| info.generator.id)
            .max()
            .map_or(0, |id| id + 1)
    }

    fn insert<D>(&mut self, id: usize, generator: Generator, diagram: D)
    where
        D: Into<Diagram>,
    {
        let info = GeneratorInfo {
            generator,
            name: format!("Cell {}", id),
            color: Color(Srgb::<u8>::from_str(COLORS[id % COLORS.len()]).unwrap()),
            diagram: diagram.into(),
        };

        self.0.push_onto(self.0.root(), SignatureItem::Item(info));
    }

    fn find_node(&self, generator: Generator) -> Option<Node> {
        self.0.iter().find_map(|(node, item)| match item.inner() {
            SignatureItem::Item(info) if info.generator == generator => Some(node),
            _ => None,
        })
    }

    fn edit(&mut self, node: Node, edit: SignatureItemEdit) {
        use SignatureItemEdit::{Recolor, Rename};
        self.0.with_mut(node, move |n| match (n.inner_mut(), edit) {
            (SignatureItem::Item(info), Rename(name)) => info.name = name,
            (SignatureItem::Item(info), Recolor(color)) => info.color = color,
            (SignatureItem::Folder(ref mut old, _), Rename(name)) => *old = name,
            (_, _) => {}
        });
    }

    pub fn has_descendents_in(&self, node: Node, diagram: &Diagram) -> bool {
        self.0.descendents_of(node).any(|node| {
            self.0.with(node, |n| {
                if let SignatureItem::Item(info) = n.inner() {
                    diagram.generators().contains(&info.generator)
                } else {
                    false
                }
            })
        })
    }

    pub fn create_generator_zero(&mut self) {
        let id = self.next_generator_id();
        let generator = Generator::new(id, 0);
        self.insert(id, generator, generator);
    }

    pub fn create_generator(
        &mut self,
        source: Diagram,
        target: Diagram,
    ) -> Result<Diagram, NewDiagramError> {
        let id = self.next_generator_id();
        let generator = Generator::new(id, source.dimension() + 1);
        let diagram = DiagramN::from_generator(generator, source, target)?;
        self.insert(id, generator, diagram.clone());
        Ok(diagram.into())
    }

    pub fn remove(&mut self, generator: Generator) {
        if let Some(node) = self.find_node(generator) {
            self.0.remove(node);
        }
    }

    pub fn update(&mut self, edit: &SignatureEdit) {
        match edit {
            SignatureEdit::Edit(node, edit) => self.edit(*node, edit.clone()),
            SignatureEdit::NewFolder(node) => {
                self.0
                    .push_onto(*node, SignatureItem::Folder("New folder".to_owned(), true));
            }
            SignatureEdit::MoveBefore(from, to) => self.0.reparent_before(*from, *to),
            SignatureEdit::MoveInto(from, to) => self.0.reparent_under(*from, *to),
            SignatureEdit::ToggleFolder(node) => {
                self.0.with_mut(*node, |n| {
                    if let SignatureItem::Folder(_, b) = n.inner_mut() {
                        *b = !*b;
                    }
                });
            }
            SignatureEdit::Remove(node) => {
                // Prepare to remove all of the descendents of the deleted node
                let mut to_remove: VecDeque<_> = self.0.descendents_of(*node).collect();
                // So long as we have something left to delete
                while let Some(removing) = to_remove.pop_front() {
                    let mut implied = self
                        .0
                        .iter()
                        .filter_map(|(node, item)| {
                            // Find all of the generators in the signature
                            if let SignatureItem::Item(info) = item.inner() {
                                if self.has_descendents_in(removing, &info.diagram) {
                                    return Some(node);
                                }
                            }

                            None
                        })
                        .collect();

                    // Queue these for deletion
                    to_remove.append(&mut implied);
                    // Delete the node
                    self.0.remove(removing);
                }
            }
        }
    }

    pub fn as_tree(&self) -> Tree<SignatureItem> {
        self.0.clone()
    }

    pub fn into_tree(self) -> Tree<SignatureItem> {
        self.0
    }
}

impl From<Tree<SignatureItem>> for Signature {
    fn from(tree: Tree<SignatureItem>) -> Self {
        Self(tree)
    }
}
