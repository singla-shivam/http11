use std::collections::HashMap;
use std::str;

#[derive(Debug, Default)]
struct Node {
    children: HashMap<char, Node>,
    is_end_of_word:bool,
    val: u32,
}

#[derive(Debug)]
struct Trie {
    root: Node,
}

impl Trie {
    fn new() -> Trie {
        
        let mut trie = Trie { root: Node::default()};
        trie.add_word("#", 0);
        for i in 1..27 {
            trie.add_word( str::from_utf8(&[i+64]).unwrap(), i as u32);
        }
        return trie;
    }

    fn add_word(&mut self, string: &str, val: u32) {
        let mut node = &mut self.root;
        for c in string.chars(){
            node = node.children
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
        let mut curr = &self.root;
        for c in string.chars(){
            if !curr.children.contains_key(&c){
                return false;
            }
            curr = curr.children.get(&c).unwrap();
        }
        return curr.is_end_of_word;
    }

    fn get_value(&mut self, string : &str) -> u32 {
        let mut curr = &self.root;
        for c in string.chars(){
            // curr = curr.children.get(&c).unwrap();
            curr = curr.children.get(&c).expect(format!("{}{}",c,string).as_str());
        }
        return curr.val;
    }
}

// fn moving<T>(t: T) -> T { t }


fn compress(string_to_be_compressed : &str) -> String{
    let string_length = string_to_be_compressed.len();
    let mut start = 27;
    let mut output_sequence : String= "".to_string();
    let mut trie_dict = Trie::new();
    let mut current_sequence = string_to_be_compressed.chars().nth(0).unwrap().to_string();
    let mut _next_letter = '#';
    // let mut temp : u32 = 8;
    for i in 0..string_length{
        if i == string_length - 1{
            _next_letter = '#';
        } 
        else{
            _next_letter = string_to_be_compressed.chars().nth(i+1).unwrap();
        }

        let con_word = current_sequence.to_string() + _next_letter.to_string().as_str();

        if trie_dict.check_word(&con_word){
            current_sequence = con_word;
        }
        else{
            // if trie_dict.check_word(&current_sequence){
            //     temp = trie_dict.get_value(&current_sequence).unwrap();
            // }
            output_sequence = output_sequence + &trie_dict.get_value(&current_sequence).to_string();
            trie_dict.add_word(&con_word, start);
            start+=1;
            current_sequence = _next_letter.to_string();
        };
    }
    return output_sequence;
}


fn decompress(vector : &Vec<u8>) -> Vec<String>{
    let mut hash : HashMap<u8 , String> = HashMap::new();
    for j in 1..27 {
        hash.insert(j , String::from_utf8(vec![(j+64) as u8]).unwrap());
    }
    // let mut trie_dict = Trie::new();
    let mut ocode = vector[0];
    let mut out_vec : Vec<String> = Vec::new();
    let mut start = 27;
    // let a = vector[0].to_string();
    out_vec.push(hash.get(&(vector[0])).unwrap().to_string());
    let mut ncode = "".to_string();
    let mut string = "".to_string();
    let mut charac = "".to_string();

    for i in 1..vector.len(){
        let v = vector[i];
        let v = vec![v];
        
        if !hash.contains_key(&(vector[i])){
            string = String::from_utf8(vec![ocode]).unwrap();
            string.push_str(&charac);
        }
        else{
            ncode = hash.get(&vector[i]).unwrap().to_string();
            string = ncode.clone();
            // out_vec.push(string.as_bytes()[0].to_string());
            out_vec.push(hash.get(&(string.as_bytes()[0])).unwrap().to_string());
        }

        charac = string.chars().nth(0).unwrap().to_string();
        let mut c = String::from_utf8(vec![ocode]).unwrap();
        c.push_str(&charac);
        // trie_dict.add_word(c.as_str(), start);
        hash.insert(start, c);
        start += 1;
        ocode = ncode.as_bytes()[0] as u8;
    } 
    return out_vec;
}

fn main() {
    let string = "TOBEORNOTTOBEORTOBEORNOT";
    let output = compress(&string);
    println!("{}",output);
   
    let compressed = vec![20,15,2,5,15,18,14,15,20,27,29,31,36,30,32,34];
    let _uncompressed: Vec<String> = decompress(&compressed);
    println!("{:?}",_uncompressed);
}