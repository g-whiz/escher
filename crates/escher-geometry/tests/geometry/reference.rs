use astro_float_num::BigFloat;
use escher_reference::num;

pub(crate) fn epsilon() -> BigFloat {
    num(f64::EPSILON)
}
