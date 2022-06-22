use rendering::{
    init,
    structs::{
        dimensions::{Dimensions},
    },
};

fn main() {
    let dimensions = Dimensions::new(200, 100);
    init(dimensions);
}
