use rayon::prelude::*;

pub fn parallel_scan_contract(
    f: impl Fn(&usize, &usize) -> usize + Copy + Send + Sync,
    id: usize,
    a: &[usize],
) -> (Vec<usize>, usize) {
    let n = a.len();

    if n == 0 {
        return (Vec::new(), id);
    }
    if n == 1 {
        return (vec![id], a[0]);
    }

    let contracted: Vec<usize> = (0..n / 2)
        .into_par_iter()
        // .into_iter()
        .map(|i| f(&a[2 * i], &a[2 * i + 1]))
        .collect();

    let (partial_results, total) = parallel_scan_contract(f, id, &contracted);

    let result: Vec<usize> = (0..n)
        .into_par_iter()
        // .into_iter()
        .map(|i| {
            if i % 2 == 0 {
                partial_results[i / 2]
            } else {
                f(&partial_results[i / 2], &a[i - 1])
            }
        })
        .collect();

    (result, total)
}

fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}
pub fn parallel_scan(
    f: impl Fn(&usize, &usize) -> usize + Copy + Send + Sync,
    id: usize,
    a: &[usize],
) -> (Vec<usize>, usize) {
    let n = a.len();
    if n == 0 {
        return (Vec::new(), id);
    }

    if !is_power_of_two(n) {
        let next_power = (n as f64).log2().ceil().exp2() as usize;
        // println!("Padding to next power of 2: {} -> {}", n, next_power);
        let mut padded = Vec::with_capacity(next_power);
        padded.extend_from_slice(a);
        padded.extend(std::iter::repeat(id).take(next_power - n));

        let (mut result, total) = parallel_scan_contract(f, id, &padded);
        result.truncate(n);
        return (result, total);
    }
    parallel_scan_contract(f, id, a)
}

// given a[i], return b, where b[i] = sum(a[0:i])
pub fn parallel_scan_add(id: usize, a: &[usize]) -> (Vec<usize>, usize) {
    parallel_scan(|x, y| x + y, id, a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_parallel_scan_add() {
        let input = vec![2, 1, 3, 2, 2, 5, 4, 1];
        let (result, total) = parallel_scan(|x, y| x + y, 0, &input);

        // Expected: [0, 2, 3, 6, 8, 10, 15, 19], 20
        assert_eq!(result, vec![0, 2, 3, 6, 8, 10, 15, 19]);
        assert_eq!(total, 20);
    }

    #[test]
    fn test_parallel_scan_non_power_of_two() {
        let input = vec![1, 2, 3, 4, 5]; // Length 5
        let (result, total) = parallel_scan(|x, y| x + y, 0, &input);
        assert_eq!(result, vec![0, 1, 3, 6, 10]);
        assert_eq!(total, 15);
    }

    #[test]
    fn test_parallel_scan_empty() {
        let input: Vec<usize> = vec![];
        let (result, total) = parallel_scan(|x, y| x + y, 0, &input);
        assert_eq!(result, vec![]);
        assert_eq!(total, 0);
    }

    #[test]
    fn test_parallel_scan_single() {
        let input = vec![5];
        let (result, total) = parallel_scan(|x, y| x + y, 0, &input);
        assert_eq!(result, vec![0]);
        assert_eq!(total, 5);
    }

    #[test]
    fn benchmark_parallel_vs_sequential() {
        // Create a large input
        let large_input: Vec<usize> = (0..1_000_000_00).collect();

        // Time sequential version
        let start = Instant::now();
        let seq_result = large_input
            .iter()
            .scan(0, |state, &x| {
                *state += x;
                Some(*state)
            })
            .collect::<Vec<_>>();
        let seq_time = start.elapsed();

        // Time parallel version
        let start = Instant::now();
        let (par_result, _) = parallel_scan(|x, y| x + y, 0, &large_input);
        let par_time = start.elapsed();

        // Verify results match
        assert_eq!(seq_result[0..large_input.len() - 1], par_result[1..]);

        println!("Sequential time: {:?}", seq_time);
        println!("Parallel time: {:?}", par_time);
    }

    #[test]
    fn benchmark_parallel_vs_sequential_2() {
        let large_input: Vec<usize> = (0..1_000_000_0).collect();
        let large_input2 = large_input.clone();

        let start = Instant::now();
        let seq_result = large_input
            .into_iter()
            .filter(|&x| {
                let mut y = x;
                for _ in 0..100 {
                    y = y.wrapping_mul(y).wrapping_add(x);
                }
                y % 2 == 0
            })
            .collect::<Vec<usize>>();
        let seq_time = start.elapsed();

        let start = Instant::now();
        let par_result = large_input2
            .into_par_iter()
            .filter(|&x| {
                let mut y = x;
                for _ in 0..100 {
                    y = y.wrapping_mul(y).wrapping_add(x);
                }
                y % 2 == 0
            })
            .collect::<Vec<usize>>();
        let par_time = start.elapsed();

        assert_eq!(seq_result, par_result);
        println!("Sequential time: {:?}", seq_time);
        println!("Parallel time: {:?}", par_time);
    }
}
