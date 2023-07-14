use homotopy_core::{common::BoundaryPath, Boundary, DiagramN, Height, SliceIndex};
use homotopy_model::proof::AttachOption;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum HighlightKind {
    Attach,
    Slice,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct HighlightSvg<const N: usize> {
    pub kind: HighlightKind,
    pub points: Vec<[SliceIndex; N]>,
}

fn coerce<const N: usize>(point: &[SliceIndex]) -> [SliceIndex; N] {
    point.try_into().unwrap()
}

pub fn highlight_attachment<const N: usize>(
    path_len: usize,
    option: &AttachOption,
) -> HighlightSvg<N> {
    use Boundary::{Source, Target};
    use Height::Regular;

    let boundary_path = option.boundary_path;
    let embedding = &option.embedding;
    let diagram = &option.diagram;

    let depth = boundary_path.map_or(0, |bp| bp.depth() + 1);
    let boundary = boundary_path.map_or(Target, BoundaryPath::boundary);

    let needle = diagram.slice(boundary.flip()).unwrap();

    let points = (|| {
        if depth <= path_len {
            // Attachment is in the interior of the diagram.
            let embedding = embedding.skip(path_len - depth);

            if N == 0 {
                return vec![coerce(&[])];
            }

            let y = embedding[0];
            let needle: DiagramN = needle.try_into().unwrap();

            if N == 1 {
                return vec![
                    coerce(&[Regular(y).into()]),
                    coerce(&[Regular(y + needle.size()).into()]),
                ];
            }

            let x = embedding[1];

            needle
                .regular_slices()
                .enumerate()
                .flat_map(|(i, slice)| {
                    [
                        coerce(&[Regular(y + i).into(), Regular(x).into()]),
                        coerce(&[
                            Regular(y + i).into(),
                            Regular(x + slice.size().unwrap()).into(),
                        ]),
                    ]
                })
                .collect()
        } else if depth == path_len + 1 {
            // Attachment is on the 1-dimensional boundary of the visible diagram (i.e. bottom/top).
            if N == 1 {
                return vec![coerce(&[boundary.into()])];
            }

            vec![
                coerce(&[boundary.into(), Regular(embedding[0]).into()]),
                coerce(&[
                    boundary.into(),
                    Regular(embedding[0] + needle.size().unwrap()).into(),
                ]),
            ]
        } else if depth == path_len + 2 {
            // Attachment is on the 0-dimensional boundary of the visible diagram (i.e. left/right).
            vec![
                coerce(&[Source.into(), boundary.into()]),
                coerce(&[Target.into(), boundary.into()]),
            ]
        } else {
            unreachable!("N cannot be more than 2")
        }
    })();

    HighlightSvg {
        points,
        kind: HighlightKind::Attach,
    }
}

pub fn highlight_slice<const N: usize>(slice: SliceIndex) -> HighlightSvg<N> {
    let mut from = [Boundary::Source.into(); N];
    from[0] = slice;
    let mut to = [Boundary::Target.into(); N];
    to[0] = slice;

    HighlightSvg {
        points: vec![from, to],
        kind: HighlightKind::Slice,
    }
}
