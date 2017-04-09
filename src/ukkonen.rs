use std::usize;

pub struct Node {
    start: usize,
    end: usize,
    edges: [Option<usize>; 257],
    suffix_link: Option<usize>,
}

pub struct SuffixTree {
    nodes: Vec<Node>,
}

impl Node {
    pub fn new(start: usize, end: usize) -> Node {
        Node {
            start: start,
            end: end,
            edges: [None; 257],
            suffix_link: None,
        }
    }

    pub fn edge_length(&self) -> usize {
        self.end - self.start
    }
}

impl SuffixTree {
    pub fn new(data: &[u8]) -> SuffixTree {
        let mut nodes = Vec::<Node>::new();
        nodes.push(Node::new(0, 0));
        let mut tree = SuffixTree {
            nodes: nodes,
        };
        tree.extend_tree(data);
        return tree;
    }

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
                        self.nodes.push(Node::new(i, usize::MAX));
                        let leaf = self.nodes.len() - 1;
                        self.nodes[split].edges[data[i] as usize] = Some(leaf);
                        // Make a suffix link to our next node
                        if last_new_node.is_some() {
                            self.nodes[last_new_node.unwrap()].suffix_link = Some(split);
                            last_new_node = None;
                        }
                        last_new_node = Some(split);
                    }
                }
                else {
                    // Rule 2: we create a new leaf edge
                    self.nodes.push(Node::new(i, usize::MAX));
                    let leaf = self.nodes.len() - 1;
                    self.nodes[active_node].edges[active_edge] = Some(leaf);
                    // Make a suffix link if there is a node waiting
                    if last_new_node.is_some() {
                        self.nodes[last_new_node.unwrap()].suffix_link = Some(leaf);
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
                    self.nodes.push(Node::new(usize::MAX, usize::MAX));
                    let leaf = self.nodes.len() - 1;
                    self.nodes[split].edges[256] = Some(leaf);
                    // Make a suffix link to our next node
                    if last_new_node.is_some() {
                        self.nodes[last_new_node.unwrap()].suffix_link = Some(split);
                        last_new_node = None;
                    }
                    last_new_node = Some(split);
                }
            }
            else {
                // Rule 2: we create a new leaf edge
                self.nodes.push(Node::new(usize::MAX, usize::MAX));
                let leaf = self.nodes.len() - 1;
                self.nodes[active_node].edges[active_edge] = Some(leaf);
                // Make a suffix link if there is a node waiting
                if last_new_node.is_some() {
                    self.nodes[last_new_node.unwrap()].suffix_link = Some(leaf);
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
                    let start = match self.nodes[edge].start {
                        usize::MAX => data.len(),
                        s => s
                    };
                    let end = match self.nodes[edge].end {
                        usize::MAX => data.len(),
                        e => e
                    };
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
