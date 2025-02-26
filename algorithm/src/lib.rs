use anyhow::{Context, Result};
use bitvec::prelude::*;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BinaryHeap},
    fs::File,
    io::{Read, Write},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct CompressedFile {
    tree: Node,
    bitstream: BitVec,
}

impl CompressedFile {
    fn new(tree: Node, bitstream: BitVec) -> Self {
        Self { tree, bitstream }
    }
    fn decompress(&self) -> Result<Vec<u8>> {
        self.tree.decode_bytes(&self.bitstream)
    }
}

pub fn compress_file(input_path: &str, output_path: &str) -> Result<()> {
    let mut input_file = File::open(input_path)?;
    let mut file_content = Vec::<u8>::new();
    input_file.read_to_end(&mut file_content)?;

    let compressed = compress_bytes(&file_content)?;
    let mut output_file = File::create(output_path)?;
    output_file.write_all(&compressed)?;
    Ok(())
}

pub fn decompress_file(input_path: &str, output_path: &str) -> Result<()> {
    let mut input_file = File::open(input_path)?;
    let mut file_content = Vec::<u8>::new();
    input_file.read_to_end(&mut file_content)?;
    let uncompressed = decompress_bytes(&file_content)?;

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&uncompressed)?;
    Ok(())
}

pub fn compress_bytes(s: &[u8]) -> Result<Vec<u8>> {
    let tree = generate_huffman_tree(s).context("Unable to generate tree")?;
    let table = tree.generate_code_table();
    let comp = table.encode_bytes(s).context("Unable to encode bytes")?;
    Ok(bitcode::serialize(&CompressedFile::new(tree, comp))?)
}

pub fn decompress_bytes(arr: &[u8]) -> Result<Vec<u8>> {
    let comp: CompressedFile = bitcode::deserialize(&arr[..])?;
    comp.decompress()
}

#[derive(Debug, Clone)]
pub struct CodeTable(FxHashMap<u8, BitVec>);

impl CodeTable {
    fn new() -> Self {
        Self(FxHashMap::default())
    }
    pub fn encode_bytes(&self, s: &[u8]) -> Option<BitVec> {
        let identity = || Some(BitVec::with_capacity(s.len()));
        let fold_fn = |acc: Option<BitVec>, c| {
            acc.map(|mut a| {
                a.extend_from_bitslice(self.0.get(c)?);
                Some(a)
            })?
            
        };
        let reduction_fn = |acc: Option<BitVec>, c: Option<BitVec>| {
            acc.map(|mut a| {
                a.extend_from_bitslice(&c?);
                Some(a)
            })?
        };
        s.par_iter()
            .fold(identity, fold_fn)
            .reduce(identity, reduction_fn)
    }
}

#[derive(Debug, Eq, Ord, PartialEq, Clone, Serialize, Deserialize)]
pub struct Node {
    frequency: u64,
    byte: Option<u8>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.frequency.partial_cmp(&self.frequency)
    }
}

impl Node {
    fn new(frequency: u64, b: u8) -> Node {
        Node {
            frequency,
            byte: Some(b),
            left: None,
            right: None,
        }
    }

    fn with_leafs(frequency: u64, left: Node, right: Node) -> Node {
        Node {
            frequency,
            byte: None,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    pub fn generate_code_table(&self) -> CodeTable {
        let mut codes = CodeTable::new();
        fn generete_codes(tree: &Node, table: &mut CodeTable, mut code: BitVec) -> Option<()> {
            match (&tree.left, &tree.right) {
                (None, None) => {
                    table.0.insert(tree.byte?, code);
                    Some(())
                }
                (Some(l), Some(r)) => {
                    let mut c = code.clone();
                    c.push(false);
                    generete_codes(l.as_ref(), table, c);
                    code.push(true);
                    generete_codes(r.as_ref(), table, code);
                    Some(())
                }
                _ => panic!("Invalid node"),
            }
        }
        generete_codes(self, &mut codes, BitVec::new());
        codes
    }
    pub fn decode_bytes(&self, bits: &BitSlice) -> Result<Vec<u8>> {
        let err_msg = "Unable to decode bytes";
        let mut current = self;
        let mut res = Vec::with_capacity(bits.len());
        for bit in bits {
            current = if *bit {
                current.right.as_ref().context(err_msg)?.as_ref()
            } else {
                current.left.as_ref().context(err_msg)?.as_ref()
            };
            if let Some(byte) = current.byte {
                current = self;
                res.push(byte);
            }
        }
        Ok(res)
    }
}

pub fn generate_huffman_tree(input: &[u8]) -> Option<Node> {
    let mut frequencies: BinaryHeap<_> = input
        .iter()
        .fold(BTreeMap::new(), |mut acc, i| {
            *acc.entry(i).or_insert(0) += 1;
            acc
        })
        .into_iter()
        .map(|(&c, i)| Node::new(i, c))
        .collect();

    while frequencies.len() > 1 {
        let left = frequencies.pop()?;
        let right = frequencies.pop()?;
        let father = Node::with_leafs(left.frequency + right.frequency, left, right);
        frequencies.push(father);
    }
    frequencies.pop()
}
