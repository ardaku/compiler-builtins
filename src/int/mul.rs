use int::{Int, LargeInt};

macro_rules! mul {
    ($intrinsic:ident: $ty:ty) => {
        /// Returns `a * b`
        #[cfg_attr(not(test), no_mangle)]
        pub extern "C" fn $intrinsic(a: $ty, b: $ty) -> $ty {
            let half_bits = <$ty>::bits() / 4;
            let lower_mask = !0 >> half_bits;
            let mut low = (a.low() & lower_mask) * (b.low() & lower_mask);
            let mut t = low >> half_bits;
            low &= lower_mask;
            t += (a.low() >> half_bits) * (b.low() & lower_mask);
            low += (t & lower_mask) << half_bits;
            let mut high = t >> half_bits;
            t = low >> half_bits;
            low &= lower_mask;
            t += (b.low() >> half_bits) * (a.low() & lower_mask);
            low += (t & lower_mask) << half_bits;
            high += t >> half_bits;
            high += (a.low() >> half_bits) * (b.low() >> half_bits);
            high = high.wrapping_add(a.high().wrapping_mul(b.low()).wrapping_add(a.low().wrapping_mul(b.high())));
            <$ty>::from_parts(low, high)
        }
    }
}

macro_rules! mulo {
    ($intrinsic:ident: $ty:ty) => {
        /// Returns `a * b` and sets `*overflow = 1` if `a * b` overflows
        #[cfg_attr(not(test), no_mangle)]
        pub extern "C" fn $intrinsic(a: $ty, b: $ty, overflow: &mut i32) -> $ty {
            *overflow = 0;
            let result = a.wrapping_mul(b);
            if a == <$ty>::min_value() {
                if b != 0 && b != 1 {
                    *overflow = 1;
                }
                return result;
            }
            if b == <$ty>::min_value() {
                if a != 0 && a != 1 {
                    *overflow = 1;
                }
                return result;
            }

            let sa = a >> (<$ty>::bits() - 1);
            let abs_a = (a ^ sa) - sa;
            let sb = b >> (<$ty>::bits() - 1);
            let abs_b = (b ^ sb) - sb;
            if abs_a < 2 || abs_b < 2 {
                return result;
            }
            if sa == sb {
                if abs_a > <$ty>::max_value() / abs_b {
                    *overflow = 1;
                }
            } else {
                if abs_a > <$ty>::min_value() / -abs_b {
                    *overflow = 1;
                }
            }
            result
        }
    }
}

mul!(__muldi3: u64);
mulo!(__mulosi4: i32);
mulo!(__mulodi4: i64);

#[cfg(test)]
mod tests {
    use gcc_s;
    use qc::{I32, I64, U64};

    quickcheck! {
        fn muldi(a: U64, b: U64) -> bool {
            let (a, b) = (a.0, b.0);
            let r = super::__muldi3(a, b);

            if let Some(muldi3) = gcc_s::muldi3() {
                r == unsafe { muldi3(a, b) }
            } else {
                r == a.wrapping_mul(b)
            }
        }

        fn mulosi(a: I32, b: I32) -> bool {
            let (a, b) = (a.0, b.0);
            let mut overflow = 2;
            let r = super::__mulosi4(a, b, &mut overflow);
            if overflow != 0 && overflow != 1 {
                return false;
            }

            if let Some(mulosi4) = gcc_s::mulosi4() {
                let mut gcc_s_overflow = 2;
                let gcc_s_r = unsafe {
                    mulosi4(a, b, &mut gcc_s_overflow)
                };

                (r, overflow) == (gcc_s_r, gcc_s_overflow)
            } else {
                (r, overflow != 0) == a.overflowing_mul(b)
            }
        }

        fn mulodi(a: I64, b: I64) -> bool {
            let (a, b) = (a.0, b.0);
            let mut overflow = 2;
            let r = super::__mulodi4(a, b, &mut overflow);
            if overflow != 0 && overflow != 1 {
                return false;
            }

            if let Some(mulodi4) = gcc_s::mulodi4() {
                let mut gcc_s_overflow = 2;
                let gcc_s_r = unsafe {
                    mulodi4(a, b, &mut gcc_s_overflow)
                };

                (r, overflow) == (gcc_s_r, gcc_s_overflow)
            } else {
                (r, overflow != 0) == a.overflowing_mul(b)
            }
        }
    }
}
