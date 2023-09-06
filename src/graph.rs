use std::collections::{HashMap, HashSet};
use std::hash::Hash;

// An undirected graph as an "adjacency list", i.e., a map from
// vertices to the set of neighbors

// Invariant: Edges are undirected so v1 is in v2s set of neighbors if
// and only if v2 is in v1s set of neigbhors
#[derive(Debug, Clone)]
pub struct Graph<V> {
    g: HashMap<V, HashSet<V>>,
}

impl<V: Eq + Hash> Graph<V> {
    /* Returns the empty graph */
    pub fn new() -> Graph<V> {
        Graph { g: HashMap::new() }
    }

    /* Inserts a vertex into the graph */
    pub fn insert_vertex(&mut self, v: V) {
        match self.g.get(&v) {
            None => {
                self.g.insert(v, HashSet::new());
            }
            Some(_) => {}
        }
    }

    /* Inserts an (undirected) edge between v1 and v2 */
    pub fn insert_edge(&mut self, v1: V, v2: V)
    where
        V: Clone,
    {
        self.insert_vertex(v1.clone());
        self.insert_vertex(v2.clone());
        match self.g.get_mut(&v1) {
            None => panic!("impossible"),
            Some(es1) => {
                let _ = es1.insert(v2.clone());
            }
        }
        match self.g.get_mut(&v2) {
            None => panic!("impossible"),
            Some(es1) => {
                let _ = es1.insert(v1);
            }
        }
    }

    /* Returns a vector of all the vertices in the graph */
    pub fn vertices(&self) -> Vec<V>
    where
        V: Clone,
    {
        self.g.keys().cloned().collect()
    }

    /* Given a vertex v, returns None if the v is not present in the
     * graph and otherwise returns the set of vertices that v has an
     * edge to. */

    pub fn neighbors(&self, v: &V) -> Option<&HashSet<V>> {
        self.g.get(v)
    }

    /* Removes a vertex from the graph, returning true if the vertex
     * was present and false if it wasn't */
    pub fn remove_vertex(&mut self, v: &V) -> bool {
        let ans = self.g.remove(v).is_some();
        for es in self.g.values_mut() {
            es.remove(v);
        }
        ans
    }

    /* Returns the number of vertices in the graph */
    pub fn num_vertices(&self) -> usize {
        self.g.keys().len()
    }

    /* Checks if an edge between v1 and v2 exists */
    pub fn contains_edge(&self, v1: &V, v2: &V) -> bool {
        match self.g.get(v1) {
            Some(neigbhors) => neigbhors.contains(v2),
            None => false,
        }
    }
}
