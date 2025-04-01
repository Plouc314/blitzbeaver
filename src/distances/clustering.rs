use bit_set::BitSet;

pub struct UPGMA {
    clusters: Vec<BitSet>,
    cluster_sizes: Vec<usize>,
    matrix: Vec<Vec<f32>>,
    num_elements: usize,
    threshold_match: f32,
}

impl UPGMA {
    pub fn new(matrix: Vec<Vec<f32>>, threshold_match: f32) -> Self {
        Self {
            clusters: (0..matrix.len())
                .map(|i| {
                    let mut bs = BitSet::new();
                    bs.insert(i);
                    bs
                })
                .collect(),
            cluster_sizes: vec![0; matrix.len()],
            num_elements: matrix.len(),
            matrix,
            threshold_match,
        }
    }

    fn get_row(&self, idx: usize) -> Vec<f32> {
        let mut row = Vec::with_capacity(self.num_elements);
        for i in 0..self.num_elements {
            row.push(self.matrix[idx][i]);
        }
        row
    }

    fn set_row(&mut self, idx: usize, row: Vec<f32>) {
        for i in 0..self.num_elements {
            self.matrix[idx][i] = row[i];
        }
    }

    fn get_col(&self, idx: usize) -> Vec<f32> {
        let mut col = Vec::with_capacity(self.num_elements);
        for i in 0..self.num_elements {
            col.push(self.matrix[i][idx]);
        }
        col
    }

    fn set_col(&mut self, idx: usize, col: Vec<f32>) {
        for i in 0..self.num_elements {
            self.matrix[i][idx] = col[i];
        }
    }

    fn swap_clusters(&mut self, idx_c1: usize, idx_c2: usize) {
        let row1 = self.get_row(idx_c1);
        let row2 = self.get_row(idx_c2);
        let col1 = self.get_col(idx_c1);
        let col2 = self.get_col(idx_c2);

        self.set_row(idx_c2, row1);
        self.set_row(idx_c1, row2);
        self.set_col(idx_c2, col1);
        self.set_col(idx_c1, col2);

        let tmp = self.clusters[idx_c1].clone();
        self.clusters[idx_c1] = self.clusters[idx_c2].clone();
        self.clusters[idx_c2] = tmp;

        let tmp = self.cluster_sizes[idx_c1];
        self.cluster_sizes[idx_c1] = self.cluster_sizes[idx_c2];
        self.cluster_sizes[idx_c2] = tmp;
    }

    fn aggregate_distances(&mut self, idx_c1: usize, idx_c2: usize) -> (Vec<f32>, Vec<f32>) {
        let mut row = Vec::with_capacity(self.num_elements);
        for i in 0..self.num_elements {
            if i == idx_c1 || i == idx_c2 {
                row.push(0.0);
                continue;
            }
            let v1 = self.matrix[idx_c1][i];
            let v2 = self.matrix[idx_c2][i];
            let w1 = self.cluster_sizes[idx_c1] as f32;
            let w2 = self.cluster_sizes[idx_c2] as f32;
            let dist = (v1 * w1) + (v2 * w2) / (w1 + w2);
            row.push(dist);
        }

        let mut col = Vec::with_capacity(self.num_elements);
        for i in 0..self.num_elements {
            if i == idx_c1 || i == idx_c2 {
                col.push(0.0);
                continue;
            }
            let v1 = self.matrix[i][idx_c1];
            let v2 = self.matrix[i][idx_c2];
            let w1 = self.cluster_sizes[idx_c1] as f32;
            let w2 = self.cluster_sizes[idx_c2] as f32;
            let dist = (v1 * w1) + (v2 * w2) / (w1 + w2);
            col.push(dist);
        }
        (row, col)
    }

    fn merge_clusters(&mut self, idx_c1: usize, idx_c2: usize) {
        let (row, col) = self.aggregate_distances(idx_c1, idx_c2);
        self.set_row(idx_c1, row);
        self.set_col(idx_c1, col);

        let c2 = self.clusters[idx_c2].clone();
        self.clusters[idx_c1].union_with(&c2);
        self.cluster_sizes[idx_c1] += self.cluster_sizes[idx_c2];

        if idx_c2 != self.num_elements - 1 {
            self.swap_clusters(idx_c2, self.num_elements - 1);
        }
        self.num_elements -= 1;
    }

    fn find_max_distance_clusters(&self) -> Option<(usize, usize)> {
        let mut max_dist = 0.0;
        let mut max_idxs = None;
        for i in 0..self.num_elements {
            for j in 0..self.num_elements {
                let v = self.matrix[i][j];
                if v > self.threshold_match && v > max_dist {
                    max_dist = v;
                    max_idxs = Some((i, j));
                }
            }
        }
        max_idxs
    }

    fn run(mut self) -> Vec<BitSet> {
        while self.num_elements > 1 {
            match self.find_max_distance_clusters() {
                Some((idx_c1, idx_c2)) => {
                    self.merge_clusters(idx_c1, idx_c2);
                }
                None => {
                    break;
                }
            }
        }
        self.clusters.into_iter().take(self.num_elements).collect()
    }
}
