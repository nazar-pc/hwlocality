# hwlocality: Rust bindings for the hwloc library

[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Package on crates.io](https://img.shields.io/crates/v/hwlocality.svg)](https://crates.io/crates/hwlocality)
[![Documentation](https://docs.rs/hwlocality/badge.svg)](https://docs.rs/hwlocality/)
[![Continuous Integration](https://img.shields.io/github/actions/workflow/status/HadrienG2/hwlocality/ci.yml?branch=master)](https://github.com/HadrienG2/hwlocality/actions?query=workflow%3A%22Continuous+Integration%22)
[![Code coverage](https://codecov.io/gh/HadrienG2/hwlocality/graph/badge.svg?token=OYWLNUD9AI)](https://codecov.io/gh/HadrienG2/hwlocality)
[![CII Best Practices Summary](https://img.shields.io/cii/summary/7876)](https://www.bestpractices.dev/en/projects/7876)
![Requires rustc 1.71.0+](https://img.shields.io/badge/rustc-1.71.0+-lightgray.svg)

## What is this?

To optimize programs for parallel performance on all hardware, you must accept
that on many common platforms,
[symmetric multiprocessing](https://en.wikipedia.org/wiki/Symmetric_multiprocessing)
is a lie. "CPUs" detected by the operating system often have inequal access
to shared resources like caches, DRAM and I/O peripherals, sometimes even
inequal specifications (as in Arm big.LITTLE, Apple Mx and Intel Adler Lake),
and significant performance gains can be achieved by taking these facts into
account in your code.

This is the latest maintained Rust binding to
[hwloc](http://www.open-mpi.org/projects/hwloc), a C library from Open MPI
for detecting the hierarchical topology of modern architectures: NUMA memory
nodes, sockets, shared data & instruction caches, cores, simultaneous multi
threading, and more. Additionally, hwloc lets you pin threads to specific CPU
cores and memory to specific NUMA nodes, which is a prerequisite to perform
topology-aware program optimizations.

`hwlocality` is based on and still shares some code and design with the
previous, now-unmaintained attempts to write Rust hwloc bindings at
[Ichbinjoe/hwloc2-rs](https://github.com/Ichbinjoe/hwloc2-rs) and
[daschl/hwloc-rs](https://github.com/daschl/hwloc-rs). However, it does not aim
for API compatibility with them. Indeed, many changes have been made with
respect to hwloc2-rs in the aim of improving ergonomics, performance, and
removing avenues for Undefined Behaviour like assuming pointers are non-null or
union fields are valid when nobody tells you they will always be.

## Prerequisites

You will need a system with hwloc >=2.0.0 and associated development packages
installed.

By default, compatibility with all hwloc 2.x versions is aimed for, which means
features from newer versions in the 2.x series (or, in the near future,
compatibility with breaking changes from the 3.x series) are not supported by
default.

You can enable them, at the cost of losing compatibility with older
hwloc 2.x releases, by enabling the cargo feature that matches the lowest hwloc
release you need to be compatible with. See [the `[features]` section of this
crate's Cargo.toml](https://github.com/hadrieng2/hwlocality/tree/master/Cargo.toml#L15)
for more information.

Beware that some Linux distributions provide very old hwloc versions. You may
have to install it from [source code](https://www.open-mpi.org/projects/hwloc/).

If you enable the `bundled` Cargo feature, we will attempt to build a recent
hwloc internally. In addition to a valid C build environment, this requires
autotools on Unices and CMake on Windows.

## Usage

First, add `hwlocality` as a dependency:

```bash
cargo add hwlocality
```

Then, inside of your code, set up a
[`Topology`](https://docs.rs/hwlocality/latest/hwlocality/topology/struct.Topology.html).
This is the main entry point to the hwloc library, through which you can access
almost every operation that hwloc allows.

Here is a quick usage example which walks though the detected hardware topology
and prints out a short description of every CPU and cache object known to hwloc:

```rust
use hwlocality::{object::depth::NormalDepth, Topology};

fn main() -> eyre::Result<()> {
    let topology = Topology::new()?;

    for depth in NormalDepth::iter_range(NormalDepth::MIN, topology.depth()) {
        println!("*** Objects at depth {depth}");

        for (idx, object) in topology.objects_at_depth(depth).enumerate() {
            println!("{idx}: {object}");
        }
    }

    Ok(())
}
```

One possible output is:

```text
*** Objects at depth 0
0: Machine
*** Objects at depth 1
0: Package
*** Objects at depth 2
0: L3 (16MB)
*** Objects at depth 3
0: L2 (512KB)
1: L2 (512KB)
2: L2 (512KB)
3: L2 (512KB)
4: L2 (512KB)
5: L2 (512KB)
*** Objects at depth 4
0: L1d (32KB)
1: L1d (32KB)
2: L1d (32KB)
3: L1d (32KB)
4: L1d (32KB)
5: L1d (32KB)
*** Objects at depth 5
0: Core
1: Core
2: Core
3: Core
4: Core
5: Core
*** Objects at depth 6
0: PU
1: PU
2: PU
3: PU
4: PU
5: PU
6: PU
7: PU
8: PU
9: PU
10: PU
11: PU
```

More examples are available [in the source
repository](https://github.com/hadrieng2/hwlocality/tree/master/examples).

## hwloc API coverage

Most of the features from the hwloc 2.x series are now exposed by hwlocality.
But some specialized features, mostly related to interoperability with other
APIs, could not make it for various reasons. [Issues with the "api coverage"
label](https://github.com/HadrienG2/hwlocality/issues?q=is%3Aopen+is%3Aissue+label%3A%22api+coverage%22)
track unimplemented features, and are a great place to look for potential
contributions to this library if you have time!

If you are already familiar with the hwloc C API, you will also be happy to know
that [`#[doc(alias)]` attributes](https://doc.rust-lang.org/rustdoc/advanced-features.html#add-aliases-for-an-item-in-documentation-search)
are extensively used so that you can search the documentation for hwloc API
entities like `hwloc_bitmap_t`, `hwloc_set_cpubind` or `hwloc_obj::arity` and
be redirected to the suggested replacement in the Rust API.

The main exceptions to this rule are notions that are not needed in Rust due to
ergonomics improvements permitted by the Rust type system. For example...

- C-style manual destructors are replaced by Drop impls
- Type argument clarification flags like `HWLOC_MEMBIND_BYNODESET` are replaced
  by generics that do the right thing. 

## License

This project uses the MIT license, please see the
[LICENSE](https://github.com/hadrieng2/hwlocality/blob/master/LICENSE) file for
more information.
