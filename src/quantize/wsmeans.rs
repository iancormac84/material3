use std::{cmp::Ordering, iter::zip, time::Instant};

use std::collections::HashMap;
use rand::{rngs::StdRng, Rng, SeedableRng};

use super::{
    point_provider::PointProvider, point_provider_lab::PointProviderLab, Quantizer, QuantizerResult,
};

#[derive(Debug, Clone)]
struct DistanceAndIndex {
    distance: f64,
    index: usize,
}

impl DistanceAndIndex {
    pub fn new(distance: f64, index: usize) -> DistanceAndIndex {
        DistanceAndIndex { distance, index }
    }
}

impl PartialEq for DistanceAndIndex {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl PartialOrd for DistanceAndIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

pub struct QuantizerWsmeans {
    pub debug: bool,
    pub starting_clusters: Vec<u32>,
    pub point_provider: PointProviderLab,
    pub max_iterations: i32,
    pub return_input_pixel_to_cluster_pixel: bool,
}

impl Default for QuantizerWsmeans {
    fn default() -> QuantizerWsmeans {
        Self {
            debug: true,
            starting_clusters: vec![],
            point_provider: PointProviderLab,
            max_iterations: 5,
            return_input_pixel_to_cluster_pixel: false,
        }
    }
}
impl Quantizer for QuantizerWsmeans {
    fn quantize(&mut self, input_pixels: &[u32], max_colors: u32) -> QuantizerResult {
        let mut random = StdRng::seed_from_u64(0x42688);
        let mut pixel_to_count = HashMap::new();
        let mut points = vec![];
        let mut pixels = vec![];
        let mut point_count = 0;
        for input_pixel in input_pixels {
            *pixel_to_count.entry(input_pixel).or_insert(0) += 1;
            let pixel_count = pixel_to_count[input_pixel];
            if pixel_count == 1 {
                point_count += 1;
                points.push(self.point_provider.from_int(*input_pixel));
                pixels.push(input_pixel);
            }
        }

        let mut counts = vec![0; point_count];
        for i in 0..point_count {
            let pixel = pixels[i];
            let count = *pixel_to_count.get(&pixel).unwrap();
            counts[i] = count;
        }

        let cluster_count = max_colors.min(point_count as u32) as usize;

        let mut clusters: Vec<[f64; 3]> = self
            .starting_clusters
            .iter()
            .map(|e| self.point_provider.from_int(*e))
            .collect();
        let additional_clusters_needed = cluster_count - clusters.len();
        if additional_clusters_needed > 0 {
            let mut indices = vec![];
            for _ in 0..additional_clusters_needed {
                // Use existing points rather than generating random centroids.
                //
                // KMeans is extremely sensitive to initial clusters. This quantizer
                // is meant to be used with a Wu quantizer that provides initial
                // centroids, but Wu is very slow on unscaled images and when extracting
                // more than 256 colors.
                //
                // Here, we can safely assume that more than 256 colors were requested
                // for extraction. Generating random centroids tends to lead to many
                // "empty" centroids, as the random centroids are nowhere near any pixels
                // in the image, and the centroids from Wu are very refined and close
                // to pixels in the image.
                //
                // Rather than generate random centroids, we'll pick centroids that
                // are actual pixels in the image, and avoid duplicating centroids.

                let mut index = random.gen_range(0..points.len());
                while indices.contains(&index) {
                    index = random.gen_range(0..points.len());
                }
                indices.push(index);
            }

            for index in indices {
                clusters.push(points[index]);
            }
        }
        if self.debug {
            println!(
                "have {} starting clusters, {} points",
                clusters.len(),
                points.len()
            );
        }

        let mut cluster_indices: Vec<usize> = (0..point_count)
            .map(|index| index % cluster_count)
            .collect();
        let mut index_matrix = vec![vec![0; cluster_count]; cluster_count];

        let mut distance_to_index_matrix: Vec<Vec<DistanceAndIndex>> = (0..cluster_count)
            .map(|_| {
                (0..cluster_count)
                    .map(|index| DistanceAndIndex::new(0.0, index))
                    .collect()
            })
            .collect();

        let mut pixel_count_sums = vec![0; cluster_count];
        for iteration in 0..self.max_iterations {
            if self.debug {
                for i in 0..cluster_count {
                    pixel_count_sums[i] = 0;
                }
                for i in 0..point_count {
                    let cluster_index = cluster_indices[i];
                    let count = counts[i];
                    pixel_count_sums[cluster_index] += count;
                }
                let mut empty_clusters = 0;
                for cluster in 0..cluster_count {
                    if pixel_count_sums[cluster] == 0 {
                        empty_clusters += 1;
                    }
                }
                println!(
                    "starting iteration {}; {} clusters are empty of {}",
                    iteration + 1,
                    empty_clusters,
                    cluster_count
                );
            }

            let mut points_moved = 0;
            for i in 0..cluster_count {
                for j in i + 1..cluster_count {
                    let distance = self.point_provider.distance(&clusters[i], &clusters[j]);
                    distance_to_index_matrix[j][i].distance = distance;
                    distance_to_index_matrix[j][i].index = i;
                    distance_to_index_matrix[i][j].distance = distance;
                    distance_to_index_matrix[i][j].index = j;
                }
                //distance_to_index_matrix[i].sort();
                for j in 0..cluster_count {
                    index_matrix[i][j] = distance_to_index_matrix[i][j].index;
                }
            }

            for i in 0..point_count {
                let point = points[i];
                let previous_cluster_index = cluster_indices[i];
                let previous_cluster = &clusters[previous_cluster_index][..];
                let previous_distance = self.point_provider.distance(&point, previous_cluster);
                let mut minimum_distance = previous_distance;
                let mut new_cluster_index: isize = -1;
                for j in 0..cluster_count {
                    if distance_to_index_matrix[previous_cluster_index][j].distance
                        >= 4.0 * previous_distance
                    {
                        continue;
                    }
                    let distance = self.point_provider.distance(&point, &clusters[j]);
                    if distance < minimum_distance {
                        minimum_distance = distance;
                        new_cluster_index = j as isize;
                    }
                }
                if new_cluster_index != -1 {
                    points_moved += 1;
                    cluster_indices[i] = new_cluster_index as usize;
                }
            }

            if points_moved == 0 && iteration > 0 {
                if self.debug {
                    println!("terminated after {} k-means iterations", iteration);
                }
                break;
            }

            if self.debug {
                println!("iteration {} moved {}", iteration + 1, points_moved);
            }
            let mut component_a_sums = vec![0.0; cluster_count];
            let mut component_b_sums = vec![0.0; cluster_count];
            let mut component_c_sums = vec![0.0; cluster_count];

            for i in 0..cluster_count {
                pixel_count_sums[i] = 0;
            }
            for i in 0..point_count {
                let cluster_index = cluster_indices[i];
                let point = points[i];
                let count = counts[i];
                pixel_count_sums[cluster_index] += count;
                component_a_sums[cluster_index] += point[0] * count as f64;
                component_b_sums[cluster_index] += point[1] * count as f64;
                component_c_sums[cluster_index] += point[2] * count as f64;
            }
            for i in 0..cluster_count {
                let count = pixel_count_sums[i];
                if count == 0 {
                    clusters[i] = [0.0, 0.0, 0.0];
                    continue;
                }
                let a = component_a_sums[i] / count as f64;
                let b = component_b_sums[i] / count as f64;
                let c = component_c_sums[i] / count as f64;
                clusters[i] = [a, b, c];
            }
        }

        let mut cluster_argbs = vec![];
        let mut cluster_populations = vec![];
        for i in 0..cluster_count {
            let count = pixel_count_sums[i];
            if count == 0 {
                continue;
            }

            let possible_new_cluster = self.point_provider.to_int(&clusters[i]);
            if cluster_argbs.contains(&possible_new_cluster) {
                continue;
            }

            cluster_argbs.push(possible_new_cluster);
            cluster_populations.push(count);
        }
        if self.debug {
            println!(
                "kmeans finished and generated {} clusters; {} were requested",
                cluster_argbs.len(),
                cluster_count
            );
        }

        let mut input_pixel_to_cluster_pixel = HashMap::new();
        if self.return_input_pixel_to_cluster_pixel {
            let stopwatch = Instant::now();
            for i in 0..pixels.len() {
                let input_pixel = pixels[i];
                let cluster_index = cluster_indices[i];
                let cluster = clusters[cluster_index];
                let cluster_pixel = self.point_provider.to_int(&cluster);
                input_pixel_to_cluster_pixel.insert(*input_pixel, cluster_pixel);
            }
            if self.debug {
                println!(
                    "took {} ms to create input to cluster map",
                    stopwatch.elapsed().as_millis()
                );
            }
        }

        QuantizerResult {
            color_to_count: zip(cluster_argbs, cluster_populations)
                .map(|(key, value)| (key, value))
                .collect(),
            input_pixel_to_cluster_pixel,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::quantize::{wsmeans::QuantizerWsmeans, Quantizer};

    const RED: u32 = 0xffff0000;
    const GREEN: u32 = 0xff00ff00;
    const BLUE: u32 = 0xff0000ff;
    const MAX_COLORS: u32 = 256;

    #[test]
    fn one_random() {
        let result = QuantizerWsmeans::default().quantize(&vec![0xff141216], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], 0xff141216);
    }

    #[test]
    fn one_red() {
        let result = QuantizerWsmeans::default().quantize(&vec![RED], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], RED);
    }

    #[test]
    fn one_green() {
        let result = QuantizerWsmeans::default().quantize(&vec![GREEN], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], GREEN);
    }

    #[test]
    fn one_blue() {
        let result = QuantizerWsmeans::default().quantize(&vec![BLUE], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], BLUE);
    }

    #[test]
    fn five_blue() {
        let result =
            QuantizerWsmeans::default().quantize(&vec![BLUE, BLUE, BLUE, BLUE, BLUE], MAX_COLORS);
        let colors: Vec<u32> = result.color_to_count.keys().copied().collect();
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0], BLUE);
    }
}
