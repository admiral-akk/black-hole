use test_utils::plot_with_title;
use wire_structs::sampler::view_bound::ViewBound;

pub fn analyze_view_bounds(view_bound: &ViewBound) {
    println!("Generating {}", "paths");

    let mut line = Vec::new();
    for (i, v) in view_bound.dist_to_view_bound.iter().enumerate() {
        let i_01 = i as f32 / (view_bound.dist_to_view_bound.len() - 1) as f32;
        line.push((i_01, *v));
    }
    let mut lines = Vec::new();
    lines.push(line);
    plot_with_title(
        &format!("{}", "View Bound by d_01"),
        &format!(
            "generate_artifacts/output/view_bound/{}/{}.png",
            "bound", "bound"
        ),
        &lines,
        ((0., 1.), (0., 1.)),
    )
    .unwrap();
}
