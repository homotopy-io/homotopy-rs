use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use palette::Srgb;

use homotopy_common::tree::{Node, Tree};

use homotopy_core::common::Generator;
use homotopy_core::diagram::NewDiagramError;
use homotopy_core::{Diagram, DiagramN};

use super::ModelError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub(crate) Srgb<u8>);

const COLORS: &[&str] = &[
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
    Folder(String),
    Item(GeneratorInfo),
}

impl Default for SignatureItem {
    fn default() -> Self {
        Self::Folder("".to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratorEdit {
    Rename(String),
    Recolor(Color),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureEdit {
    MoveBefore(Node, Node),
    MoveInto(Node, Node),
    NewFolder,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Signature(Tree<SignatureItem>);

impl Signature {
    pub fn iter(&self) -> impl Iterator<Item = &GeneratorInfo> {
        self.0.iter().flat_map(|(_, data)| match data.inner() {
            SignatureItem::Item(info) => Some(info),
            _ => None,
        })
    }

    pub fn generator_info(&self, generator: Generator) -> Option<&GeneratorInfo> {
        self.iter()
            .filter(|info| info.generator == generator)
            .next()
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
        let diagram = DiagramN::new(generator, source, target)?;
        self.insert(id, generator, diagram.clone());
        Ok(diagram.into())
    }

    pub fn edit_generator(
        &mut self,
        generator: Generator,
        edit: GeneratorEdit,
    ) -> Result<(), ModelError> {
        let node = self
            .find_node(generator)
            .ok_or(ModelError::UnknownGeneratorSelected)?;

        self.0.with_mut(node, move |n| {
            if let SignatureItem::Item(info) = n.inner_mut() {
                match edit {
                    GeneratorEdit::Rename(name) => info.name = name,
                    GeneratorEdit::Recolor(color) => info.color = color,
                }
            }
        });

        Ok(())
    }

    pub fn remove(&mut self, generator: Generator) {
        if let Some(node) = self.find_node(generator) {
            self.0.remove(node)
        }
    }

    pub fn update(&mut self, edit: &SignatureEdit) {
        match edit {
            SignatureEdit::NewFolder => {
                self.0.push_onto(
                    self.0.root(),
                    SignatureItem::Folder("New folder".to_owned()),
                );
            }
            SignatureEdit::MoveBefore(from, to) => self.0.reparent_before(*from, *to),
            SignatureEdit::MoveInto(from, to) => self.0.reparent_under(*from, *to),
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
