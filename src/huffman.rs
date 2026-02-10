#[derive(Debug, Clone)]
pub enum HuffmanNode {
    Node {
        left: Box<HuffmanNode>,
        right: Box<HuffmanNode>,
    },
    Leaf {
        val: u8,
        count: usize,
    },
}

impl PartialEq for HuffmanNode {
    // we don't care about counts. those are only useful during building.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Leaf { val: v1, .. }, Self::Leaf { val: v2, .. }) => v1 == v2,
            (
                Self::Node {
                    left: l1,
                    right: r1,
                },
                Self::Node {
                    left: l2,
                    right: r2,
                },
            ) => l1 == l2 && r1 == r2,
            _ => false,
        }
    }
}

impl HuffmanNode {
    pub fn get_usage(&self) -> usize {
        match self {
            Self::Leaf { val: _, count } => *count,
            Self::Node { left, right } => left.get_usage() + right.get_usage(),
        }
    }

    pub fn build_tree(items: &[u8]) -> Self {
        if items.is_empty() {
            panic!("Cannot build a tree over an empty string!")
        }

        let mut usages = [0; u8::MAX as usize + 1];

        for item in items {
            usages[*item as usize] += 1;
        }

        let mut trees = Vec::new();
        for (i, use_count) in usages.into_iter().enumerate() {
            if use_count > 0 {
                trees.push(HuffmanNode::Leaf {
                    val: i as u8,
                    count: use_count,
                })
            }
        }

        while trees.len() > 1 {
            trees.sort_by(|a, b| b.get_usage().cmp(&a.get_usage()));

            // last 2 will be smallest
            let a = trees.pop().unwrap();
            let b = trees.pop().unwrap();

            let new = HuffmanNode::Node {
                left: Box::new(a),
                right: Box::new(b),
            };

            trees.push(new);
        }

        let tree = trees.pop().unwrap();

        if let HuffmanNode::Leaf { .. } = tree {
            panic!("somehow, palpatine returned");
        } else {
            tree
        }
    }

    pub fn serialize(&self, s: &[u8]) -> Vec<u8> {
        let tree = self.serialize_tree();
        let encoded = s
            .iter()
            .map(|n| {
                let s = self.encode(*n).unwrap();
                s
            })
            .fold("".to_string(), |s, n| s + &n);

        let total = tree + &encoded;

        total
            .as_bytes()
            .chunks(8)
            .map(|chunk| {
                let n = chunk.iter().fold(0_u8, |acc, &b| (acc << 1) | (b - b'0'));
                let extra = 8 - chunk.len();
                n << extra
            })
            .collect()
    }

    fn encode(&self, n: u8) -> Option<String> {
        let (left, right) = match self {
            Self::Node { left, right } => (left, right),
            Self::Leaf { val, .. } => return if *val == n { Some("".into()) } else { None },
        };

        // lowk we're doing a tree search cuz im lazy
        if let Some(s) = left.encode(n) {
            return Some("0".to_string() + &s);
        }
        if let Some(s) = right.encode(n) {
            return Some("1".to_string() + &s);
        }

        return None;
    }

    fn in_order_traversal(&self, pre: String) -> Vec<(u8, String)> {
        match self {
            Self::Leaf { val, .. } => vec![(*val, pre)],
            Self::Node { left, right } => {
                let left = left.in_order_traversal(pre.clone() + "0");

                let right = right.in_order_traversal(pre + "1");

                [left, right].concat()
            }
        }
    }

    fn serialize_tree(&self) -> String {
        // collect codes
        let codes = self.in_order_traversal("".into());

        let mut codes = codes
            .into_iter()
            .map(|(val, code)| format!("{:08b}{:08b}", code.len(), val))
            .fold("".to_string(), |acc, b| acc + &b);

        codes.push_str("00000000");

        codes
    }

    pub fn decode(input: &[u8]) -> (Self, Vec<u8>) {
        let input: &[_] = &input
            .iter()
            .flat_map(|n| (0..8).map(move |i| (n >> (7 - i)) & 1 == 1))
            .chain(std::iter::once(false)) // avoid slice issues
            .collect::<Vec<_>>();

        let (tree, mut input) = Self::decode_tree(input);

        let mut items = Vec::new();
        while input.len() > 1 {
            let (item, inp) = tree.decode_item(input);
            input = inp;
            if item == 0 {
                break;
            }
            items.push(item);
        }

        (tree, items)
    }

    fn decode_item<'a>(&self, mut input: &'a [bool]) -> (u8, &'a [bool]) {
        let mut node = self;
        loop {
            let n = input[0];

            match node {
                HuffmanNode::Leaf { val, .. } => {
                    return (*val, input);
                }
                HuffmanNode::Node { left, right } => {
                    if n {
                        node = right;
                    } else {
                        node = left;
                    }
                    input = &input[1..];
                }
            }
        }
    }

    fn decode_tree(mut input: &[bool]) -> (Self, &[bool]) {
        fn get_byte(input: &[bool]) -> u8 {
            let mut byte = 0;
            for (i, &bit) in input.iter().take(8).enumerate() {
                if bit {
                    byte |= 1 << (7 - i);
                }
            }
            byte
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        struct CodePoint {
            len: u8,
            data: u8,
        }

        let mut codes = Vec::new();
        while get_byte(input) != 0 {
            let len = get_byte(input);
            input = &input[8..];
            let char = get_byte(input);
            input = &input[8..];
            codes.push(CodePoint { len, data: char });
        }
        input = &input[8..]; // go past the 0

        fn build_tree(items: &[CodePoint], index: &mut usize, depth: u8) -> HuffmanNode {
            if items[*index].len == depth {
                let codepoint = items[*index];
                *index += 1;
                return HuffmanNode::Leaf {
                    val: codepoint.data,
                    count: 0,
                };
            }

            let left = build_tree(items, index, depth + 1);
            let right = build_tree(items, index, depth + 1);

            HuffmanNode::Node {
                left: Box::new(left),
                right: Box::new(right),
            }
        }

        let tree = build_tree(&codes, &mut 0, 0);

        (tree, input)
    }

    #[allow(unused)]
    pub fn get_depth(&self) -> usize {
        match self {
            HuffmanNode::Leaf { .. } => 1,
            HuffmanNode::Node { left, right } => {
                let l = left.get_depth();
                let r = right.get_depth();

                l.max(r) + 1
            }
        }
    }
}
