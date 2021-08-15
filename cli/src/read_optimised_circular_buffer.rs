use bounded_vec_deque::BoundedVecDeque;
use std::sync::{Arc, RwLock};

pub type ReadOptimisedCircularBuffer<T> = Arc<RwLock<BoundedVecDeque<T>>>;
