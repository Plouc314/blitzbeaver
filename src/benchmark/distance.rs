// use pyo3::{exceptions::PyValueError, pyfunction, PyErr, PyResult};
// use pyo3_polars::{error::PyPolarsErr, PySeries};

// use crate::{
//     distances::{
//         CachedDistanceCalculatorWord, Distance, LvDistance, LvMultiDistance, LvOptiDistance,
//         TraceCachedDistanceCalculator,
//     },
//     word::Word,
// };

// fn get_distance_function(distance_function: &str) -> PyResult<Box<dyn Distance<Word>>> {
//     match distance_function {
//         "lv" => Ok(Box::new(LvDistance::new())),
//         "lv_opti" => Ok(Box::new(LvOptiDistance::new())),
//         // "lv_multi" => Ok(Box::new(LvMultiDistance::new())),
//         _ => {
//             return Err(PyErr::new::<PyValueError, _>("Invalid distance function"));
//         }
//     }
// }

// fn pyserie_to_vec<'a>(serie: &'a PySeries) -> PyResult<Vec<Option<Word<'a>>>> {
//     Ok(serie
//         .0
//         .str()
//         .map_err(PyPolarsErr::from)?
//         .iter()
//         .map(|v| v.map(|v| Word::new(v)))
//         .collect())
// }

// #[pyfunction]
// pub fn benchmark_distance_functions(
//     values: PySeries,
//     value: String,
//     num_runs: usize,
//     distance_function: String,
// ) -> PyResult<f32> {
//     let value = Word::new(&value);

//     let values = values.0.str().map_err(PyPolarsErr::from)?;
//     let values = values
//         .iter()
//         .map(|v| Word::new(v.unwrap()))
//         .collect::<Vec<Word>>();

//     let mut distance = get_distance_function(&distance_function)?;

//     let mut tot = 0.0;
//     for _ in 0..num_runs {
//         for v in values.iter() {
//             let r = distance.dist(v, &value);
//             tot += r;
//         }
//     }
//     Ok(tot)
// }

// fn compute_product_distances<'a>(
//     fdc: &mut CachedDistanceCalculatorWord<'a>,
//     values1: &'a Vec<Option<Word<'a>>>,
//     values2: &'a Vec<Option<Word<'a>>>,
// ) -> PyResult<f32> {
//     fdc.precompute(&values1, &values2);

//     let mut tot = 0.0;
//     for v1 in values1 {
//         for v2 in values2 {
//             match (v1, v2) {
//                 (Some(v1), Some(v2)) => {
//                     let dist = fdc.get_dist(v1, v2);
//                     tot += dist;
//                 }
//                 _ => {}
//             }
//         }
//     }

//     Ok(tot)
// }

// #[pyfunction]
// pub fn benchmark_feature_distance_calculator(
//     values1: PySeries,
//     values2: PySeries,
//     num_runs: usize,
//     cache_dist_threshold: u32,
//     distance_function: String,
// ) -> PyResult<(u128, u64, u64, usize, f32)> {
//     let values1 = pyserie_to_vec(&values1)?;
//     let values2 = pyserie_to_vec(&values2)?;

//     let st = std::time::Instant::now();

//     let mut tot_trace = TraceCachedDistanceCalculator::new();
//     let mut tot_dist = 0.0;
//     for _ in 0..num_runs {
//         let distance = get_distance_function(&distance_function)?;
//         let mut fdc = CachedDistanceCalculatorWord::new(distance, cache_dist_threshold);

//         let dist = compute_product_distances(&mut fdc, &values1, &values2)?;
//         let trace = fdc.trace.clone();
//         tot_dist += dist;
//         tot_trace.merge(trace);
//     }

//     let duration = st.elapsed();

//     Ok((
//         duration.as_nanos(),
//         tot_trace.computation_count / num_runs as u64,
//         tot_trace.cache_hit_count / num_runs as u64,
//         tot_trace.cache_size / num_runs,
//         tot_dist,
//     ))
// }

// #[pyfunction]
// pub fn benchmark_feature_distance_calculator_second_pass(
//     values1: PySeries,
//     values2: PySeries,
//     values3: PySeries,
//     num_runs: usize,
//     cache_dist_threshold: u32,
//     distance_function: String,
// ) -> PyResult<(u128, u64, u64, usize, f32)> {
//     let values1 = pyserie_to_vec(&values1)?;
//     let values2 = pyserie_to_vec(&values2)?;
//     let values3 = pyserie_to_vec(&values3)?;

//     let mut fdcs = Vec::new();
//     for _ in 0..num_runs {
//         let distance = get_distance_function(&distance_function)?;
//         let mut fdc = CachedDistanceCalculatorWord::new(distance, cache_dist_threshold);
//         compute_product_distances(&mut fdc, &values1, &values2)?;
//         fdc.trace.reset();
//         fdcs.push(fdc);
//     }

//     let st = std::time::Instant::now();

//     let mut tot_trace = TraceCachedDistanceCalculator::new();
//     let mut tot_dist = 0.0;
//     for i in 0..num_runs {
//         let fdc = fdcs.get_mut(i).unwrap();
//         let dist = compute_product_distances(fdc, &values2, &values3)?;
//         tot_dist += dist;
//         tot_trace.merge(fdc.trace.clone());
//     }

//     let duration = st.elapsed();

//     Ok((
//         duration.as_nanos(),
//         tot_trace.computation_count / num_runs as u64,
//         tot_trace.cache_hit_count / num_runs as u64,
//         tot_trace.cache_size / num_runs,
//         tot_dist,
//     ))
// }

// #[pyfunction]
// pub fn benchmark_feature_distance_calculator_multi_pass(
//     values: Vec<PySeries>,
//     num_runs: usize,
//     cache_dist_threshold: u32,
//     distance_function: String,
// ) -> PyResult<(Vec<(u128, u64, u64, usize)>, f32)> {
//     let mut vs = Vec::new();
//     for v in values.iter() {
//         vs.push(pyserie_to_vec(v)?);
//     }

//     let mut traces = vec![TraceCachedDistanceCalculator::new(); values.len() - 1];
//     let mut durations = vec![0; values.len() - 1];

//     let mut tot_dist = 0.0;
//     for _ in 0..num_runs {
//         let distance = get_distance_function(&distance_function)?;
//         let mut fdc = CachedDistanceCalculatorWord::new(distance, cache_dist_threshold);

//         let mut i = 0;
//         let mut prev_v = &vs[0];
//         for v in vs.iter().skip(1) {
//             let st = std::time::Instant::now();
//             let dist = compute_product_distances(&mut fdc, prev_v, v)?;
//             let duration = st.elapsed();

//             durations[i] = duration.as_nanos();
//             traces[i].merge(fdc.trace.clone());

//             tot_dist += dist;
//             prev_v = v;
//             i += 1;
//         }
//     }

//     Ok((
//         durations
//             .into_iter()
//             .zip(traces.into_iter())
//             .map(|(d, t)| {
//                 (
//                     d,
//                     t.computation_count / num_runs as u64,
//                     t.cache_hit_count / num_runs as u64,
//                     t.cache_size / num_runs,
//                 )
//             })
//             .collect(),
//         tot_dist,
//     ))
// }
