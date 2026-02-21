/// Cluster nearby values together.
/// Returns groups of indices where values are within `tolerance` of each other.
pub fn cluster_values(values: &[f64], tolerance: f64) -> Vec<Vec<usize>> {
    if values.is_empty() {
        return vec![];
    }

    let mut indexed: Vec<(usize, f64)> = values.iter().copied().enumerate().collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut clusters: Vec<Vec<usize>> = vec![vec![indexed[0].0]];

    for &(idx, val) in &indexed[1..] {
        let last_cluster = clusters.last().unwrap();
        let last_idx = *last_cluster.last().unwrap();
        let last_val = values[last_idx];

        if (val - last_val).abs() <= tolerance {
            clusters.last_mut().unwrap().push(idx);
        } else {
            clusters.push(vec![idx]);
        }
    }

    clusters
}

/// Cluster values and return representative (mean) values for each cluster.
pub fn cluster_to_means(values: &[f64], tolerance: f64) -> Vec<f64> {
    let clusters = cluster_values(values, tolerance);
    clusters
        .iter()
        .map(|cluster| {
            let sum: f64 = cluster.iter().map(|&i| values[i]).sum();
            sum / cluster.len() as f64
        })
        .collect()
}

/// Given a set of positions, find distinct coordinate lines (for table detection).
/// Returns sorted, deduplicated coordinate values after clustering.
pub fn find_grid_lines(positions: &[f64], tolerance: f64) -> Vec<f64> {
    if positions.is_empty() {
        return vec![];
    }

    let mut means = cluster_to_means(positions, tolerance);
    means.sort_by(|a, b| a.partial_cmp(b).unwrap());
    means
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_values() {
        let vals = vec![1.0, 1.5, 2.0, 10.0, 10.5, 11.0, 20.0];
        let clusters = cluster_values(&vals, 1.5);
        assert_eq!(clusters.len(), 3);
    }

    #[test]
    fn test_find_grid_lines() {
        let positions = vec![10.0, 10.1, 10.2, 50.0, 50.1, 90.0, 90.2];
        let lines = find_grid_lines(&positions, 1.0);
        assert_eq!(lines.len(), 3);
    }
}
