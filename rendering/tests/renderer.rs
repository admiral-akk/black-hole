#[cfg(test)]

mod tests {
    use rendering::{
        renderer::renderer::{RenderConfig, RenderType, Renderer},
        structs::dimensions::Dimensions,
    };

    fn assert_eq_color(buf: &[u8], start_index: usize, expected_color: &[u8]) {
        assert_eq!(
            buf[(4 * start_index)..(4 * start_index + 4)],
            *expected_color
        );
    }

    #[test]
    fn tiny_uv() {
        let renderer = Renderer::new();
        let config = RenderConfig {
            dimensions: Dimensions::new(10, 10),
            render_type: RenderType::UV,
        };
        let mut buf = config.dimensions.get_buffer();
        renderer.render(&mut buf, &config);

        let (width, height) = (config.dimensions.width, config.dimensions.height);

        // top left corner
        assert_eq_color(&buf, 0, &[0, 255, 0, 255]);
        // top right corner
        assert_eq_color(&buf, width - 1, &[255, 255, 0, 255]);
        // bottom left corner
        assert_eq_color(&buf, height * (width - 1), &[0, 0, 0, 255]);
        // bottom right corner
        assert_eq_color(&buf, height * width - 1, &[255, 0, 0, 255]);
    }
}
