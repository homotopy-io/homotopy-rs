use std::{collections::VecDeque, str::FromStr};

use homotopy_common::{
    hash::FastHashMap,
    tree::{Node, Tree},
};
use homotopy_core::{
    diagram::NewDiagramError,
    signature::{Invertibility, Signature as _},
    typecheck::{typecheck, Mode},
    Diagram, Diagram0, DiagramN, Generator,
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
    MakeInvertible(Invertibility),
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
    #[error("generator cannot be marked as {0} because it is being used in an incompatible way")]
    Invertibility(Invertibility),
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

    #[must_use]
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
    pub fn filter_map<F>(&self, f: F) -> Self
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
        invertibility: Invertibility,
    ) {
        let diagram: Diagram = diagram.into();
        let info = GeneratorInfo {
            generator,
            name: format!("{name} {}", generator.id),
            oriented: false,
            invertibility,
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
        use SignatureItem::{Folder, Item};
        use SignatureItemEdit::{
            MakeInvertible, MakeOriented, Recolor, Rename, Reshape, ShowSourceTarget,
        };
        self.0.with_mut(node, move |n| match (n.inner_mut(), edit) {
            (Item(info), Rename(name)) => info.name = name,
            (Item(info), Recolor(color)) => info.color = color,
            (Item(info), Reshape(shape)) => info.shape = shape,
            (Item(info), MakeOriented(true)) => info.oriented = true,
            (Item(info), MakeInvertible(invertibility)) => info.invertibility = invertibility,
            (Item(info), ShowSourceTarget(show)) => info.single_preview = !show,
            (Folder(info), Rename(name)) => info.name = name,
            (_, _) => {}
        });
    }

    #[must_use]
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
        self.insert(generator, diagram, name, Invertibility::Directed);
        diagram
    }

    pub fn create_generator(
        &mut self,
        source: Diagram,
        target: Diagram,
        name: &str,
        invertibility: Invertibility,
    ) -> Result<DiagramN, NewDiagramError> {
        let id = self.next_generator_id();
        let generator = Generator::new(id, source.dimension() + 1);
        let diagram = DiagramN::from_generator(generator, source, target)?;
        self.insert(generator, diagram.clone(), name, invertibility);
        Ok(diagram)
    }

    pub fn remove(&mut self, generator: Generator) {
        if let Some(node) = self.find_node(generator) {
            self.0.remove(node);
        }
    }

    /// Disjoint union this signature with another one, adding its contents to a new folder.
    pub(crate) fn union_signature(&mut self, other: Self, metadata: Metadata) {
        // We cannot naively copy the other signature, because its
        // Node-s, folder IDs, and Generator-s may conflict with our
        // existing IDs. Thus we need to generate fresh IDs ("here")
        // and then replace those within the other signature ("there")
        // as we add its items to this one.

        // First, create fresh generator IDs for each generator in the source.
        let mut generator_there_to_here = FastHashMap::<Generator, Generator>::default();
        for (generator_there, id_here) in other.generators().zip(self.next_generator_id()..) {
            generator_there_to_here.insert(
                generator_there,
                Generator::new(id_here, generator_there.dimension),
            );
        }

        // Then, replace all generators in parallel, and create fresh folder IDs.
        let mut folder_ids = (self.next_folder_id()..).into_iter();
        let mut tree_internally_here = other.0.map(|node| match node {
            SignatureItem::Folder(mut info) => {
                info.id = folder_ids.next().unwrap_or_default();
                SignatureItem::Folder(info)
            }
            SignatureItem::Item(mut info) => {
                if let Some(&generator_here) = generator_there_to_here.get(&info.generator) {
                    info.generator = generator_here;
                } else {
                    tracing::error!("Generator {:?} not represented here", info.generator);
                }

                // It is important to substitute in parallel so that
                // there is never a mix of old and new generators in
                // the diagram.
                info.diagram = info.diagram.replace_map(&|gen_there| {
                    if let Some(&gen_here) = generator_there_to_here.get(&gen_there) {
                        gen_here
                    } else {
                        // We are replacing all generators in the
                        // imported signature with fresh generators,
                        // so all of them should be represented.
                        tracing::error!(
                            "Generator {:?} in diagram not represented here",
                            gen_there
                        );
                        gen_there
                    }
                });

                SignatureItem::Item(info)
            }
        });

        // Then merge the tree into the new folder.
        let mut node_there_to_here = FastHashMap::<Node, Node>::default();
        if let Some(info) = tree_internally_here.get_mut(tree_internally_here.root()) {
            if let SignatureItem::Folder(folder) = info.inner_mut() {
                folder.name = metadata
                    .title
                    .unwrap_or_else(|| "Imported folder".to_owned());
            }
        }

        for (node_there, info) in tree_internally_here.iter() {
            let parent_here = info
                .parent()
                .and_then(|parent| node_there_to_here.get(&parent).copied())
                .unwrap_or_else(|| self.0.root());
            if let Some(node_here) = self.0.push_onto(parent_here, info.inner().clone()) {
                node_there_to_here.insert(node_there, node_here);
            }
        }
    }

    pub fn update(&mut self, edit: &SignatureEdit) -> Result<(), SignatureError> {
        match edit {
            SignatureEdit::Edit(node, edit) => {
                self.edit(*node, edit.clone());

                // Intercept edit in order to update the whole signature.
                if matches!(edit, SignatureItemEdit::MakeOriented(true)) {
                    if let Some(generator) = self.find_generator(*node) {
                        self.0 = self.0.clone().map(|item| match item {
                            SignatureItem::Item(info) => SignatureItem::Item(GeneratorInfo {
                                diagram: info.diagram.remove_framing(generator),
                                ..info
                            }),
                            SignatureItem::Folder(_) => item,
                        });
                    }
                }
                if let SignatureItemEdit::MakeInvertible(invertibility) = edit {
                    if let Some(generator) = self.find_generator(*node) {
                        let info = self.generator_info(generator).unwrap();
                        if *invertibility < info.invertibility {
                            let mode = Mode {
                                directed: true,
                                dualisable: true,
                                simplices: false,
                                generator: Some(generator),
                            };
                            if self
                                .iter()
                                .any(|info| typecheck(&info.diagram, self, mode, true).is_err())
                            {
                                return Err(SignatureError::Invertibility(*invertibility));
                            }
                        }
                    }
                }
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

    #[must_use]
    pub fn as_tree(&self) -> Tree<SignatureItem> {
        self.0.clone()
    }

    #[must_use]
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
    #[must_use]
    pub const fn new() -> Self {
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
