pub fn generate_gaussian_weights(sigma: f32, steps: usize) -> Vec<f32> {
    let mut v = vec![0.0; steps];
    for i in 0..steps {
        v[i] = f32::exp(-0.5 * (i * i) as f32 / (sigma * sigma));
    }

    // normalize
    let mut sum = v[0];

    // double all non-zero weights since there's the negative and positive sides.
    for i in 1..steps {
        sum += 2.0 * v[i]
    }
    for i in 0..steps {
        v[i] /= sum;
    }

    v
}
