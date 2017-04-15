//! TreeMatch is a binary matching algorithm based on a suffix tree to retrieve matching strings. 
//!
//! The suffix tree is built in linear time using Ukkonen's algorithm.

use std::collections::HashMap;
use std::iter::Iterator;
use std::usize;

use Match;

/// A node in the [`SuffixTree`](struct.SuffixTree.html)
pub struct Node {
    /// The index in the data where the edge leading to this node starts.
    pub start: usize,
    /// The index in the data where the edge leading to this node ends.
    pub end: usize,
    /// The potential sub nodes under this one. Each index in the array represent on of the 
    /// possible byte value. The index `256` is reserved for the end of data. Each element value is 
    /// an index in the `SuffixTree::nodes`(struct.SuffixTree.html#nodes.v) vector.
    pub edges: [Option<usize>; 257],
    /// Suffix link (see Ukkonen's algorithm).
    pub suffix_link: Option<usize>,
}

/// A suffix tree.
pub struct SuffixTree {
    /// A vector of [`Node`](struct.Node.html) composing this tree. The first element is the root 
    /// node.
    pub nodes: Vec<Node>,
}

impl Node {
    /// Allocate a new node with a leading edge `[start..end]`.
    pub fn new(start: usize, end: usize) -> Node {
        Node {
            start: start,
            end: end,
            edges: [None; 257],
            suffix_link: None,
        }
    }
    /// Returns this node leading edge length.
    pub fn edge_length(&self) -> usize {
        self.end - self.start
    }
}

impl SuffixTree {
    /// Build a new suffix tree for `data` using Ukkonen's algorithm.
    pub fn new(data: &[u8]) -> SuffixTree {
        let mut nodes = Vec::<Node>::new();
        nodes.push(Node::new(0, 0));
        let mut tree = SuffixTree {
            nodes: nodes,
        };
        tree.extend_tree(data);
        return tree;
    }

    #[allow(unused_assignments)]
    fn extend_tree(&mut self, data: &[u8]) {
        let mut last_new_node: Option<usize> = None;
        let mut active_node: usize = 0;
        let mut active_length: usize = 0;
        let mut active_edge: usize = data[0] as usize;
        let mut remaining_suffix: usize = 0;
        for i in 0..data.len() {
            last_new_node = None;
            remaining_suffix += 1;
            while remaining_suffix > 0 {
                // Active length is zero, so the current character is data[i] and no walk down is needed.
                if active_length == 0 {
                    active_edge = data[i] as usize;
                }
                if let Some(next_node) = self.nodes[active_node].edges[active_edge] {
                    // If the active length is longer than the current edge, we walk down the edge 
                    // to the next node.
                    if active_length >= self.nodes[next_node].edge_length() {
                        active_node = next_node;
                        active_edge = data[self.nodes[next_node].end] as usize;
                        active_length -= self.nodes[next_node].edge_length();
                        continue;
                    }
                    // Rule 3: the current character is on the edge
                    else if data[self.nodes[next_node].start + active_length] == data[i] {
                        // Make a suffix link to the active node if there is a node waiting and if 
                        // the active node is not the root node
                        if last_new_node.is_some() && active_node > 0 {
                            self.nodes[last_new_node.unwrap()].suffix_link = Some(active_node);
                            last_new_node = None;
                        }
                        active_length += 1;
                        break;
                    }
                    // We need to split the edge at the current character
                    else {
                        let start = self.nodes[next_node].start;
                        let split_pos = self.nodes[next_node].start + active_length;
                        self.nodes.push(Node::new(start, split_pos));
                        let split = self.nodes.len() - 1;
                        self.nodes[next_node].start = split_pos;
                        self.nodes[active_node].edges[data[start] as usize] = Some(split);
                        self.nodes[split].edges[data[split_pos] as usize] = Some(next_node);
                        self.nodes.push(Node::new(i, data.len()));
                        let leaf = self.nodes.len() - 1;
                        self.nodes[split].edges[data[i] as usize] = Some(leaf);
                        // Make a suffix link to our next node
                        if last_new_node.is_some() {
                            self.nodes[last_new_node.unwrap()].suffix_link = Some(split);
                        }
                        last_new_node = Some(split);
                    }
                }
                else {
                    // Rule 2: we create a new leaf edge
                    self.nodes.push(Node::new(i, data.len()));
                    let leaf = self.nodes.len() - 1;
                    self.nodes[active_node].edges[active_edge] = Some(leaf);
                    // Make a suffix link if there is a node waiting
                    if last_new_node.is_some() {
                        self.nodes[last_new_node.unwrap()].suffix_link = Some(active_node);
                        last_new_node = None;
                    }
                }
                
                remaining_suffix -= 1;
                if active_node == 0 && active_length > 0 {
                    active_length -= 1;
                    active_edge = data[i - remaining_suffix + 1] as usize;
                }
                else if active_node != 0 {
                    active_node = match self.nodes[active_node].suffix_link {
                        Some(linked) => linked,
                        None => 0
                    };
                }
            }
        }
        // Simulate end character by doing another step with false character 256
        remaining_suffix += 1;
        while remaining_suffix > 0 {
            // Active length is zero, so the current character is *i* and no walk down is needed.
            if active_length == 0 {
                // Special end character
                active_edge = 256;
            }
            if let Some(next_node) = self.nodes[active_node].edges[active_edge] {
                // If the active length is longer than the current edge, we walk down the edge
                if active_length >= self.nodes[next_node].edge_length() {
                    active_edge += self.nodes[next_node].edge_length();
                    active_length -= self.nodes[next_node].edge_length();
                    active_node = next_node;
                    continue;
                }
                else if self.nodes[next_node].start + active_length == data.len() {
                    // Make a suffix link to the active node if there is a node waiting and if 
                    // the active node is not the root node
                    if last_new_node.is_some() && active_node > 0 {
                        self.nodes[last_new_node.unwrap()].suffix_link = Some(active_node);
                        last_new_node = None;
                    }
                    active_length += 1;
                    break;
                }
                // We need to split the edge at the current character
                else {
                    let start = self.nodes[next_node].start;
                    let split_pos = self.nodes[next_node].start + active_length;
                    self.nodes.push(Node::new(start, split_pos));
                    let split = self.nodes.len() - 1;
                    self.nodes[next_node].start = split_pos;
                    self.nodes[active_node].edges[data[start] as usize] = Some(split);
                    self.nodes[split].edges[data[split_pos] as usize] = Some(next_node);
                    self.nodes.push(Node::new(data.len(), data.len()));
                    let leaf = self.nodes.len() - 1;
                    self.nodes[split].edges[256] = Some(leaf);
                    // Make a suffix link to our next node
                    if last_new_node.is_some() {
                        self.nodes[last_new_node.unwrap()].suffix_link = Some(split);
                    }
                    last_new_node = Some(split);
                }
            }
            else {
                // Rule 2: we create a new leaf edge
                self.nodes.push(Node::new(data.len(), data.len()));
                let leaf = self.nodes.len() - 1;
                self.nodes[active_node].edges[active_edge] = Some(leaf);
                // Make a suffix link if there is a node waiting
                if last_new_node.is_some() {
                    self.nodes[last_new_node.unwrap()].suffix_link = Some(active_node);
                    last_new_node = None;
                }
            }
            
            remaining_suffix -= 1;
            if active_node == 0 && active_length > 0 {
                active_length -= 1;
                if remaining_suffix < 2 {
                    active_edge = 256;
                }
                else {
                    active_edge = data[data.len() - remaining_suffix + 1] as usize;
                }
            }
            else if active_node != 0 {
                active_node = match self.nodes[active_node].suffix_link {
                    Some(linked) => linked,
                    None => 0
                };
            }
        }
    }

    pub fn to_graphviz(&self, data: &[u8]) -> String {
        let mut graphviz = String::new();
        graphviz.push_str("digraph {\n");
        for i in 0..self.nodes.len() {
            graphviz.push_str(&format!("    NODE_{};\n", i));
        }
        for i in 0..self.nodes.len() {
            for j in 0..self.nodes[i].edges.len() {
                if let Some(edge) = self.nodes[i].edges[j] {
                    let start = self.nodes[edge].start;
                    let end = self.nodes[edge].end;
                    if let Ok(s) = String::from_utf8(data[start..end].to_owned()) {
                        graphviz.push_str(&format!("    NODE_{} -> NODE_{} [ label = \"{}\" ];\n", i, edge, &s));
                    }
                    else {
                        graphviz.push_str(&format!("    NODE_{} -> NODE_{} [ label = \"{:?}\" ];\n", i, edge, &data[start..end]));
                    }
                }
            }
            if let Some(linked) = self.nodes[i].suffix_link {
                graphviz.push_str(&format!("    NODE_{} -> NODE_{} [ style = \"dashed\" ];\n", i, linked));
            }
        }
        graphviz.push_str("}");
        return graphviz;
    }
}

/// An iterator over all the [`Match`](../struct.Match.html) bewteen two pieces of data.
///
/// # Examples
/// 
/// ```
/// use bcmp::treematch::TreeMatchIterator;
///
/// let a = "abcdefg";
/// let b = "012abc34cdef56efg78abcdefg";
/// let match_iter = TreeMatchIterator::new(a.as_bytes(), b.as_bytes(), 2);
/// for m in match_iter {
///     println!("Match: {:}", &a[m.first_pos..m.first_end()]);
/// }
/// ```
pub struct TreeMatchIterator<'a> {
    first: &'a [u8],
    second: &'a [u8],
    tree: SuffixTree,
    minimal_length: usize,
    i: usize,
    backtrace: Vec<(usize,usize)>,
    match_length: usize,
    depth: usize,
    matched: HashMap<isize, usize>
}

impl<'a> TreeMatchIterator<'a> {
    /// Allocate a new iterator over the matches between two byte slices with a minimal matching length.
    pub fn new(first: &'a[u8], second: &'a [u8], minimal_length: usize) -> TreeMatchIterator<'a> {
        let tree = SuffixTree::new(first);
        TreeMatchIterator {
            first: first,
            second: second,
            tree: tree,
            minimal_length: minimal_length,
            i: 0,
            backtrace: Vec::new(),
            match_length: 0,
            depth: 0,
            matched: HashMap::new()
        }
    }
    /// Reset the iterator to its start. This allows to iterate multiple times over the matches 
    /// without wasting time regenerating the `HashMap`.
    pub fn reset(&mut self) {
        self.i = 0;
        self.backtrace.clear();
        self.matched.clear();
    }
}

impl<'a> Iterator for TreeMatchIterator<'a> {
    type Item = Match;
    fn next(&mut self) -> Option<Match> {
        while self.i < self.second.len() {
            // Starting a backtrace at position i
            if self.backtrace.is_empty() {
                self.match_length = 0;
                self.depth = 0;
                let mut cur = 0;
                // Dive of at least minimal length
                while self.match_length == self.depth && self.match_length < self.minimal_length {
                    let second_idx = self.i + self.depth;
                    if second_idx >= self.second.len() {
                        break;
                    }
                    if let Some(next) = self.tree.nodes[cur].edges[self.second[second_idx] as usize] {
                        for j in 0..self.tree.nodes[next].edge_length() {
                            let first_idx = self.tree.nodes[next].start + j;
                            let second_idx = self.i + self.depth + j;
                            if second_idx < self.second.len() && self.first[first_idx] == self.second[second_idx] {
                                self.match_length += 1;
                            }
                            else {
                                break;
                            }
                        }
                        self.depth += self.tree.nodes[next].edge_length();
                        cur = next;
                    }
                    else {
                        break;
                    }
                }
                // Was the dive successful? If not, go to the next index in second
                if self.match_length < self.minimal_length {
                    self.i += 1;
                    continue;
                }
                // Mark this node as the start
                self.backtrace.push((cur,0));
            }
            while self.backtrace.len() > 0 {
                let (cur, mut idx) = self.backtrace.last().unwrap().clone();
                while idx < 257 {
                    if let Some(next) = self.tree.nodes[cur].edges[idx] {
                        // Are we still matching? or just enumerating the terminating leaf?
                        if self.match_length == self.depth {
                            for j in 0..self.tree.nodes[next].edge_length() {
                                let first_idx = self.tree.nodes[next].start + j;
                                let second_idx = self.i + self.depth + j;
                                if second_idx < self.second.len() && self.first[first_idx] == self.second[second_idx] {
                                    self.match_length += 1;
                                }
                                else {
                                    break;
                                }
                            }
                        }
                        // Update the idx
                        self.backtrace.last_mut().unwrap().1 = idx + 1;
                        // Go down
                        self.depth += self.tree.nodes[next].edge_length();
                        self.backtrace.push((next,0));
                        break;
                    }
                    idx += 1;
                }
                // If we are still on the same node
                if cur == self.backtrace.last().unwrap().0 {
                    // If we went over all the possible edges without finding a node, we were on a leaf
                    if self.backtrace.last().unwrap().1 == 0 {
                        // Update the idx
                        self.backtrace.last_mut().unwrap().1 = idx + 1;
                        // Handle the match
                        let m = Match::new(self.tree.nodes[cur].end - self.depth, self.i, self.match_length);
                        let delta = m.first_pos as isize - m.second_pos as isize;
                        if !(self.matched.contains_key(&delta) && self.matched.get(&delta).unwrap() > &m.second_pos) {
                            self.matched.insert(delta, m.second_pos + m.length);
                            return Some(m);
                        }
                    }
                    // Else we just backtrack
                    else {
                        // Go up one level
                        self.depth -= self.tree.nodes[cur].edge_length();
                        // Update the match length
                        if self.depth < self.match_length {
                            self.match_length = self.depth;
                        }
                        self.backtrace.pop();
                    }
                }
            }
            self.i += 1;
        }
        return None;
    }
}
