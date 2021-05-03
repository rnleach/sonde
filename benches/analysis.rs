use criterion::{criterion_group, criterion_main, Criterion};

criterion_main!(benches);
criterion_group!(benches, analysis_bench);

fn analysis_bench(c: &mut Criterion) {
    let data = load_soundings();

    let mut group = c.benchmark_group("Analysis");
    group.sample_size(10);

    for (key, data_vec) in data {
        group.bench_function(key, |b| {
            b.iter_batched(
                || data_vec.clone(),
                |dvec| {
                    for mut anal in dvec {
                        anal.fill_in_missing_analysis_mut();
                    }
                },
                criterion::BatchSize::LargeInput,
            )
        });
    }

    group.finish();
}

fn load_soundings() -> std::collections::HashMap<String, Vec<sonde::Analysis>> {
    use std::collections::HashMap;

    let paths = std::fs::read_dir("bench_data").expect("Error reading bench data directory");

    let mut data = HashMap::new();
    for path in paths {
        let path = path.expect("Error unwrapping DirEntry");

        let key: String = path.file_name().to_string_lossy().to_string();

        if key.contains(".DS_Store") {
            continue;
        }

        let file_type = path.file_type().expect("Error getting file type");
        if file_type.is_dir() {
            let inner_paths =
                std::fs::read_dir(path.path()).expect("Error reading bench data sub-directory");
            let mut inner_data = vec![];
            for inner_path in inner_paths {
                let inner_path = inner_path.expect("Error unwraping DirEntry");
                let file_type = inner_path.file_type().expect("Error getting file type");
                assert!(file_type.is_file());
                let inner_vec = sonde::load_file(&inner_path.path()).expect("Error loading file");
                inner_data.extend(inner_vec);
            }

            let assume_none = data.insert(key, inner_data);
            if assume_none.is_some() {
                panic!("How did we get a file twice?");
            }
        } else {
            let anal_vec = sonde::load_file(&path.path()).expect("Error loading file.");
            let assume_none = data.insert(key, anal_vec);
            if assume_none.is_some() {
                panic!("How did we get a file twice?");
            }
        }
    }

    data
}
