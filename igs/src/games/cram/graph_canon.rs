use nauty_Traces_sys::{sparsegraph, size_t, optionblk, statsblk, sparsenauty, sortlists_sg, TRUE, SG_FREE, FALSE};
use crate::dbs::{NimbersProvider, NimbersStorer, HasLen};
use std::collections::HashMap;
use crate::games::cram::Cram;
use std::mem::MaybeUninit;

/// Sparse graph with allocated memory
#[derive(Debug, Default, Clone)]
pub struct SparseGraph {
    pub v: Vec<size_t>,
    pub d: Vec<::std::os::raw::c_int>,
    pub e: Vec<::std::os::raw::c_int>,
}

impl SparseGraph {
    /// Create a sparse graph with the given number of vertices and edges.
    pub fn new(vertices: usize, edges: usize) -> Self {
        SparseGraph {
            v: vec![0; vertices],
            d: vec![0; vertices],
            e: vec![0; edges],
        }
    }

    /// Create a sparse graph with the given maximum numbers of neighbours (out degrees) of vertices.
    pub fn with_max_neighbours_per_vertex<I: IntoIterator<Item=usize>>(max_neighbours: I) -> Self {
        let mut edges = 0usize;
        let v: Vec<_> = max_neighbours.into_iter().map(|n| {
            let result = edges;
            edges += n;
            result as _
        }).collect();
        let d = vec![0; v.len()];
        Self { v, d, e: vec![0; edges] }
    }

    /// Create a sparse graph with the given numbers of vertices and their neighbours (out degrees).
    pub fn with_max_neighbours(number_of_vertices: usize, max_neighbours_per_vertex: usize) -> Self {
        Self::with_max_neighbours_per_vertex((0..number_of_vertices).map(|_| max_neighbours_per_vertex))
    }

    /// Add directed edge.
    fn add_directed_edge(&mut self, from_vertex: usize, to_vertex: usize) {
        self.e[self.v[from_vertex] as usize + self.d[from_vertex] as usize] = to_vertex as _;
        self.d[from_vertex] += 1;
    }

    /// Add undirected edge.
    pub fn add_undirected_edge(&mut self, vertex_a: usize, vertex_b: usize) {
        self.add_directed_edge(vertex_a, vertex_b);
        self.add_directed_edge(vertex_b, vertex_a);
    }
}

impl<'a> std::convert::From<&'a mut SparseGraph> for sparsegraph {
    fn from(g: &'a mut SparseGraph) -> Self {
        sparsegraph {
            nv: g.v.len() as ::std::os::raw::c_int,
            nde: g.d.iter().map(|v| *v as size_t).sum(),
            v: g.v.as_mut_ptr(),
            d: g.d.as_mut_ptr(),
            e: g.e.as_mut_ptr(),
            w: std::ptr::null_mut(),
            vlen: g.v.len() as size_t,
            dlen: g.d.len() as size_t,
            elen: g.e.len() as size_t,
            wlen: 0,
        }
    }
}


fn position_to_graph(cram: &Cram, bitboard: u64) -> SparseGraph {
    // calculate bitboard_to_graph_index mapping and number of vertices (graph_size):
    let mut bitboard_to_graph_index = vec![0u8; cram.board_size() as usize];
    let mut graph_size = 0u8;
    let mut rest = bitboard;
    while rest != 0 {
        let bitboard_index = rest.trailing_zeros();
        rest ^= 1u64 << bitboard_index;
        bitboard_to_graph_index[bitboard_index as usize] = graph_size;
        graph_size += 1;
    }
    // initialize empty graph for graph_size vertices, each vertex have 0, and can have 4 neighbours max
    let graph_size = graph_size as usize;
    let mut result = SparseGraph::with_max_neighbours(graph_size, 4);
    // add edges to our graph:
    rest = bitboard;
    while rest != 0 {
        let current_bitboard_index = rest.trailing_zeros() as usize;
        let current_bitboard_bit = 1u64 << current_bitboard_index;
        rest ^= current_bitboard_bit;
        if cram.shifted_right(current_bitboard_bit) & bitboard != 0 {
            result.add_undirected_edge(
                bitboard_to_graph_index[current_bitboard_index] as usize,
                bitboard_to_graph_index[current_bitboard_index + 1] as usize);
        }
        if cram.shifted_down(current_bitboard_bit) & bitboard != 0 {
            result.add_undirected_edge(
                bitboard_to_graph_index[current_bitboard_index] as usize,
                bitboard_to_graph_index[current_bitboard_index + cram.number_of_cols as usize] as usize);
        }
    }
    result
}

pub struct GraphCanonTT<'c> {
    cram: &'c Cram,
    db: HashMap<Box<[u8]>, u8>
}

impl<'c> GraphCanonTT<'c> {
    pub fn new(cram: &'c Cram) -> Self {
        Self { cram, db: HashMap::new() }
    }

    fn key(&self, position: u64) -> Box<[u8]> {
        let mut options = optionblk::default_sparse();
        let mut stats = statsblk::default();
        options.getcanon = TRUE;
        options.digraph = FALSE;
        options.tc_level = 0;
        //options.schreier = TRUE;
        let mut lab_to_ignore: [MaybeUninit<::std::os::raw::c_int>; 64] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut ptn_to_ignore: [MaybeUninit<::std::os::raw::c_int>; 64] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut orbits_to_ignore: [MaybeUninit<::std::os::raw::c_int>; 64] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut canonical_graph = sparsegraph::default();
        unsafe {
            sparsenauty(
                &mut (&mut position_to_graph(self.cram, position)).into(),
                lab_to_ignore.as_mut_ptr() as *mut ::std::os::raw::c_int,
                ptn_to_ignore.as_mut_ptr() as *mut ::std::os::raw::c_int,
                orbits_to_ignore.as_mut_ptr() as *mut ::std::os::raw::c_int,
                &mut options,
                &mut stats,
                &mut canonical_graph,
            );
            sortlists_sg(&mut canonical_graph);
        }

        let code_len = (canonical_graph.nde / 2) as u8 + canonical_graph.nv as u8;
        let mut result = Vec::with_capacity(code_len as usize);
        result.push(code_len);
        let nv = canonical_graph.nv as usize;
        let v = unsafe { std::slice::from_raw_parts_mut(canonical_graph.v, canonical_graph.vlen as usize) };
        let e = unsafe { std::slice::from_raw_parts_mut(canonical_graph.e, canonical_graph.elen as usize) };
        let d = unsafe { std::slice::from_raw_parts_mut(canonical_graph.d, canonical_graph.dlen as usize) };
        for vertex in 0..nv {
            let beg = v[vertex] as usize;
            result.extend(e[beg..(beg+d[vertex] as usize)].iter().map(|v|*v as u8));
            if vertex+1 != nv {
                result.push(u8::MAX);
            }
        }
        SG_FREE(&mut canonical_graph);
        result.into_boxed_slice()
    }
}

impl<'c> NimbersProvider<u64> for GraphCanonTT<'c> {
    #[inline(always)] fn get_nimber(&self, position: &u64) -> Option<u8> {
        self.db.get_nimber(&self.key(*position))
    }
}

impl<'c> NimbersStorer<u64> for GraphCanonTT<'c> {
    #[inline(always)] fn store_nimber(&mut self, position: u64, nimber: u8) {
        self.db.store_nimber(self.key(position), nimber)
    }
}

impl<'c> HasLen for GraphCanonTT<'c> {
    #[inline(always)] fn len(&self) -> usize { self.db.len() }
}