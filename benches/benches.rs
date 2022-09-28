use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use spinout::Atom;
use std::sync::{Arc, Mutex, RwLock};
const UNSORTED_ARR: [i32; 20] = [9, 1, 8, 2, 7, 3, 6, 4, 5, 0, 9, 1, 42, 2, 7, 3, 6, 4, 5, 0];

// Vec's sort optimizes for already sorted arrays and we don't want that here.
fn merge_sort_recurse(numbers: &mut [i32]) {
	let len = numbers.len();
	if len <= 1 {
		return;
	}

	let mid = len / 2;
	let (left, right) = numbers.split_at_mut(mid);

	merge_sort_recurse(left);
	merge_sort_recurse(right);

	let mut tmp = Vec::with_capacity(len);

	let mut left_idx = 0;
	let mut right_idx = 0;

	while left_idx < left.len() && right_idx < right.len() {
		if left[left_idx] < right[right_idx] {
			tmp.push(left[left_idx]);
			left_idx += 1;
		} else {
			tmp.push(right[right_idx]);
			right_idx += 1;
		}
	}

	while left_idx < left.len() {
		tmp.push(left[left_idx]);
		left_idx += 1;
	}

	while right_idx < right.len() {
		tmp.push(right[right_idx]);
		right_idx += 1;
	}

	numbers.copy_from_slice(&tmp);
}

fn merge_sort(numbers: &mut Vec<i32>) {
	merge_sort_recurse(numbers.as_mut_slice());
}

macro_rules ! make_test_rw {
	($name:ident, $tcnt:expr, $modulo:expr, $multiplier:expr) => {
		fn $name(c: &mut Criterion) {
			let name = stringify!($name);
			let mut group = c.benchmark_group(name);
			for i in [1, 2, 4, 8].iter() {
				group.bench_with_input(BenchmarkId::new("ATOM", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							atom_test_rw($tcnt, *i * $multiplier, $modulo);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("MUTEX", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							mutex_test_rw($tcnt, *i * $multiplier, $modulo);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("RWLOCK", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							rwlock_test_rw($tcnt, *i * $multiplier, $modulo);
						})
					})
				});
			}
			group.finish();
		}
	};
}

macro_rules ! make_test_r {
	($name:ident, $tcnt:expr, $multiplier:expr) => {
		fn $name(c: &mut Criterion) {
			let name = stringify!($name);
			let mut group = c.benchmark_group(name);
			for i in [1, 2, 4, 8].iter() {
				group.bench_with_input(BenchmarkId::new("ATOM", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							atom_test_r($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("MUTEX", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							mutex_test_r($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("RWLOCK", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							rwlock_test_r($tcnt, $multiplier * i);
						})
					})
				});
			}
			group.finish();
		}
	};
}

macro_rules ! make_test_w {
	($name:ident, $tcnt:expr, $multiplier:expr) => {
		fn $name(c: &mut Criterion) {
			let name = stringify!($name);
			let mut group = c.benchmark_group(name);
			for i in [1, 2, 4, 8].iter() {
				group.bench_with_input(BenchmarkId::new("ATOM", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							atom_test_w($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("MUTEX", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							mutex_test_w($tcnt, $multiplier * i);
						})
					})
				});

				group.bench_with_input(BenchmarkId::new("RWLOCK", $multiplier * i), &i, |b, _| {
					b.iter(|| {
						black_box({
							rwlock_test_w($tcnt, $multiplier * i);
						})
					})
				});
			}
			group.finish();
		}
	};
}

fn atom_test_rw(tcnt: usize, iters: usize, modulo: usize) {
	let atom = Atom::new(UNSORTED_ARR.to_vec());

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tatom = atom.clone();
		threads.push(std::thread::spawn(move || {
			for i in 0..iters {
				if i % modulo == 0 {
					tatom.lock(|x| {
						merge_sort(x);
						x.reverse();
					});
				}
				tatom.lock(|x| {
					let y = x.get(0);
					match y {
						Some(fortytwo) => { assert_eq!(fortytwo, &42); },
						None => {},
					}
				});
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn atom_test_w(tcnt: usize, iters: usize) {
	let atom = Atom::new(UNSORTED_ARR.to_vec());

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tatom = atom.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				tatom.lock(|x| {
					merge_sort(x);
					x.reverse();
				});
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn atom_test_r(tcnt: usize, iters: usize) {
	let atom = Atom::new(UNSORTED_ARR.to_vec());

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tatom = atom.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				let y = tatom.map(|x| x.iter().find(|x| **x == 42).unwrap().clone());
				assert_eq!(y, 42);
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn rwlock_test_rw(tcnt: usize, iters: usize, modulo: usize) {
	let arc = Arc::new(RwLock::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for i in 0..iters {
				if i % modulo == 0 {
					match tarc.write() {
						Ok(mut x) => {
							merge_sort(&mut x);
							x.reverse();
						},
						Err(_) => panic!("lock failed"),
					}
				}
				match tarc.read() {
					Ok(x) => {
						let y = x.get(0);
						match y {
							Some(fortytwo) => { assert_eq!(fortytwo, &42); },
							None => {},
						}
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn rwlock_test_w(tcnt: usize, iters: usize) {
	let arc = Arc::new(RwLock::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				match tarc.write() {
					Ok(mut x) => {
						merge_sort(&mut x);
						x.reverse();
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn rwlock_test_r(tcnt: usize, iters: usize) {
	let arc = Arc::new(RwLock::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				match tarc.read() {
					Ok(x) => {
						let y = x.iter().find(|x| **x == 42).unwrap().clone();
						assert_eq!(y, 42);
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn mutex_test_rw(tcnt: usize, iters: usize, modulo: usize) {
	let arc = Arc::new(Mutex::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for i in 0..iters {
				if i % modulo == 0 {
					match tarc.lock() {
						Ok(mut x) => {
							merge_sort(&mut x);
							x.reverse();
						},
						Err(_) => panic!("lock failed"),
					}
				}
				match tarc.lock() {
					Ok(x) => {
						let y = x.get(0);
						match y {
							Some(fortytwo) => { assert_eq!(fortytwo, &42); },
							None => {},
						}
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn mutex_test_w(tcnt: usize, iters: usize) {
	let arc = Arc::new(Mutex::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				match tarc.lock() {
					Ok(mut x) => {
						merge_sort(&mut x);
						x.reverse();
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

fn mutex_test_r(tcnt: usize, iters: usize) {
	let arc = Arc::new(Mutex::new(UNSORTED_ARR.to_vec()));

	let mut threads = Vec::new();
	for _ in 0..tcnt {
		let tarc = arc.clone();
		threads.push(std::thread::spawn(move || {
			for _ in 0..iters {
				match tarc.lock() {
					Ok(x) => {
						let y = x.iter().find(|x| **x == 42).unwrap().clone();
						assert_eq!(y, 42);
					},
					Err(_) => panic!("lock failed"),
				}
			}
		}));
	}
	for thread in threads {
		thread.join().unwrap();
	}
}

make_test_rw!(t1_small_balanced_rw, 1, 1, 10);
make_test_rw!(t1_small_read_heavy_rw, 1, 10, 10);
make_test_r!(t1_small_read_only, 1, 10);
make_test_w!(t1_small_write_only, 1, 10);

make_test_rw!(t1_big_balanced_rw, 1, 1, 1000);
make_test_rw!(t1_big_read_heavy_rw, 1, 10, 1000);
make_test_r!(t1_big_read_only, 1, 1000);
make_test_w!(t1_big_write_only, 1, 1000);

make_test_rw!(t4_small_balanced_rw, 4, 1, 10);
make_test_rw!(t4_small_read_heavy_rw, 4, 10, 10);
make_test_r!(t4_small_read_only, 4, 10);
make_test_w!(t4_small_write_only, 4, 10);

make_test_rw!(t4_big_balanced_rw, 4, 1, 1000);
make_test_rw!(t4_big_read_heavy_rw, 4, 10, 1000);
make_test_r!(t4_big_read_only, 4, 1000);
make_test_w!(t4_big_write_only, 4, 1000);

make_test_rw!(t32_big_balanced_rw, 32, 1, 100);
make_test_rw!(t32_big_read_heavy_rw, 32, 10, 100);
make_test_r!(t32_big_read_only, 32, 100);
make_test_w!(t32_big_write_only, 32, 100);

make_test_rw!(t100_big_balanced_rw, 100, 1, 100);
make_test_rw!(t100_big_read_heavy_rw, 100, 10, 100);
make_test_r!(t100_big_read_only, 100, 100);
make_test_w!(t100_big_write_only, 100, 100);

criterion_group!(benches,
	// t1_small_balanced_rw,
	// t1_small_read_heavy_rw,
	// t1_small_read_only,
	// t1_small_write_only,
	// t1_big_balanced_rw,
	// t1_big_read_heavy_rw,
	// t1_big_read_only,
	// t1_big_write_only,
	t4_small_balanced_rw,
	t4_small_read_heavy_rw,
	t4_small_read_only,
	t4_small_write_only,
	t4_big_balanced_rw,
	t4_big_read_heavy_rw,
	t4_big_read_only,
	t4_big_write_only,

	// t32_big_balanced_rw,
	// t32_big_read_heavy_rw,
	// t32_big_read_only,
	// t32_big_write_only,

	// t100_big_balanced_rw,
	// t100_big_read_heavy_rw,
	// t100_big_read_only,
	// t100_big_write_only,
);

criterion_main!(benches);