use std::io::Write;

mod huffman;
use huffman::HuffmanNode;

struct Settings {
    decode: bool,
    input: String,
    output: Option<String>,
}

impl Settings {
    pub fn parse() -> Self {
        let mut decode = false;
        let mut input = None;
        let mut output = None;

        let mut args = std::env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-o" => {
                    let output_file = args.next().expect("Cant have -o followed by nothin");
                    if output.is_some() {
                        panic!("Cant have multiple output files");
                    }
                    output = Some(output_file);
                }
                "-d" => decode = true,
                _ => {
                    if input.is_some() {
                        panic!("Cannot have multiple inputs");
                    }

                    input = Some(arg);
                }
            }
        }

        let Some(input) = input else {
            panic!("Give an input file")
        };

        Self {
            decode,
            input,
            output,
        }
    }
}

fn main() {
    let settings = Settings::parse();

    let mut output_stream: Box<dyn Write> = if let Some(output) = settings.output {
        Box::new(std::fs::File::create(output).unwrap())
    } else {
        Box::new(std::io::stdout())
    };

    if settings.decode {
        let input = std::fs::read(settings.input).unwrap();
        let output = decode(input);

        output_stream.write_all(output.as_bytes()).unwrap();
    } else {
        let input = std::fs::read_to_string(settings.input).unwrap();
        let output = encode(input);

        output_stream.write_all(&output).unwrap();
    }

    output_stream.flush().unwrap();
}

fn encode(mut input: String) -> Vec<u8> {
    input.push('\0');
    let input = input.as_bytes();

    let tree = HuffmanNode::build_tree(input);

    tree.serialize(input)
}

fn decode(input: Vec<u8>) -> String {
    let (_tree, output) = HuffmanNode::decode(&input);

    String::from_utf8(output).unwrap()
}
