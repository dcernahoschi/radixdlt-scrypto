use std::concat;
use std::str::FromStr;

use criterion::{BenchmarkId, Criterion};
use radix_engine_common::math::PreciseDecimal;

use crate::macros::QUICK;
use crate::{bench_ops, ops_fn, ops_root_fn, process_op};

const ADD_OPERANDS: [(&str, &str); 4] = [
    (
        "170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
        "170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.90834517138450159290932430254268769414059732849732168245030",
    ),
    (
        "-170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
        "170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.90834517138450159290932430254268769414059732849732168245030",
    ),
    ("1", "-1"),
    (
        "-170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
        "-170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.90834517138450159290932430254268769414059732849732168245030",
    )
];

const SUB_OPERANDS: [(&str, &str); 4] = [
    (
        "170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
        "170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.90834517138450159290932430254268769414059732849732168245030",
    ),
    (
        "-170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
        "170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.90834517138450159290932430254268769414059732849732168245030",
    ),
    ("1", "-1"),
    (
        "-170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
        "-170390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.90834517138450159290932430254268769414059732849732168245030",
    )
];

const MUL_OPERANDS: [(&str, &str); 4] = [
    (
        "987230481981237981237192837123797213981273.1231231231231238709238092384",
        "579061686944417527709862483156896256668839859369.854925043439539290804298",
    ),
    (
        "-987230481981237981237192837123797213981273.1231231231231238709238092384",
        "579061686944417527709862483156896256668839859369.854925043439539290804298",
    ),
    (
        "278960446186580977117.854925043439539",
        "278960446186580977117.8549250434395392",
    ),
    ("-123123123123", "-1"),
];

const DIV_OPERANDS: [(&str, &str); 4] = [
    (
        "570390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
        "987230481981237981237192837123797213981273.123123123123123870923809238"
    ),
    (
        "-570390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
        "987230481981237981237192837123797213981273.123123123123123870923809238"
    ),
    ("57896044618658097711785492504343953926634992332820282019728.792003956564819967", "278960446186580977117.8549250434395392"),
    ("-123123123123", "-1"),
];

const ROOT_OPERANDS: [(&str, &str); 4] = [
    (
        "670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
        "30"
    ),
    ("57896044618658097711785492504343953926634992332820282019728.792003956564819967","17"),
    ("12379879872423987.123123123", "13"),
    ("9", "2"),
];

const POW_OPERANDS: [(&str, &str); 4] = [
    ("12.123123123123123123", "15"),
    ("1.123123123", "13"),
    ("4", "5"),
    ("9", "2"),
];

const TO_STRING_OPERANDS: [&str; 4] = [
    "670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
    "57896044618658097711785492504343953926634992332820282019728.792003956564819967",
    "-11237987890123090890328.1928379813",
    "9",
];

const FROM_STRING_OPERANDS: [&str; 4] = [
    "670390396497129854978701249910292306373968291029619668886178072186088201503677348840093714.9083451713845015929093243025426876941405973284973216824503042047",
    "57896044618658097711785492504343953926634992332820282019728.792003956564819967",
    "-11237987890123090890328.1928379813",
    "9",
];

ops_fn!(PreciseDecimal, powi, i64, "clone");
ops_root_fn!(PreciseDecimal, nth_root, "clone");
bench_ops!(PreciseDecimal, "add");
bench_ops!(PreciseDecimal, "sub");
bench_ops!(PreciseDecimal, "mul");
bench_ops!(PreciseDecimal, "div");
bench_ops!(PreciseDecimal, "root", u32);
bench_ops!(PreciseDecimal, "pow", i64);
bench_ops!(PreciseDecimal, "to_string");
bench_ops!(PreciseDecimal, "from_string");
