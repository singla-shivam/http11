use std::collections::HashMap;

#[derive(Debug, Default)]
struct Node {
    chars: HashMap<char, Node>,
    is_end_of_word:bool,
    val: Option<u32>,
}

#[derive(Debug)]
struct Trie {
    root: Node,
}

impl Trie {
    fn new() -> Trie {
        Trie { root: Node::default() }
    }

    fn add_word(&mut self, string: &str, val: Option<u32>) {
        let mut node = &mut self.root;
        for c in string.chars() {
            node = moving(node).chars
                .entry(c)
                .or_insert(Node::default());
        }
        node.is_end_of_word=true;
        node.val = val;
    }

    fn check_word(&mut self, string : &str) -> bool{
        if string == ""{
            return true;
        }
        let curr = &mut self.root;
        for c in string.chars(){
            if !curr.chars.contains_key(&c){
                return false;
            }
        }
        return curr.is_end_of_word;
    }

    fn get_value(&mut self, string : &str) -> Option<u32> {
        let mut curr = &mut self.root;
        for c in string.chars(){
            curr = curr;
        }
        return curr.val;
    }
}

fn moving<T>(t: T) -> T { t }


fn compress(string_to_be_compressed : &str) -> &str{
    let string_length = string_to_be_compressed.len();
    let mut start = 27;
    let output_sequence = "";
    let mut trie_dict = Trie::new();
    let mut current_sequence = string_to_be_compressed.chars().nth(0);
    let mut next_letter = '#';

    for i in 0..string_length{
        if i == string_length - 1{
            next_letter = '#';
        } 
        else{
            next_letter = string_to_be_compressed.chars().nth(i+1);
        }

        let mut con_word = current_sequence + next_letter;

        if trie_dict.check_word(&con_word){
            current_sequence = con_word;
        }
        else{
            output_sequence = output_sequence + trie_dict.get_value(&current_sequence);
            trie_dict.add_word(con_word, start);
            start+=1;
            current_sequence = next_letter;
        };
    }
    return output_sequence;
}


fn main() {
    let string = "TOBEORNOTTOBEORTOBEORNOT";
    let mut output = compress(string);
    println!("{}",output);
    
    let mut trie = Trie::new();
    trie.add_word("foo", Some(1));
    trie.add_word("foobar", Some(2));
    println!("{}",trie.check_word("foo"));
    println!("{:#?}", trie);
}
