extern crate libc;
extern crate openblas_src;

use libc::{ c_char, c_int, c_float };

extern "C" {
    pub fn srotg_(a: *mut c_float, b: *mut c_float, c: *mut c_float, s: *mut c_float);
   pub fn sgemm_(
        transa: *const c_char,
        transb: *const c_char,
        m: *const c_int,
        n: *const c_int,
        k: *const c_int,
        alpha: *const c_float,
        a: *const c_float,
        lda: *const c_int,
        b: *const c_float,
        ldb: *const c_int,
        beta: *const c_float,
        c: *mut c_float,
        ldc: *const c_int,
);
}

#[test]
fn link() {
    unsafe {
        let mut a: f32 = 0.0;
        let mut b: f32 = 0.0;
        let mut c: f32 = 42.0;
        let mut d: f32 = 42.0;
        srotg_(
            &mut a as *mut _,
            &mut b as *mut _,
            &mut c as *mut _,
            &mut d as *mut _,
        );
        assert!(c == 1.0);
        assert!(d == 0.0);
    }
}

#[test]
fn sgemm() {
    let a = vec!(1., 2., 3., 4.);
    let b = vec!(1., 0., 0., 0.);
    let mut c = vec!(0., 0., 0., 0.);
    unsafe {
        sgemm_(
            (&b'N') as *const u8 as _,
            (&b'N') as *const u8 as _,
            &2, &2, &2,
            &1.0, a.as_ptr(), &2, b.as_ptr(), &2,
            &1.0, c.as_mut_ptr(), &2);
    }
    assert_eq!(&[1., 2., 0., 0.], &*c);
}

