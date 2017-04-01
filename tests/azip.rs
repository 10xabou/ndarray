
#[macro_use]
extern crate ndarray;
extern crate itertools;

use ndarray::prelude::*;
use ndarray::Zip;

use itertools::{assert_equal, cloned, enumerate};

use std::mem::swap;


#[test]
fn test_azip1() {
    let mut a = Array::zeros(62);
    let mut x = 0;
    azip!(mut a in { *a = x; x += 1; });
    assert_equal(cloned(&a), 0..a.len());
}

#[test]
fn test_azip2() {
    let mut a = Array::zeros((5, 7));
    let b = Array::from_shape_fn(a.dim(), |(i, j)| 1. / (i + 2*j) as f32);
    azip!(mut a, b in { *a = b; });
    assert_eq!(a, b);
}

#[test]
fn test_azip2_1() {
    let mut a = Array::zeros((5, 7));
    let b = Array::from_shape_fn((5, 10), |(i, j)| 1. / (i + 2*j) as f32);
    let b = b.slice(s![..;-1, 3..]);
    azip!(mut a, b in { *a = b; });
    assert_eq!(a, b);
}

#[test]
fn test_azip2_3() {
    let mut b = Array::from_shape_fn((5, 10), |(i, j)| 1. / (i + 2*j) as f32);
    let mut c = Array::from_shape_fn((5, 10), |(i, j)| f32::exp((i + j) as f32));
    let a = b.clone();
    azip!(mut b, mut c in { swap(b, c) });
    assert_eq!(a, c);
    assert!(a != b);
}

#[test]
fn test_azip2_sum() {
    let c = Array::from_shape_fn((5, 10), |(i, j)| f32::exp((i + j) as f32));
    for i in 0..2 {
        let ax = Axis(i);
        let mut b = Array::zeros(c.len_of(ax));
        azip!(mut b, ref c (c.axis_iter(ax)) in { *b = c.scalar_sum() });
        assert!(b.all_close(&c.sum(Axis(1 - i)), 1e-6));
    }
}

#[test]
fn test_azip3_slices() {
    let mut a = [0.; 32];
    let mut b = [0.; 32];
    let mut c = [0.; 32];
    for (i, elt) in enumerate(&mut b) {
        *elt = i as f32;
    }

    azip!(mut a (&mut a[..]), b (&b[..]), mut c (&mut c[..]) in {
        *a += b / 10.;
        *c = a.sin();
    });
    let res = Array::linspace(0., 3.1, 32).mapv_into(f32::sin);
    assert!(res.all_close(&ArrayView::from(&c), 1e-4));
}

#[test]
fn test_broadcast() {
    let n = 16;
    let mut a = Array::<f32, _>::zeros((n, n));
    let mut b = Array::<f32, _>::from_elem((1, n), 1.);
    for ((i, j), elt) in b.indexed_iter_mut() {
        *elt /= 1. + (i + 2 * j) as f32;
    }
    let d = Array::from_elem((1, n), 1.);
    let e = Array::from_elem((), 2.);

    {
        let mut z = Zip::from(a.view_mut())
            .and_broadcast(&b)
            .and_broadcast(&d)
            .and_broadcast(&e);
        z.apply(|x, &y, &z, &w| *x = y + z + w);
    }
    assert!(a.all_close(&(&b + &d + &e), 1e-4));
}

#[should_panic]
#[test]
fn test_zip_dim_mismatch_1() {
    let mut a = Array::zeros((5, 7));
    let mut d = a.raw_dim();
    d[0] += 1;
    let b = Array::from_shape_fn(d, |(i, j)| 1. / (i + 2*j) as f32);
    azip!(mut a, b in { *a = b; });
}

// Test that Zip handles memory layout correctly for
// Zip::from(A).and(B)
// where A is F-contiguous and B contiguous but neither F nor C contiguous.
#[test]
fn test_contiguous_but_not_c_or_f() {
    let a = Array::from_iter(0..27).into_shape((3, 3, 3)).unwrap();

    // both F order
    let a = a.reversed_axes();
    let mut b = a.clone();
    assert_eq!(a.strides(), b.strides());
    assert_eq!(a.strides(), &[1, 3, 9]);
    b.swap_axes(0, 1);
    // test single elem so that test keeps working if array `+` impl changes
    let correct = &a + &b;
    let correct_012 = a[[0, 1, 2]] + b[[0, 1, 2]];

    let mut ans = Array::zeros(a.dim().f());
    azip!(mut ans, a, b in { *ans = a + b });
    println!("{:?}", a);
    println!("{:?}", b);
    println!("{:?}", ans);

    assert_eq!(ans[[0, 1, 2]], correct_012);
    assert_eq!(ans, correct);
}
