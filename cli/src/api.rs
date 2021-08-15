pub use filters::get_n_indicators;
mod filters {
    use super::handlers;
    use crate::actors::messages::PerformanceIndicators;
    use crate::read_optimised_circular_buffer::ReadOptimisedCircularBuffer;
    use std::convert::Infallible;
    use warp::Filter;

    pub fn get_n_indicators(
        buf: ReadOptimisedCircularBuffer<PerformanceIndicators>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("tail")
            .and(warp::path::param())
            .and(with_buf(buf))
            .map(handlers::get_n_indicators)
    }

    fn with_buf(
        buf: ReadOptimisedCircularBuffer<PerformanceIndicators>,
    ) -> impl Filter<
        Extract = (ReadOptimisedCircularBuffer<PerformanceIndicators>,),
        Error = Infallible,
    > + Clone {
        warp::any().map(move || buf.clone())
    }
}

mod handlers {
    use super::models::Indicators;
    use crate::actors::messages::PerformanceIndicators;

    use crate::read_optimised_circular_buffer::ReadOptimisedCircularBuffer;
    pub fn get_n_indicators(
        n: usize,
        buf: ReadOptimisedCircularBuffer<PerformanceIndicators>,
    ) -> impl warp::Reply {
        let reader = buf.read().unwrap();
        warp::reply::json(&Indicators::new(
            reader.iter().take(n).map(|x| x.clone()).collect(),
        ))
    }
}

mod models {
    use crate::actors::messages::PerformanceIndicators;
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Indicators(Vec<PerformanceIndicators>);

    impl Indicators {
        pub fn new(performance_indicators: Vec<PerformanceIndicators>) -> Self {
            Indicators(performance_indicators)
        }
    }
}
