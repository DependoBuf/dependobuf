use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub struct TopSortBuilder<T> {
    mapping: HashMap<T, u32>,
    reverse_mapping: Vec<T>,
    unused_index: u32,
    graph: internal::Graph,
}

impl<T> Default for TopSortBuilder<T> {
    fn default() -> Self {
        Self {
            mapping: Default::default(),
            reverse_mapping: Default::default(),
            unused_index: 0,
            graph: Default::default(),
        }
    }
}

impl<T> TopSortBuilder<T>
where
    T: Clone + Hash + Eq,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_item(&mut self, item: &T) -> Result<u32, u32> {
        let new_index = match self.mapping.insert(item.clone(), self.unused_index) {
            None => self.unused_index,
            Some(index) => return Err(index),
        };
        self.reverse_mapping.push(item.clone());
        self.graph.add_vertex();
        self.unused_index += 1;
        Ok(new_index)
    }

    pub fn get_index(&self, item: &T) -> Option<u32> {
        self.mapping.get(item).copied()
    }

    pub fn get_index_or_insert(&mut self, item: &T) -> u32 {
        match self.get_index(item) {
            None => self.add_item(item).expect("Item not added yet"),
            Some(index) => index,
        }
    }

    pub fn add_edge(&mut self, lhs: &T, rhs: &T) {
        let v = self.get_index_or_insert(lhs);
        let u = self.get_index_or_insert(rhs);
        self.graph.add_edge(v, u);
    }

    pub fn top_sort(self) -> Vec<T> {
        let order = self.graph.top_sort();
        order
            .into_iter()
            .map(|i| self.reverse_mapping[i as usize].clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_graph() {
        let mut builder: TopSortBuilder<String> = TopSortBuilder::new();
        builder.add_edge(&"A".to_owned(), &"B".to_owned());
        builder.add_edge(&"B".to_owned(), &"C".to_owned());
        let order = builder.top_sort();
        assert_eq!(order, vec!["C".to_owned(), "B".to_owned(), "A".to_owned()])
    }
}

mod internal {
    #[derive(Debug, Default, Clone, Copy)]
    enum VertexStatus {
        #[default]
        NotVisited,
        Visiting,
        Visited,
    }

    #[derive(Debug, Default)]
    pub struct Graph {
        adjacency_list: Vec<Vec<u32>>,
        visited: Vec<VertexStatus>,
        order: Vec<u32>,
    }

    impl Graph {
        fn neighbors(&self, v: u32) -> &[u32] {
            &self.adjacency_list[v as usize]
        }

        fn neighbors_mut(&mut self, v: u32) -> &mut Vec<u32> {
            &mut self.adjacency_list[v as usize]
        }

        fn status(&self, v: u32) -> VertexStatus {
            self.visited[v as usize]
        }

        fn status_mut(&mut self, v: u32) -> &mut VertexStatus {
            &mut self.visited[v as usize]
        }

        pub fn add_vertex(&mut self) {
            self.adjacency_list.push(vec![]);
            self.visited.push(VertexStatus::NotVisited);
        }

        pub fn add_edge(&mut self, a: u32, b: u32) {
            self.neighbors_mut(a).push(b);
        }

        fn find_order(&mut self, v: u32) -> bool {
            use VertexStatus::*;

            *self.status_mut(v) = Visiting;
            let neighbors_count = self.neighbors(v).len();

            for i in 0..neighbors_count {
                let u = self.neighbors(v)[i];
                if let Visiting = self.status(u) {
                    return true;
                }
                let result = self.find_order(u);
                if result == true {
                    return true;
                }
            }

            *self.status_mut(v) = Visited;
            self.order.push(v);

            false
        }

        pub fn top_sort(mut self) -> Vec<u32> {
            let vertices_count = self.adjacency_list.len() as u32;
            for v in 0..vertices_count {
                if let VertexStatus::Visited = self.status(v) {
                    continue;
                }
                self.find_order(v);
            }
            self.order
        }
    }
}
