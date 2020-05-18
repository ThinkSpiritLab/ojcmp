use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn bench_normal(c: &mut Criterion, gen: fn(usize) -> (String, String), group_name: &str) {
    use ojcmp::comparers::ByteComparer;
    use std::io::Cursor;

    let mut group = c.benchmark_group(group_name);
    let ns = [
        1, 100, 500, 1000, 5000, 10_000, 50_000, 100_000, 500_000, 1_000_000, 5_000_000,
    ];
    let inputs = ns.iter().map(|&n| (n, gen(n as usize))).collect::<Vec<_>>();

    for &(n, ref input) in inputs.iter() {
        group.throughput(Throughput::Elements(n));
        group.bench_with_input(BenchmarkId::from_parameter(n), input, |b, (s, u)| {
            let comparer = ojcmp::comparers::NormalComparer::new();

            b.iter(|| {
                let mut s_reader = Cursor::new(s.as_bytes());
                let mut u_reader = Cursor::new(u.as_bytes());
                comparer.compare(&mut s_reader, &mut u_reader);
            })
        });
    }
}

fn bench_same(c: &mut Criterion) {
    fn generate_same(n: usize) -> (String, String) {
        use std::fmt::Write;

        let mut s = String::with_capacity(n * 12);
        let mut u = String::with_capacity(n * 12);
        let mut buf = String::with_capacity(16);

        for _ in 0..n {
            let num = rand::random::<i32>();
            write!(buf, "{}\n", num).unwrap();
            s.push_str(buf.as_str());
            u.push_str(buf.as_str());
            buf.clear();
        }

        (s, u)
    }

    bench_normal(c, generate_same, "normal-same")
}

fn bench_endline(c: &mut Criterion) {
    fn generate_endline(n: usize) -> (String, String) {
        use std::fmt::Write;

        let mut s = String::with_capacity(n * 12);
        let mut u = String::with_capacity(n * 12);
        let mut buf = String::with_capacity(16);

        for _ in 0..n {
            let num = rand::random::<i32>();

            write!(buf, "{}\r\n", num).unwrap();
            s.push_str(buf.as_str());
            buf.clear();

            write!(buf, "{}\n", num).unwrap();
            u.push_str(buf.as_str());
            buf.clear();
        }

        (s, u)
    }

    bench_normal(c, generate_endline, "normal-endline")
}

fn bench_space(c: &mut Criterion) {
    fn generate_space(n: usize) -> (String, String) {
        use std::fmt::Write;

        let mut s = String::with_capacity(n * 12);
        let mut u = String::with_capacity(n * 12);
        let mut buf = String::with_capacity(16);

        for _ in 0..n {
            let num = rand::random::<i8>();

            write!(buf, "{} {}\n", num, num).unwrap();
            s.push_str(buf.as_str());
            buf.clear();

            write!(buf, "{}\t{}\n", num, num).unwrap();
            u.push_str(buf.as_str());
            buf.clear();
        }

        (s, u)
    }

    bench_normal(c, generate_space, "normal-space")
}

fn bench_spj_float(c: &mut Criterion) {
    const EPS: f64 = 1e-8;

    fn gen(n: usize) -> (String, String) {
        use std::fmt::Write;

        let mut s = String::with_capacity(n * 20);
        let mut u = String::with_capacity(n * 20);
        let mut buf = String::with_capacity(20);

        for _ in 0..n {
            let num = rand::random::<f64>() * 100.0;
            let diff = rand::random::<f64>() / 1e9;
            let sgn = rand::random::<bool>();

            let rhs = num + if sgn { diff } else { -diff };

            write!(buf, "{}\n", num).unwrap();
            s.push_str(buf.as_str());
            buf.clear();

            write!(buf, "{}\n", rhs).unwrap();
            u.push_str(buf.as_str());
            buf.clear();
        }

        (s, u)
    }

    use ojcmp::compare::Comparison;
    use ojcmp::comparers::ByteComparer;
    use std::io::Cursor;

    let group_name = "spj_float";
    let mut group = c.benchmark_group(group_name);
    let ns = [
        1, 100, 500, 1000, 5000, 10_000, 50_000, 100_000, 500_000, 1_000_000, 5_000_000,
    ];
    let inputs = ns.iter().map(|&n| (n, gen(n as usize))).collect::<Vec<_>>();

    for &(n, ref input) in inputs.iter() {
        group.throughput(Throughput::Elements(n));
        group.bench_with_input(BenchmarkId::from_parameter(n), input, |b, (s, u)| {
            let comparer = ojcmp::comparers::SpjFloatComparer::new(EPS);

            b.iter(|| {
                let mut s_reader = Cursor::new(s.as_bytes());
                let mut u_reader = Cursor::new(u.as_bytes());
                assert_eq!(
                    comparer.compare(&mut s_reader, &mut u_reader),
                    Comparison::AC
                );
            })
        });
    }
}

criterion_group!(spj, bench_spj_float);
criterion_group!(normal, bench_same, bench_endline, bench_space);
criterion_main!(normal, spj);
