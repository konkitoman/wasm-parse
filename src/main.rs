use wasm_parse::Parse;

fn main() {
    let bytes = include_bytes!("../target/wasm32-wasi/debug/wasi-test.wasm");
    let mut bytes = bytes.to_vec();
    let module = wasm_parse::modules::Module::parse(&mut bytes).unwrap();
    for section in module.sections {
        match section {
            wasm_parse::modules::Section::Custom(_) => {}
            wasm_parse::modules::Section::Type(_) => {}
            wasm_parse::modules::Section::Import(_) => {}
            wasm_parse::modules::Section::Function(_) => {}
            wasm_parse::modules::Section::Table(_) => {}
            wasm_parse::modules::Section::Memory(_) => {}
            wasm_parse::modules::Section::Global(_) => {}
            wasm_parse::modules::Section::Export(_) => {}
            wasm_parse::modules::Section::Start(_) => {}
            wasm_parse::modules::Section::Element(_) => {}
            wasm_parse::modules::Section::Code(code) => println!("Code {code:#?}"),
            wasm_parse::modules::Section::Data(_) => {}
            wasm_parse::modules::Section::DataCountSection(_) => {}
            wasm_parse::modules::Section::Unknown(_) => {}
        }
    }
    println!("Remains: {}", bytes.len());
}
