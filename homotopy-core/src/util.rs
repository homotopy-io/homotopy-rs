use crate::common::Generator;

pub(crate) fn first_max_generator<I>(iterator: I) -> Option<Generator>
where
    I: IntoIterator<Item = Generator>,
{
    let mut max: Option<Generator> = None;

    for generator in iterator {
        max = match max {
            Some(prev) if prev.dimension >= generator.dimension => Some(prev),
            _ => Some(generator),
        };
    }

    max
}
