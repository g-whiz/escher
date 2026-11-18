//! Integration tests of `escher-group` through its public interface.

use escher_group::{
    CosetLimit, IndexLimit, Letter, Presentation, Relator, SearchBudget, Word, conway_presentation,
};

#[cfg(test)]
mod common;
#[cfg(test)]
mod examples;
#[cfg(test)]
mod limits;
#[cfg(test)]
mod properties;
#[cfg(test)]
mod search;
#[cfg(test)]
mod spherical;
