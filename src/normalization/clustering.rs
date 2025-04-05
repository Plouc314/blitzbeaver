use bit_set::BitSet;

use crate::{distances::CachedDistanceCalculator, word::Word};

/// Computes the clusters of words using the UPGMA algorithm.
pub fn compute_words_clusters<'a>(
    distance_calculator: &mut CachedDistanceCalculator,
    words: Vec<&'a Word>,
    threshold_match: f32,
) -> Vec<BitSet> {
    let mut matrix = Vec::with_capacity(words.len());
    for i in 0..words.len() {
        let mut row = Vec::with_capacity(words.len());
        for j in 0..words.len() {
            if i == j {
                row.push(0.0);
            } else {
                let dist = distance_calculator.get_dist_word(&words[i], &words[j]);
                row.push(dist);
            }
        }
        matrix.push(row);
    }

    let upgma = UPGMA::new(matrix, threshold_match);
    upgma.run()
}

/// UPGMA clustering algorithm implementation.
///
/// Executes the UPGMA algorithm until the maximum distance between
/// the clusters is below the threshold.
pub struct UPGMA {
    /// bit sets of the index of the elements in each cluster
    clusters: Vec<BitSet>,
    /// sizes of each cluster
    cluster_sizes: Vec<usize>,
    /// distance matrix
    matrix: Vec<Vec<f32>>,
    /// current number of clusters
    /// (decreases as clusters are merged)
    num_clusters: usize,
    /// threshold for merging clusters
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
            cluster_sizes: vec![1; matrix.len()],
            num_clusters: matrix.len(),
            matrix,
            threshold_match,
        }
    }

    fn get_row(&self, idx: usize) -> Vec<f32> {
        let mut row = Vec::with_capacity(self.num_clusters);
        for i in 0..self.num_clusters {
            row.push(self.matrix[idx][i]);
        }
        row
    }

    fn set_row(&mut self, idx: usize, row: Vec<f32>) {
        for i in 0..self.num_clusters {
            self.matrix[idx][i] = row[i];
        }
    }

    fn get_col(&self, idx: usize) -> Vec<f32> {
        let mut col = Vec::with_capacity(self.num_clusters);
        for i in 0..self.num_clusters {
            col.push(self.matrix[i][idx]);
        }
        col
    }

    fn set_col(&mut self, idx: usize, col: Vec<f32>) {
        for i in 0..self.num_clusters {
            self.matrix[i][idx] = col[i];
        }
    }

    /// Swaps the clusters at the given indices.
    ///
    /// This is useful to move a deleted cluster to the end of the matrix
    /// to avoid resizing the matrix after each merge.
    fn swap_clusters(&mut self, idx_c1: usize, idx_c2: usize) {
        let dist_12 = self.matrix[idx_c1][idx_c2];
        let row1 = self.get_row(idx_c1);
        let row2 = self.get_row(idx_c2);
        let col1 = self.get_col(idx_c1);
        let col2 = self.get_col(idx_c2);

        self.set_row(idx_c2, row1);
        self.set_row(idx_c1, row2);
        self.set_col(idx_c2, col1);
        self.set_col(idx_c1, col2);

        self.matrix[idx_c1][idx_c2] = dist_12;
        self.matrix[idx_c2][idx_c1] = dist_12;
        self.matrix[idx_c1][idx_c1] = 0.0;
        self.matrix[idx_c2][idx_c2] = 0.0;

        let tmp = self.clusters[idx_c1].clone();
        self.clusters[idx_c1] = self.clusters[idx_c2].clone();
        self.clusters[idx_c2] = tmp;

        let tmp = self.cluster_sizes[idx_c1];
        self.cluster_sizes[idx_c1] = self.cluster_sizes[idx_c2];
        self.cluster_sizes[idx_c2] = tmp;
    }

    /// Computes the new distances between the merged clusters and the rest of the clusters.
    ///
    /// Returns the new row and column of distances.
    fn aggregate_distances(&mut self, idx_c1: usize, idx_c2: usize) -> (Vec<f32>, Vec<f32>) {
        let mut row = Vec::with_capacity(self.num_clusters);
        for i in 0..self.num_clusters {
            if i == idx_c1 || i == idx_c2 {
                row.push(0.0);
                continue;
            }
            let v1 = self.matrix[idx_c1][i];
            let v2 = self.matrix[idx_c2][i];
            let w1 = self.cluster_sizes[idx_c1] as f32;
            let w2 = self.cluster_sizes[idx_c2] as f32;
            let dist = ((v1 * w1) + (v2 * w2)) / (w1 + w2);
            row.push(dist);
        }

        let mut col = Vec::with_capacity(self.num_clusters);
        for i in 0..self.num_clusters {
            if i == idx_c1 || i == idx_c2 {
                col.push(0.0);
                continue;
            }
            let v1 = self.matrix[i][idx_c1];
            let v2 = self.matrix[i][idx_c2];
            let w1 = self.cluster_sizes[idx_c1] as f32;
            let w2 = self.cluster_sizes[idx_c2] as f32;
            let dist = ((v1 * w1) + (v2 * w2)) / (w1 + w2);
            col.push(dist);
        }
        (row, col)
    }

    /// Merges the two clusters at the given indices.
    fn merge_clusters(&mut self, idx_c1: usize, idx_c2: usize) {
        let (row, col) = self.aggregate_distances(idx_c1, idx_c2);

        self.set_row(idx_c1, row);
        self.set_col(idx_c1, col);

        let c2 = self.clusters[idx_c2].clone();
        self.clusters[idx_c1].union_with(&c2);
        self.cluster_sizes[idx_c1] += self.cluster_sizes[idx_c2];

        if idx_c2 != self.num_clusters - 1 {
            self.swap_clusters(idx_c2, self.num_clusters - 1);
        }
        self.num_clusters -= 1;
    }

    /// Finds the two clusters with the maximum distance.
    ///
    /// Returns the indices of the clusters or None if no distance is above the threshold.
    fn find_max_distance_clusters(&self) -> Option<(usize, usize)> {
        let mut max_dist = 0.0;
        let mut max_idxs = None;
        for i in 0..self.num_clusters {
            for j in 0..self.num_clusters {
                let v = self.matrix[i][j];
                if v > self.threshold_match && v > max_dist {
                    max_dist = v;
                    max_idxs = Some((i, j));
                }
            }
        }
        max_idxs
    }

    /// Runs the UPGMA algorithm until the maximum distance between
    /// the clusters is below the threshold.
    fn run(mut self) -> Vec<BitSet> {
        while self.num_clusters > 1 {
            match self.find_max_distance_clusters() {
                Some((idx_c1, idx_c2)) => {
                    self.merge_clusters(idx_c1, idx_c2);
                }
                None => {
                    break;
                }
            }
        }
        self.clusters.into_iter().take(self.num_clusters).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bit_set::BitSet;

    #[test]
    fn test_upgma_swap_clusters() {
        let matrix = vec![
            vec![0.0, 1.0, 2.0, 4.0],
            vec![1.0, 0.0, 3.0, 5.0],
            vec![2.0, 3.0, 0.0, 6.0],
            vec![4.0, 5.0, 6.0, 0.0],
        ];

        let mut upgma = UPGMA::new(matrix.clone(), 0.0);

        upgma.swap_clusters(0, 3);

        let swapped_matrix = vec![
            vec![0.0, 5.0, 6.0, 4.0],
            vec![5.0, 0.0, 3.0, 1.0],
            vec![6.0, 3.0, 0.0, 2.0],
            vec![4.0, 1.0, 2.0, 0.0],
        ];

        assert_eq!(upgma.matrix, swapped_matrix);
    }

    #[test]
    fn test_upgma_single_cluster() {
        let matrix = vec![
            vec![0.0, 1.0, 2.0],
            vec![1.0, 0.0, 3.0],
            vec![2.0, 3.0, 0.0],
        ];
        let threshold_match = 2.0;
        let upgma = UPGMA::new(matrix.clone(), threshold_match);
        let clusters = upgma.run();
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0], BitSet::from_iter(vec![0]));
        assert_eq!(clusters[1], BitSet::from_iter(vec![1, 2]));
    }

    #[test]
    fn test_upgma_dual_clusters() {
        let matrix = vec![
            vec![0.0, 1.0, 0.2, 0.3],
            vec![1.0, 0.0, 0.1, 0.2],
            vec![0.2, 0.1, 0.0, 0.9],
            vec![0.3, 0.2, 0.9, 0.0],
        ];
        let threshold_match = 0.6;
        let upgma = UPGMA::new(matrix.clone(), threshold_match);
        let clusters = upgma.run();
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0], BitSet::from_iter(vec![0, 1]));
        assert_eq!(clusters[1], BitSet::from_iter(vec![2, 3]));
    }
}
