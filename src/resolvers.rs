mod best_match_resolving_strategy;
mod resolver;
mod simple_resolving_strategy;

pub use best_match_resolving_strategy::BestMatchResolvingStrategy;
pub use resolver::{Resolver, ResolvingStrategy, ScoreBucket};
pub use simple_resolving_strategy::SimpleResolvingStrategy;
