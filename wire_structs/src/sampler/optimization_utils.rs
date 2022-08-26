const ITERATIONS: usize = 10;

pub fn optimize<T>(
    bounds: &mut [[f32; 2]],
    constructor: &dyn Fn(&[f32]) -> T,
    error: &dyn Fn(T) -> f32,
) -> T {
    let mut params = Vec::new();
    for bound_index in 0..bounds.len() {
        let mut bound = bounds[bound_index];
        for _ in 0..ITERATIONS {
            let (low, delta) = (bound[0], bound[1] - bound[0]);
            let (low, high) = (low + delta / 3., low + 2. * delta / 3.);
            params.push(low);
            let low_err = get_error(bounds, constructor, error, &mut params);
            params.pop();
            params.push(high);
            let high_err = get_error(bounds, constructor, error, &mut params);
            params.pop();
            if high_err > low_err {
                bound[1] = high;
            } else {
                bound[0] = low;
            }
        }
        params.push((bound[0] + bound[1]) / 2.);
    }
    constructor(&params)
}

pub fn get_error<T>(
    bounds: &mut [[f32; 2]],
    constructor: &dyn Fn(&[f32]) -> T,
    error: &dyn Fn(T) -> f32,
    params: &mut Vec<f32>,
) -> f32 {
    if params.len() == bounds.len() {
        let mid_obj = constructor(&params);
        return error(mid_obj);
    } else {
        let mut bound = bounds[params.len()];
        for _ in 0..ITERATIONS {
            let (low, delta) = (bound[0], bound[1] - bound[0]);
            let (low, high) = (low + delta / 3., low + 2. * delta / 3.);

            params.push(low);
            let low_err = get_error(bounds, constructor, error, params);
            params.pop();

            params.push(high);
            let high_err = get_error(bounds, constructor, error, params);
            params.pop();
            if high_err > low_err {
                bound[1] = high;
            } else {
                bound[0] = low;
            }
        }
        params.push((bound[1] + bound[0]) / 2.);
        let mid_err = get_error(bounds, constructor, error, params);
        params.pop();
        return mid_err;
    }
}
