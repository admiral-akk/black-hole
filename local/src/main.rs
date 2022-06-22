use rendering::{
    init,
    structs::{
        config::Config,
        dimensions::{self, Dimensions},
    },
};

fn main() {
    let dimensions = Dimensions::new(200, 100);
    init(dimensions);
}
