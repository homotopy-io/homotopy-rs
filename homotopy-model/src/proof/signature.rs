use std::{collections::VecDeque, str::FromStr};

use homotopy_common::tree::{Node, Tree};
use homotopy_core::{
    diagram::NewDiagramError, signature::Signature as S, Diagram, Diagram0, DiagramN, Generator,
    Orientation,
};
use homotopy_graphics::style::{Color, SignatureStyleData, VertexShape};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::proof::generators::GeneratorInfo;

pub const COLORS: &[&str] = &[
    "#2980b9", // belize blue
    "#c0392b", // pomegranate
    "#f39c12", // orange
    "#8e44ad", // wisteria
    "#27ae60", // nephritis
    "#f1c40f", // sunflower
    "#f6f5f4", // white(ish)
    "#000000", // black
];

pub const VERTEX_SHAPES: &[VertexShape] = &[VertexShape::Circle, VertexShape::Square];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SignatureItem {
    Folder(FolderInfo),
    Item(GeneratorInfo),
}

impl Default for SignatureItem {
    fn default() -> Self {
        Self::Folder(FolderInfo {
            id: 0,
            name: Default::default(),
            open: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureItemEdit {
    Rename(String),
    Recolor(Color),
    Reshape(VertexShape),
    MakeOriented(bool),
    MakeInvertible(bool),
    ShowSourceTarget(bool),
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

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("generator cannot be marked as directed because its inverse is being used")]
    CannotBeMadeDirected,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Signature(Tree<SignatureItem>);

impl Signature {
    pub fn iter(&self) -> impl Iterator<Item = &GeneratorInfo> {
        self.0.iter().filter_map(|(_, data)| match data.inner() {
            SignatureItem::Item(info) => Some(info),
            &SignatureItem::Folder(_) => None,
        })
    }

    pub fn folder_iter(&self) -> impl Iterator<Item = &FolderInfo> {
        self.0.iter().filter_map(|(_, data)| match data.inner() {
            SignatureItem::Folder(info) => Some(info),
            SignatureItem::Item(_) => None,
        })
    }

    pub fn has_generators(&self) -> bool {
        self.iter().next().is_some()
    }

    pub(crate) fn next_generator_id(&self) -> usize {
        self.generators().map(|g| g.id).max().map_or(0, |id| id + 1)
    }

    fn next_folder_id(&self) -> usize {
        self.folder_iter()
            .map(|info| info.id)
            .max()
            .map_or(0, |id| id + 1)
    }

    #[must_use]
    pub fn filter_map<F>(&self, f: F) -> Signature
    where
        F: Fn(&GeneratorInfo) -> Option<GeneratorInfo>,
    {
        Self(self.as_tree().filter_map(|data| {
            if let SignatureItem::Item(info) = data {
                Some(SignatureItem::Item(f(info)?))
            } else {
                Some(data.clone())
            }
        }))
    }

    pub(crate) fn insert(
        &mut self,
        generator: Generator,
        diagram: impl Into<Diagram>,
        name: &str,
        invertible: bool,
    ) {
        let diagram: Diagram = diagram.into();
        let info = GeneratorInfo {
            generator,
            name: format!("{name} {}", generator.id),
            oriented: false,
            invertible,
            single_preview: true,
            color: Color::from_str(COLORS[generator.id % COLORS.len()]).unwrap(),
            shape: Default::default(),
            diagram,
        };

        self.0.push_onto(self.0.root(), SignatureItem::Item(info));
    }

    pub fn insert_item(&mut self, item: SignatureItem) {
        self.0.push_onto(self.0.root(), item);
    }

    fn find_node(&self, generator: Generator) -> Option<Node> {
        self.0.iter().find_map(|(node, item)| match item.inner() {
            SignatureItem::Item(info) if info.generator == generator => Some(node),
            _ => None,
        })
    }

    pub(crate) fn find_generator(&self, node: Node) -> Option<Generator> {
        self.0
            .with(node, |n| match n.inner() {
                SignatureItem::Item(info) => Some(info.generator),
                SignatureItem::Folder(_) => None,
            })
            .flatten()
    }

    fn edit(&mut self, node: Node, edit: SignatureItemEdit) {
        use SignatureItemEdit::{
            MakeInvertible, MakeOriented, Recolor, Rename, Reshape, ShowSourceTarget,
        };
        self.0.with_mut(node, move |n| match (n.inner_mut(), edit) {
            (SignatureItem::Item(info), Rename(name)) => info.name = name,
            (SignatureItem::Item(info), Recolor(color)) => info.color = color,
            (SignatureItem::Item(info), Reshape(shape)) => info.shape = shape,
            (SignatureItem::Item(info), MakeOriented(true)) => info.oriented = true,
            (SignatureItem::Item(info), MakeInvertible(invertible)) => info.invertible = invertible,
            (SignatureItem::Item(info), ShowSourceTarget(show)) => info.single_preview = !show,
            (SignatureItem::Folder(info), Rename(name)) => info.name = name,
            (_, _) => {}
        });
    }

    pub fn has_descendents_in(&self, node: Node, diagram: &Diagram) -> bool {
        self.0.descendents_of(node).any(|node| {
            self.0
                .with(node, |n| {
                    if let SignatureItem::Item(info) = n.inner() {
                        diagram.generators().contains_key(&info.generator)
                    } else {
                        false
                    }
                })
                .unwrap_or_default()
        })
    }

    pub fn create_generator_zero(&mut self, name: &str) -> Diagram0 {
        let id = self.next_generator_id();
        let generator = Generator::new(id, 0);
        let diagram = Diagram0::from(generator);
        self.insert(generator, diagram, name, false);
        diagram
    }

    pub fn create_generator(
        &mut self,
        source: Diagram,
        target: Diagram,
        name: &str,
        invertible: bool,
    ) -> Result<DiagramN, NewDiagramError> {
        let id = self.next_generator_id();
        let generator = Generator::new(id, source.dimension() + 1);
        let diagram = DiagramN::from_generator(generator, source, target)?;
        self.insert(generator, diagram.clone(), name, invertible);
        Ok(diagram)
    }

    pub fn remove(&mut self, generator: Generator) {
        if let Some(node) = self.find_node(generator) {
            self.0.remove(node);
        }
    }

    pub fn update(&mut self, edit: &SignatureEdit) -> Result<(), SignatureError> {
        match edit {
            SignatureEdit::Edit(node, edit) => {
                // Intercept edit in order to update the whole signature.
                if matches!(edit, SignatureItemEdit::MakeOriented(true)) {
                    let generator = self.find_generator(*node).unwrap();
                    self.0 = self.0.clone().map(|item| match item {
                        SignatureItem::Item(info) => SignatureItem::Item(GeneratorInfo {
                            diagram: info.diagram.remove_framing(generator),
                            ..info
                        }),
                        SignatureItem::Folder(_) => item,
                    });
                }
                if matches!(edit, SignatureItemEdit::MakeInvertible(false)) {
                    let generator = self.find_generator(*node).unwrap();
                    for info in self.iter() {
                        if info
                            .diagram
                            .generators()
                            .get(&generator)
                            .is_some_and(|os| os.contains(&Orientation::Negative))
                        {
                            return Err(SignatureError::CannotBeMadeDirected);
                        }
                    }
                }
                self.edit(*node, edit.clone());
            }
            SignatureEdit::NewFolder(node) => {
                self.0.push_onto(
                    *node,
                    SignatureItem::Folder(FolderInfo {
                        id: self.next_folder_id(),
                        name: "New folder".to_owned(),
                        open: true,
                    }),
                );
            }
            SignatureEdit::MoveBefore(from, to) => {
                if *from != self.0.root() && !self.0.descendents_of(*from).any(|node| node == *to) {
                    self.0.reparent_before(*from, *to);
                }
            }
            SignatureEdit::MoveInto(from, to) => {
                if *from != self.0.root() && !self.0.descendents_of(*from).any(|node| node == *to) {
                    self.0.reparent_under(*from, *to);
                }
            }
            SignatureEdit::ToggleFolder(node) => {
                self.0.with_mut(*node, |n| {
                    if let SignatureItem::Folder(info) = n.inner_mut() {
                        info.open = !info.open;
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

        Ok(())
    }

    pub fn as_tree(&self) -> Tree<SignatureItem> {
        self.0.clone()
    }

    pub fn into_tree(self) -> Tree<SignatureItem> {
        self.0
    }
}

impl SignatureStyleData for Signature {
    type Style = GeneratorInfo;

    fn generator_style(&self, g: Generator) -> Option<&Self::Style> {
        self.generator_info(g)
    }
}

impl homotopy_core::signature::Signature for Signature {
    type Info = GeneratorInfo;

    fn generators(&self) -> impl Iterator<Item = Generator> {
        self.iter().map(|info| info.generator)
    }

    fn generator_info(&self, g: Generator) -> Option<&GeneratorInfo> {
        self.iter().find(|info| info.generator == g)
    }

    fn add_zero(&mut self) -> Diagram0 {
        self.create_generator_zero("Cell")
    }

    fn add(
        &mut self,
        source: impl Into<Diagram>,
        target: impl Into<Diagram>,
    ) -> Result<DiagramN, NewDiagramError> {
        self.create_generator(source.into(), target.into(), "Cell", false)
    }
}

impl From<Tree<SignatureItem>> for Signature {
    fn from(tree: Tree<SignatureItem>) -> Self {
        Self(tree)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub abstr: Option<String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            title: None,
            author: None,
            abstr: None,
        }
    }

    pub fn edit(&mut self, edit: MetadataEdit) {
        match edit {
            MetadataEdit::Title(title) => self.title = Some(title),
            MetadataEdit::Author(author) => self.author = Some(author),
            MetadataEdit::Abstract(abstr) => self.abstr = Some(abstr),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataEdit {
    Title(String),
    Author(String),
    Abstract(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataViewMessage {
    Edit(MetadataEdit),
    //SwitchView(Mode),
    Noop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderInfo {
    pub id: usize,
    pub open: bool,
    pub name: String,
}
