use names::Generator;

pub fn generate_name() -> String {
    let mut generator = Generator::default();
    generator.next().unwrap()
}
