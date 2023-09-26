#![allow(non_camel_case_types)]
#![deny(missing_docs)]

#[cfg(target_os = "linux")]
use libc::pid_t;
use std::{
    ffi::{c_char, c_float, c_int, c_uchar, c_uint, c_ulong, c_ushort, c_void},
    marker::{PhantomData, PhantomPinned},
};

// === Things which are not part of the main hwloc documentation

/// Rust model of a C incomplete type (struct declaration without a definition)
///
/// From <https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs>
#[repr(C)]
pub(crate) struct IncompleteType {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// Thread identifier (OS-specific)
///
/// This is `HANDLE` on Windows and `libc::pthread_t` on all other platforms.
#[cfg(target_os = "windows")]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub type hwloc_thread_t = windows_sys::Win32::Foundation::HANDLE;

/// Process identifier (OS-specific)
///
/// This is `u32` on Windows and `libc::pid_t` on all other platforms.
#[cfg(target_os = "windows")]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub type hwloc_pid_t = u32;

/// Thread identifier (OS-specific)
///
/// This is `HANDLE` on Windows and `libc::pthread_t` on all other platforms.
#[cfg(not(target_os = "windows"))]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub type hwloc_thread_t = libc::pthread_t;

/// Process identifier (OS-specific)
///
/// This is `u32` on Windows and `libc::pid_t` on all other platforms.
#[cfg(not(target_os = "windows"))]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub type hwloc_pid_t = libc::pid_t;

// === Object Sets: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__object__sets.html

/// A non-modifiable [`hwloc_cpuset_t`]
pub type hwloc_const_cpuset_t = hwloc_const_bitmap_t;

/// A non-modifiable [`hwloc_nodeset_t`]
pub type hwloc_const_nodeset_t = hwloc_const_bitmap_t;

/// A CPU set is a bitmap whose bits are set according to CPU physical OS indexes
///
/// It may be consulted and modified with the bitmap API as any [`hwloc_bitmap_t`].
///
/// Each bit may be converted into a PU object using [`hwloc_get_pu_obj_by_os_index()`].
pub type hwloc_cpuset_t = hwloc_bitmap_t;

/// A node set is a bitmap whose bits are set according to NUMA memory node
/// physical OS indexes
///
/// It may be consulted and modified with the bitmap API as any
/// [`hwloc_bitmap_t`]. Each bit may be converted into a NUMA node object using
/// [`hwloc_get_numanode_obj_by_os_index()`].
///
/// When binding memory on a system without any NUMA node, the single main
/// memory bank is considered as NUMA node `#0`.
///
/// See also [Converting between CPU sets and node
/// sets](https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__helper__nodeset__convert.html).
pub type hwloc_nodeset_t = hwloc_bitmap_t;

// === Object Types: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__object__types.html

/// Value returned by [`hwloc_compare_types()`] when types can not be compared
pub const HWLOC_TYPE_UNORDERED: c_int = c_int::MAX;

/// Type of one side (upstream or downstream) of an I/O bridge
///
/// We can't use Rust enums to model C enums in FFI because that results in
/// undefined behavior if the C API gets new enum variants and sends them to us.
#[doc(alias = "hwloc_obj_bridge_type_e")]
pub type hwloc_obj_bridge_type_t = c_uint;

/// Host-side of a bridge, only possible upstream
pub const HWLOC_OBJ_BRIDGE_HOST: hwloc_obj_bridge_type_t = 0;

/// PCI-side of a bridge
pub const HWLOC_OBJ_BRIDGE_PCI: hwloc_obj_bridge_type_t = 1;

/// Cache type
///
/// We can't use Rust enums to model C enums in FFI because that results in
/// undefined behavior if the C API gets new enum variants and sends them to us.
#[doc(alias = "hwloc_obj_cache_type_e")]
pub type hwloc_obj_cache_type_t = c_uint;

/// Unified cache
pub const HWLOC_OBJ_CACHE_UNIFIED: hwloc_obj_cache_type_t = 0;

/// Data cache
pub const HWLOC_OBJ_CACHE_DATA: hwloc_obj_cache_type_t = 1;

/// Instruction cache (filtered out by default)
pub const HWLOC_OBJ_CACHE_INSTRUCTION: hwloc_obj_cache_type_t = 2;

/// Type of a OS device
///
/// We can't use Rust enums to model C enums in FFI because that results in
/// undefined behavior if the C API gets new enum variants and sends them to us.
#[doc(alias = "hwloc_obj_osdev_type_e")]
pub type hwloc_obj_osdev_type_t = c_uint;

/// Operating system storage device (e.g. block)
///
/// For instance "sda" or "dax2.0" on Linux.
#[doc(alias = "HWLOC_OBJ_OSDEV_BLOCK")]
pub const HWLOC_OBJ_OSDEV_STORAGE: hwloc_obj_osdev_type_t = 0;

/// Operating system GPU device
///
/// For instance ":0.0" for a GL display, "card0" for a Linux DRM device.
pub const HWLOC_OBJ_OSDEV_GPU: hwloc_obj_osdev_type_t = 1;

/// Operating system network device
///
/// For instance the "eth0" interface on Linux.
pub const HWLOC_OBJ_OSDEV_NETWORK: hwloc_obj_osdev_type_t = 2;

/// Operating system openfabrics device
///
/// For instance the "mlx4_0" InfiniBand HCA, "hfi1_0" Omni-Path interface,
/// or "bxi0" Atos/Bull BXI HCA on Linux.
pub const HWLOC_OBJ_OSDEV_OPENFABRICS: hwloc_obj_osdev_type_t = 3;

/// Operating system dma engine device
///
/// For instance the "dma0chan0" DMA channel on Linux.
pub const HWLOC_OBJ_OSDEV_DMA: hwloc_obj_osdev_type_t = 4;

/// Operating system co-processor device
///
/// For instance "opencl0d0" for a OpenCL device, "cuda0" for a CUDA device.
pub const HWLOC_OBJ_OSDEV_COPROC: hwloc_obj_osdev_type_t = 5;

/// Operating system memory device
///
/// For instance DAX file for non-volatile or high-bandwidth memory, like
/// "dax2.0" on Linux.
#[cfg(feature = "hwloc-3_0_0")]
pub const HWLOC_OBJ_OSDEV_MEMORY: hwloc_obj_osdev_type_t = 6;

/// Type of topology object
///
/// We can't use Rust enums to model C enums in FFI because that results in
/// undefined behavior if the C API gets new enum variants and sends them to us.
#[doc(alias = "hwloc_obj_type_e")]
pub type hwloc_obj_type_t = c_uint;

/// The root object, a set of processors and memory with cache coherency
///
/// This type is always used for the root object of a topology, and never
/// used anywhere else. Hence it never has a parent.
pub const HWLOC_OBJ_MACHINE: hwloc_obj_type_t = 0;

/// Physical package, what goes into a physical motherboard socket
///
/// Usually contains multiple cores, and possibly some dies.
pub const HWLOC_OBJ_PACKAGE: hwloc_obj_type_t = 1;

/// A computation unit (may be shared by several PUs aka logical processors)
pub const HWLOC_OBJ_CORE: hwloc_obj_type_t = 2;

/// Processing Unit, or (Logical) Processor
///
/// An execution unit (may share a core with some other logical
/// processors, e.g. in the case of an SMT core).
///
/// This is the leaf of the CPU resource hierarchy, it can only have Misc
/// children.
///
/// It is always reported even when other objects are not detected. However,
/// an incorrect number of PUs may be reported if
/// [`hwloc_topology_discovery_support::pu`] is not set.
pub const HWLOC_OBJ_PU: hwloc_obj_type_t = 3;

/// Level 1 Data (or Unified) Cache
pub const HWLOC_OBJ_L1CACHE: hwloc_obj_type_t = 4;

/// Level 2 Data (or Unified) Cache
pub const HWLOC_OBJ_L2CACHE: hwloc_obj_type_t = 5;

/// Level 3 Data (or Unified) Cache
pub const HWLOC_OBJ_L3CACHE: hwloc_obj_type_t = 6;

/// Level 4 Data (or Unified) Cache
pub const HWLOC_OBJ_L4CACHE: hwloc_obj_type_t = 7;

/// Level 5 Data (or Unified) Cache
// NOTE: If hwloc adds more cache levels, update the hwlocality::cache module accordingly
pub const HWLOC_OBJ_L5CACHE: hwloc_obj_type_t = 8;

/// Level 1 Instruction cache (filtered out by default)
pub const HWLOC_OBJ_L1ICACHE: hwloc_obj_type_t = 9;

/// Level 2 Instruction cache (filtered out by default)
pub const HWLOC_OBJ_L2ICACHE: hwloc_obj_type_t = 10;

/// Level 3 Instruction cache (filtered out by default)
pub const HWLOC_OBJ_L3ICACHE: hwloc_obj_type_t = 11;

/// Group object
///
/// Objects which do not fit in the above but are detected by hwloc and
/// are useful to take into account for affinity. For instance, some
/// operating systems expose their arbitrary processors aggregation this
/// way. And hwloc may insert such objects to group NUMA nodes according
/// to their distances. See also [What are these Group objects in my
/// topology?](https://hwloc.readthedocs.io/en/v2.9/faq.html#faq_groups).
///
/// These objects are ignored when they do not bring any structure (see
/// [`HWLOC_TYPE_FILTER_KEEP_STRUCTURE`])
pub const HWLOC_OBJ_GROUP: hwloc_obj_type_t = 12;

/// NUMA node
///
/// An object that contains memory that is directly and byte-accessible to
/// the host processors. It is usually close to some cores
/// (the corresponding objects are descendants of the NUMA node object in
/// the hwloc tree).
///
/// This is the smallest object representing Memory resources, it cannot
/// have any child except Misc objects. However it may have Memory-side
/// cache parents.
///
/// There is always at least one such object in the topology even if the machine
/// is not NUMA. However, an incorrect number of NUMA nodes may be reported if
/// [`hwloc_topology_discovery_support::numa`] is not set.
///
/// Memory objects are not listed in the main children list, but rather in the
/// dedicated Memory children list. They also have a special depth
/// [`HWLOC_TYPE_DEPTH_NUMANODE`] instead of a normal depth just like other
/// objects in the main tree.
pub const HWLOC_OBJ_NUMANODE: hwloc_obj_type_t = 13;

/// Bridge (filtered out by default)
///
/// Any bridge that connects the host or an I/O bus, to another I/O bus.
///
/// Bridges are not added to the topology unless their filtering is changed
/// (see [`hwloc_topology_set_type_filter()`] and
/// [`hwloc_topology_set_io_types_filter()`]).
///
/// I/O objects are not listed in the main children list, but rather in the
/// dedicated Memory children list. They have NULL CPU and node sets. They
/// also have a special depth [`HWLOC_TYPE_DEPTH_BRIDGE`] instead of a normal
/// depth just like other objects in the main tree.
pub const HWLOC_OBJ_BRIDGE: hwloc_obj_type_t = 14;

/// PCI device (filtered out by default)
///
/// PCI devices are not added to the topology unless their filtering is
/// changed (see [`hwloc_topology_set_type_filter()`] and
/// [`hwloc_topology_set_io_types_filter()`]).
///
/// I/O objects are not listed in the main children list, but rather in the
/// dedicated I/O children list. They have NULL CPU and node sets. They also
/// have a special depth [`HWLOC_TYPE_DEPTH_PCI_DEVICE`] instead of a normal
/// depth just like other objects in the main tree.
pub const HWLOC_OBJ_PCI_DEVICE: hwloc_obj_type_t = 15;

/// Operating system device (filtered out by default)
///
/// OS devices are not added to the topology unless their filtering is
/// changed (see [`hwloc_topology_set_type_filter()`] and
/// [`hwloc_topology_set_io_types_filter()`]).
///
/// I/O objects are not listed in the main children list, but rather in the
/// dedicated I/O children list. They have NULL CPU and node sets. They also
/// have a special depth [`HWLOC_TYPE_DEPTH_OS_DEVICE`] instead of a normal
/// depth just like other objects in the main tree.
pub const HWLOC_OBJ_OS_DEVICE: hwloc_obj_type_t = 16;

/// Miscellaneous object (filtered out by default)
///
/// Objects without particular meaning, that can e.g. be added by the
/// application for its own use, or by hwloc for miscellaneous objects such
/// as MemoryModule (DIMMs).
///
/// They are not added to the topology unless their filtering is
/// changed (see [`hwloc_topology_set_type_filter()`]).
///
/// Misc objects have no CPU and node sets, and may only have other Misc objects
/// as children. They are not part of the main children list, but rather reside
/// in the dedicated Misc children list. They have NULL CPU and node sets.
/// They also have a special depth [`HWLOC_TYPE_DEPTH_MISC`] instead of a normal
/// depth just like other objects in the main tree.
pub const HWLOC_OBJ_MISC: hwloc_obj_type_t = 17;

/// Memory-side cache (filtered out by default)
///
/// A cache in front of a specific NUMA node. This object always has at
/// least one NUMA node as a memory child.
///
/// Memory objects are not listed in the main children list, but rather in
/// the dedicated Memory children list. They also have a special depth
/// [`HWLOC_TYPE_DEPTH_MEMCACHE`] instead of a normal depth just like other
/// objects in the main tree.
#[cfg(feature = "hwloc-2_1_0")]
pub const HWLOC_OBJ_MEMCACHE: hwloc_obj_type_t = 18;

/// Die within a physical package
///
/// A subpart of the physical package, that contains multiple cores.
#[cfg(feature = "hwloc-2_1_0")]
pub const HWLOC_OBJ_DIE: hwloc_obj_type_t = 19;

// === Object Structure and Attributes: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__objects.html

#[repr(C)]
pub struct hwloc_obj {
    /// Type of object
    #[doc(alias = "hwloc_obj::type")]
    ty: hwloc_obj_type_t,

    /// Subtype string to better describe the type field
    ///
    /// See <https://hwloc.readthedocs.io/en/v2.9/attributes.html#attributes_normal>
    /// for a list of subtype strings that hwloc can emit.
    subtype: *mut c_char,

    /// The OS-provided physical index number
    ///
    /// It is not guaranteed unique across the entire machine,
    /// except for PUs and NUMA nodes.
    ///
    /// Set to [`HWLOC_UNKNOWN_INDEX`] if unknown or irrelevant for this object.
    os_index: c_uint,

    /// Object-specific name, if any
    ///
    /// Mostly used for identifying OS devices and Misc objects where a name
    /// string is more useful than numerical indices.
    name: *mut c_char,

    /// Total memory (in bytes) in NUMA nodes below this object
    ///
    /// Requires [`hwloc_topology_discovery_support::numa_memory`].
    total_memory: u64,

    /// Object type-specific attributes, if any
    attr: *mut hwloc_obj_attr_u,

    /// Vertical index in the hierarchy
    ///
    /// For normal objects, this is the depth of the horizontal level that
    /// contains this object and its cousins of the same type. If the topology
    /// is symmetric, this is equal to the parent depth plus one, and also equal
    /// to the number of parent/child links from the root object to here.
    ///
    /// For special objects (NUMA nodes, I/O and Misc) that are not in the main
    /// tree, this is a special value that is unique to their type.
    depth: c_int,

    /// Horizontal index in the whole list of similar objects, hence guaranteed
    /// unique across the entire machine
    ///
    /// Could be a "cousin_rank" since it's the rank within the "cousin" list.
    ///
    /// Note that this index may change when restricting the topology
    /// or when inserting a group.
    logical_index: c_uint,

    /// Next object of same type and depth
    next_cousin: hwloc_obj_t,

    /// Previous object of same type and depth
    prev_cousin: hwloc_obj_t,

    /// Parent object
    ///
    /// Only NULL for the root [`HWLOC_OBJ_MACHINE`] object.
    parent: hwloc_obj_t,

    /// Index in the parent's relevant child list for this object type
    sibling_rank: c_uint,

    /// Next object below the same parent, in the same child list
    next_sibling: hwloc_obj_t,

    /// Previous object below the same parent, in the same child list
    prev_sibling: hwloc_obj_t,

    /// Number of normal children (excluding Memory, Misc and I/O)
    arity: c_uint,

    /// Normal children of this object
    children: *mut hwloc_obj_t,

    /// First normal child of this object
    first_child: hwloc_obj_t,

    /// Last normal child of this object
    last_child: hwloc_obj_t,

    /// Truth that this object is symmetric, which means all normal children and
    /// their children have identical subtrees
    ///
    /// Memory, I/O and Misc children are ignored.
    ///
    /// If this is true of the root object, then the topology may be exported
    /// as a synthetic string.
    symmetric_subtree: c_int,

    /// Number of memory children
    memory_arity: c_uint,

    /// First memory child of this object
    ///
    /// NUMA nodes and Memory-side caches are listed here instead of in the
    /// normal [`children`] list. See also [`hwloc_obj_type_is_memory()`].
    ///
    /// A memory hierarchy starts from a normal CPU-side object (e.g.[`Package`]
    /// (HWLOC_OBJ_PACKAGE)) and ends with NUMA nodes as leaves. There might
    /// exist some memory-side caches between them in the middle of the memory
    /// subtree.
    ///
    /// [`children`]: Self::children
    memory_first_child: hwloc_obj_t,

    /// Number of I/O children
    io_arity: c_uint,

    /// First I/O child of this object
    ///
    /// Bridges, PCI and OS devices are listed here instead of in the normal
    /// [`children`] list. See also [`hwloc_obj_type_is_io()`].
    ///
    /// [`children`]: Self::children
    io_first_child: hwloc_obj_t,

    /// Number of Misc children
    misc_arity: c_uint,

    /// First Misc child of this object
    ///
    /// Misc objects are listed here instead of in the normal [`children`] list.
    ///
    /// [`children`]: Self::children
    misc_first_child: hwloc_obj_t,

    /// CPUs covered by this object
    ///
    /// This is the set of CPUs for which there are PU objects in the
    /// topology under this object, i.e. which are known to be physically
    /// contained in this object and known how (the children path between this
    /// object and the PU objects).
    ///
    /// If the [`HWLOC_TOPOLOGY_FLAG_INCLUDE_DISALLOWED`] topology building
    /// configuration flag is set, some of these CPUs may be online but not
    /// allowed for binding, see [`allowed_cpuset`].
    ///
    /// All objects have CPU and node sets except Misc and I/O objects, so if
    /// you know this object to be a normal or Memory object, you can safely
    /// assume this pointer to be non-NULL.
    ///
    /// [`allowed_cpuset`]: Self::allowed_cpuset
    cpuset: hwloc_cpuset_t,

    /// The complete CPU set of this object
    ///
    /// To the CPUs listed by [`cpuset`], this adds CPUs for which topology
    /// information is unknown or incomplete, some offline CPUs, and CPUs that
    /// are ignored when the [`HWLOC_TOPOLOGY_FLAG_INCLUDE_DISALLOWED`] topology
    /// building configuration flag is not set.
    ///
    /// Thus no corresponding PU object may be found in the topology, because
    /// the precise position is undefined. It is however known that it would be
    /// somewhere under this object.
    ///
    /// [`cpuset`]: Self::cpuset
    complete_cpuset: hwloc_cpuset_t,

    /// NUMA nodes covered by this object or containing this object.
    ///
    /// This is the set of NUMA nodes for which there are NUMA node objects in
    /// the topology under or above this object, i.e. which are known to be
    /// physically contained in this object or containing it and known how
    /// (the children path between this object and the NUMA node objects). In
    /// the end, these nodes are those that are close to the current object.
    ///
    #[cfg_attr(
        feature = "hwloc-2_3_0",
        doc = "With hwloc 2.3+, [`hwloc_get_local_numanode_objs()`] may be used to"
    )]
    #[cfg_attr(feature = "hwloc-2_3_0", doc = "list those NUMA nodes more precisely.")]
    ///
    /// If the [`HWLOC_TOPOLOGY_FLAG_INCLUDE_DISALLOWED`] topology building
    /// configuration flag is set, some of these nodes may not be allowed for
    /// allocation, see [`allowed_nodeset`].
    ///
    /// If there are no NUMA nodes in the machine, all the memory is close to
    /// this object, so the nodeset is full.
    ///
    /// All objects have CPU and node sets except Misc and I/O objects, so if
    /// you know this object to be a normal or Memory object, you can safely
    /// assume this pointer to be non-NULL.
    ///
    /// [`allowed_nodeset`]: Self::allowed_nodeset
    nodeset: hwloc_nodeset_t,

    /// The complete NUMA node set of this object
    ///
    /// To the nodes listed by [`nodeset`], this adds nodes for which topology
    /// information is unknown or incomplete, some offline nodes, and nodes
    /// that are ignored when the
    /// [`HWLOC_TOPOLOGY_FLAG_INCLUDE_DISALLOWED`] topology building
    /// configuration flag is not set.
    ///
    /// Thus no corresponding NUMANode object may be found in the topology,
    /// because the precise position is undefined. It is however known that it
    /// would be somewhere under this object.
    ///
    /// If there are no NUMA nodes in the machine, all the memory is close to
    /// this object, so complete_nodeset is full.
    ///
    /// [`nodeset`]: Self::nodeset
    complete_nodeset: hwloc_nodeset_t,

    /// Complete list of (key, value) textual info pairs
    ///
    /// hwloc defines [a number of standard object info attribute names with
    /// associated semantics](https://hwloc.readthedocs.io/en/v2.9/attributes.html#attributes_info).
    ///
    /// Beware that hwloc allows multiple informations with the same key to
    /// exist, although no sane programs should leverage this possibility.
    infos: *mut hwloc_info_s,

    /// Number of (key, value) pairs in [`infos`]
    ///
    /// [`infos`]: Self::infos
    infos_count: c_uint,

    /// Application-given private data pointer, initialized to NULL, use it as
    /// you wish
    //
    // TODO: Add once support is ready: "See
    // [`hwloc_topology_set_userdata_export_callback()`] if you wish to export
    // this field to XML."
    userdata: *mut c_void,

    /// Global persistent index
    ///
    /// Generated by hwloc, unique across the topology (contrary to
    /// [`os_index`]) and persistent across topology changes (contrary to
    /// [`logical_index`]).
    ///
    /// All this means you can safely use this index as a cheap key representing
    /// the object in a Set or a Map, as long as that Set or Map only refers to
    /// [`hwloc_obj`]s originating from a single [`hwloc_topology`].
    ///
    /// [`logical_index`]: Self::logical_index
    /// [`os_index()`]: Self::os_index
    gp_index: u64,
}

/// Value of [`hwloc_obj::os_index`] when unknown or irrelevant for this object
pub const HWLOC_UNKNOWN_INDEX: c_uint = c_uint::MAX;

/// Convenience typedef, a pointer to a struct [`hwloc_obj`]
pub type hwloc_obj_t = *mut hwloc_obj;

/// [`hwloc_obj_type_t`]-specific attributes
#[derive(Copy, Clone)]
#[repr(C)]
pub union hwloc_obj_attr_u {
    /// [`HWLOC_OBJ_NUMANODE`]-specific attributes
    pub numa: hwloc_numanode_attr_s,

    /// Cache-specific attributes
    pub cache: hwloc_cache_attr_s,

    /// [`HWLOC_OBJ_GROUP`]-specific attributes
    pub group: hwloc_group_attr_s,

    /// [`HWLOC_OBJ_PCI_DEVICE`]-specific attributes
    pub pcidev: hwloc_pcidev_attr_s,

    /// [`HWLOC_OBJ_BRIDGE`]-specific attributes
    pub bridge: hwloc_bridge_attr_s,

    /// [`HWLOC_OBJ_OS_DEVICE`]-specific attributes
    pub osdev: hwloc_osdev_attr_s,
}

/// [`HWLOC_OBJ_NUMANODE`]-specific attributes
#[derive(Copy, Clone, Debug)]
#[doc(alias = "hwloc_obj_attr_u::hwloc_numanode_attr_s")]
#[repr(C)]
pub struct hwloc_numanode_attr_s {
    /// Local memory in bytes
    ///
    /// Requires [`hwloc_topology_discovery_support::numa_memory`].
    #[doc(alias = "hwloc_obj_attr_u::hwloc_numanode_attr_s::local_memory")]
    pub local_memory: u64,

    /// Number of memory page types
    #[doc(alias = "hwloc_obj_attr_u::hwloc_numanode_attr_s::page_types_len")]
    pub page_types_len: c_uint,

    /// Memory page types, sorted by increasing page size
    #[doc(alias = "hwloc_obj_attr_u::hwloc_numanode_attr_s::page_types")]
    pub page_types: *mut hwloc_memory_page_type_s,
}
//
impl Default for hwloc_numanode_attr_s {
    fn default() -> Self {
        Self {
            local_memory: 0,
            page_types_len: 0,
            page_types: std::ptr::null_mut(),
        }
    }
}

/// Local memory page type
#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
#[doc(alias = "hwloc_numanode_attr_s::hwloc_memory_page_type_s")]
#[doc(alias = "hwloc_obj_attr_u::hwloc_numanode_attr_s::hwloc_memory_page_type_s")]
#[repr(C)]
pub struct hwloc_memory_page_type_s {
    /// Size of pages
    #[doc(alias = "hwloc_numanode_attr_s::hwloc_memory_page_type_s::size")]
    #[doc(alias = "hwloc_obj_attr_u::hwloc_numanode_attr_s::hwloc_memory_page_type_s::size")]
    pub size: u64,

    /// Number of pages of this size
    #[doc(alias = "hwloc_numanode_attr_s::hwloc_memory_page_type_s::count")]
    #[doc(alias = "hwloc_obj_attr_u::hwloc_numanode_attr_s::hwloc_memory_page_type_s::count")]
    pub count: u64,
}

/// Cache-specific attributes
#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
#[doc(alias = "hwloc_obj_attr_u::hwloc_cache_attr_s")]
#[repr(C)]
pub struct hwloc_cache_attr_s {
    /// Size of the cache in bytes
    #[doc(alias = "hwloc_obj_attr_u::hwloc_cache_attr_s::size")]
    pub size: u64,

    /// Depth ofthe cache (e.g. L1, L2, ...)
    #[doc(alias = "hwloc_obj_attr_u::hwloc_cache_attr_s::depth")]
    pub depth: c_uint,

    /// Cache line size in bytes
    #[doc(alias = "hwloc_obj_attr_u::hwloc_cache_attr_s::linesize")]
    pub linesize: c_uint,

    /// Ways of associativity, -1 if fully associative, 0 if unknown
    #[doc(alias = "hwloc_obj_attr_u::hwloc_cache_attr_s::associativity")]
    pub associativity: c_int,

    /// Cache type
    #[doc(alias = "hwloc_cache_attr_s::type")]
    #[doc(alias = "hwloc_obj_attr_u::hwloc_cache_attr_s::type")]
    pub ty: hwloc_obj_cache_type_t,
}

/// [`HWLOC_OBJ_GROUP`]-specific attributes
#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
#[doc(alias = "hwloc_obj_attr_u::hwloc_group_attr_s")]
#[repr(C)]
pub struct hwloc_group_attr_s {
    /// Depth of group object
    ///
    /// It may change if intermediate Group objects are added.
    #[doc(alias = "hwloc_obj_attr_u::hwloc_group_attr_s::depth")]
    pub depth: c_uint,

    /// Internally-used kind of group
    #[doc(alias = "hwloc_obj_attr_u::hwloc_group_attr_s::kind")]
    pub kind: c_uint,

    /// Internally-used subkind to distinguish different levels of groups with
    /// the same kind
    #[doc(alias = "hwloc_obj_attr_u::hwloc_group_attr_s::subkind")]
    pub subkind: c_uint,

    /// Flag preventing groups from being automatically merged with identical
    /// parent or children
    #[cfg(feature = "hwloc-2_0_4")]
    #[doc(alias = "hwloc_obj_attr_u::hwloc_group_attr_s::dont_merge")]
    pub dont_merge: c_uchar,
}

/// PCI domain width (depends on hwloc version)
#[cfg(feature = "hwloc-3_0_0")]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub type PCIDomain = u32;

/// PCI domain width (depends on hwloc version)
#[cfg(not(feature = "hwloc-3_0_0"))]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub type PCIDomain = u16;

/// [`HWLOC_OBJ_PCI_DEVICE`]-specific attributes
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s")]
#[repr(C)]
pub struct hwloc_pcidev_attr_s {
    /// PCI domain
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::domain")]
    pub domain: PCIDomain,

    /// PCI bus id
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::bus")]
    pub bus: c_uchar,

    /// PCI bus device
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::dev")]
    pub dev: c_uchar,

    /// PCI function
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::func")]
    pub func: c_uchar,

    /// PCI class ID
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::class_id")]
    pub class_id: c_ushort,

    /// PCI vendor ID
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::vendor_id")]
    pub vendor_id: c_ushort,

    /// PCI device ID
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::device_id")]
    pub device_id: c_ushort,

    /// PCI sub-vendor ID
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::subvendor_id")]
    pub subvendor_id: c_ushort,

    /// PCI sub-device ID
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::subdevice_id")]
    pub subdevice_id: c_ushort,

    /// PCI revision
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::revision")]
    pub revision: c_uchar,

    /// Link speed in GB/s
    #[doc(alias = "hwloc_obj_attr_u::hwloc_pcidev_attr_s::linkspeed")]
    pub linkspeed: c_float,
}

/// [`HWLOC_OBJ_BRIDGE`]-specific attributes
#[derive(Copy, Clone)]
#[doc(alias = "hwloc_obj_attr_u::hwloc_bridge_attr_s")]
#[repr(C)]
pub struct hwloc_bridge_attr_s {
    /// Upstream attributes
    #[doc(alias = "hwloc_bridge_attr_s::upstream")]
    pub upstream: RawUpstreamAttributes,

    /// Upstream type
    #[doc(alias = "hwloc_obj_attr_u::hwloc_bridge_attr_s::upstream_type")]
    pub upstream_type: hwloc_obj_bridge_type_t,

    /// Downstream attributes
    #[doc(alias = "hwloc_obj_attr_u::hwloc_bridge_attr_s::downstream")]
    pub downstream: RawDownstreamAttributes,

    /// Downstream type
    #[doc(alias = "hwloc_obj_attr_u::hwloc_bridge_attr_s::downstream_type")]
    pub downstream_type: hwloc_obj_bridge_type_t,

    /// Bridge depth
    #[doc(alias = "hwloc_obj_attr_u::hwloc_bridge_attr_s::depth")]
    pub depth: c_uint,
}

/// Upstream device attributes
#[derive(Copy, Clone)]
#[repr(C)]
pub union RawUpstreamAttributes {
    /// PCI-specific attributes
    pub pci: hwloc_pcidev_attr_s,
}

/// Downstream PCI device attributes
#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
#[repr(C)]
pub struct RawDownstreamPCIAttributes {
    /// Downstram domain
    pub domain: PCIDomain,

    /// Downstream secondary bus
    pub secondary_bus: c_uchar,

    /// Downstream subordinate bus
    pub subordinate_bus: c_uchar,
}

/// Downstream device attributes
#[derive(Copy, Clone)]
#[repr(C)]
pub union RawDownstreamAttributes {
    /// PCI-specific attributes
    pub pci: RawDownstreamPCIAttributes,
}

/// [`HWLOC_OBJ_OS_DEVICE`]-specific attributes
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[doc(alias = "hwloc_obj_attr_u::hwloc_osdev_attr_s")]
#[repr(C)]
pub struct hwloc_osdev_attr_s {
    /// OS device type
    #[doc(alias = "hwloc_osdev_attr_s::type")]
    #[doc(alias = "hwloc_obj_attr_u::hwloc_osdev_attr_s::type")]
    pub ty: hwloc_obj_osdev_type_t,
}

/// Key-value string attributes
///
/// Used in multiple places of the hwloc API for extensible metadata.
///
/// See also [Consulting and Adding Info
/// Attributes](https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__info__attr.html).
#[repr(C)]
pub struct hwloc_info_s {
    /// Info name
    name: *mut c_char,

    /// Info value
    value: *mut c_char,
}

// === Topology Creation and Destruction: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__creation.html

/// Opaque topology struct
///
/// Models the incomplete type that [`hwloc_topology_t`] API pointers map to.
#[repr(C)]
pub struct hwloc_topology(IncompleteType);

/// Topology context
///
/// To be initialized with [`hwloc_topology_init()`] and built with [`hwloc_topology_load()`].
pub type hwloc_topology_t = *mut hwloc_topology;

/// A non-modifiable [`hwloc_topology_t`]
pub type hwloc_const_topology_t = *const hwloc_topology;

// === Object levels, depths and types: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__levels.html

/// Depth of an object (or object type) in the topology
///
/// We can't use Rust enums to model C enums in FFI because that results in
/// undefined behavior if the C API gets new enum variants and sends them to us.
pub type hwloc_get_type_depth_e = c_int;

/// No object of given type exists in the topology
pub const HWLOC_TYPE_DEPTH_UNKNOWN: hwloc_get_type_depth_e = -1;

/// Objects of given type exist at different depth in the topology (only for Groups)
pub const HWLOC_TYPE_DEPTH_MULTIPLE: hwloc_get_type_depth_e = -2;

/// Virtual depth for [`HWLOC_OBJ_NUMANODE`]
pub const HWLOC_TYPE_DEPTH_NUMANODE: hwloc_get_type_depth_e = -3;

/// Virtual depth for [`HWLOC_OBJ_BRIDGE`]
pub const HWLOC_TYPE_DEPTH_BRIDGE: hwloc_get_type_depth_e = -4;

/// Virtual depth for [`HWLOC_OBJ_PCI_DEVICE`]
pub const HWLOC_TYPE_DEPTH_PCI_DEVICE: hwloc_get_type_depth_e = -5;

/// Virtual depth for [`HWLOC_OS_DEVICE`]
pub const HWLOC_TYPE_DEPTH_OS_DEVICE: hwloc_get_type_depth_e = -6;

/// Virtual depth for [`HWLOC_OBJ_MISC`]
pub const HWLOC_TYPE_DEPTH_MISC: hwloc_get_type_depth_e = -7;

/// Virtual depth for [`HWLOC_OBJ_MEMCACHE`]
#[cfg(feature = "hwloc-2_1_0")]
pub const HWLOC_TYPE_DEPTH_MEMCACHE: hwloc_get_type_depth_e = -8;

// === CPU binding: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__cpubinding.html

/// Process/Thread binding flags
///
/// These bit flags can be used to refine the binding policy. All flags can be
/// OR'ed together with the exception of the binding targets flags
/// [`HWLOC_CPUBIND_THREAD`] and [`HWLOC_CPUBIND_PROCESS`], which are mutually
/// exclusive.
///
/// When using one of the functions that target the active process, you must use
/// at most one of these flags. The most portable binding targets are no flags,
/// which is interpreted as "assume a single-threaded process", followed by
/// [`HWLOC_CPUBIND_THREAD`] and [`HWLOC_CPUBIND_PROCESS`] in this order. These
/// flags must generally not be used with any other function, except on Linux
/// where flag [`HWLOC_CPUBIND_THREAD`] can also be used to turn
/// process-binding functions into thread-binding functions.
///
/// Individual CPU binding functions may not support all of these flags.
/// Please check the documentation of the function that you are
/// trying to call for more information.
pub type hwloc_cpubind_flags_t = c_int;

/// Bind the current thread of the current process
///
/// This is the second most portable option when the process is multi-threaded,
/// and specifying no flags would thus be incorrect.
///
/// On Linux, this flag can also be used to turn process-binding
/// functions into thread-binding functions.
///
/// This is mutually exclusive with [`HWLOC_CPUBIND_PROCESS`].
pub const HWLOC_CPUBIND_THREAD: hwloc_cpubind_flags_t = 1 << 1;

/// Bind all threads of the current process
///
/// This is mutually exclusive with [`HWLOC_CPUBIND_THREAD`].
pub const HWLOC_CPUBIND_PROCESS: hwloc_cpubind_flags_t = 1 << 0;

/// Request for strict binding from the OS
///
/// By default, when the designated CPUs are all busy while other CPUs
/// are idle, operating systems may execute the thread/process on those
/// other CPUs instead of the designated CPUs, to let them progress
/// anyway. Strict binding means that the thread/process will _never_
/// execute on other CPUs than the designated CPUs, even when those are
/// busy with other tasks and other CPUs are idle.
///
/// Depending on the operating system, strict binding may not be
/// possible (e.g. the OS does not implement it) or not allowed (e.g.
/// for an administrative reasons), and the binding function will fail
/// in that case.
///
/// When retrieving the binding of a process, this flag checks whether
/// all its threads actually have the same binding. If the flag is not
/// given, the binding of each thread will be accumulated.
///
/// This flag should not be used when retrieving the binding of a
/// thread or the CPU location of a process.
pub const HWLOC_CPUBIND_STRICT: hwloc_cpubind_flags_t = 1 << 2;

/// Avoid any effect on memory binding
///
/// On some operating systems, some CPU binding function would also bind
/// the memory on the corresponding NUMA node. It is often not a
/// problem for the application, but if it is, setting this flag will
/// make hwloc avoid using OS functions that would also bind memory.
/// This will however reduce the support of CPU bindings, i.e.
/// potentially result in the binding function erroring out.
///
/// This flag should only be used with functions that set the CPU
/// binding.
pub const HWLOC_CPUBIND_NOMEMBIND: hwloc_cpubind_flags_t = 1 << 3;

// === TODO: More sections

// === The bitmap API: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__bitmap.html

/// Opaque bitmap struct
///
/// Represents the private `hwloc_bitmap_s` type that `hwloc_bitmap_t` API
/// pointers map to.
#[repr(C)]
pub struct hwloc_bitmap_s(IncompleteType);

/// Set of bits represented as an opaque pointer to an internal bitmap
pub type hwloc_bitmap_t = *mut hwloc_bitmap_s;

/// A non-modifiable [`hwloc_bitmap_t`]
pub type hwloc_const_bitmap_t = *const hwloc_bitmap_s;

// === TODO: Remaining sections

// === Entry points

// TODO: More docs

macro_rules! extern_c_block {
    ($link_name:literal) => {
        #[link(name = $link_name)]
        extern "C" {
            // === API versioning: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__api__version.html

            /// Indicate at runtime which hwloc API version was used at build time
            ///
            /// This number is updated to `(X<<16)+(Y<<8)+Z` when a new release X.Y.Z
            /// actually modifies the API.
            #[must_use]
            pub fn hwloc_get_api_version() -> c_uint;

            // === Object types: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__object__types.html

            /// Compare the depth of two object types.
            ///
            /// Types shouldn't be compared as they are, since newer ones may be
            /// added in the future.
            ///
            /// # Returns
            ///
            /// - A negative integer if `type1` objects usually include `type2`
            ///   objects.
            /// - A positive integer if `type1` objects are usually included in
            ///   `type2` objects.
            /// - 0 if `type1` and `type2` objects are the same.
            /// - [`HWLOC_TYPE_UNORDERED`] if objects cannot be compared
            ///   (because neither is usually contained in the other).
            ///
            /// # Note
            ///
            /// - Object types containing CPUs can always be compared (usually,
            ///   a machine contains packages, which contain caches, which
            ///   contain cores, which contain PUs).
            /// - [`HWLOC_OBJ_PU`] will always be the deepest, while
            ///   [`HWLOC_OBJ_MACHINE`] is always the highest.
            /// - This does not mean that the actual topology will respect that
            ///   order: e.g. as of today cores may also contain caches, and
            ///   packages may also contain nodes. This is thus just to be seen
            ///   as a fallback comparison method.
            #[must_use]
            pub fn hwloc_compare_types(type1: hwloc_obj_type_t, type2: hwloc_obj_type_t) -> c_int;

            // === Topology creation and destruction: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__creation.html

            /// Allocate a topology context
            ///
            /// # Parameters
            ///
            /// `[out] topologyp` is assigned a pointer to the new allocated
            /// context.
            ///
            /// # Returns
            ///
            /// 0 on success, -1 on error
            #[must_use]
            pub fn hwloc_topology_init(topology: *mut hwloc_topology_t) -> c_int;

            /// Build the actual topology
            ///
            /// Build the actual topology once initialized with
            /// [`hwloc_topology_init()`] and tuned with[Topology Detection
            /// Configuration and
            /// Query](https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__configuration.html)
            /// and [Changing the Source of Topology
            /// Discovery](https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__setsource.html)
            /// routines. No other routine may be called earlier using this
            /// topology context.
            ///
            /// # Parameters
            ///
            /// `topology` is the topology to be loaded with objects.
            ///
            /// # Returns
            ///
            /// 0 on success, -1 on error
            ///
            /// # Note
            ///
            /// - On failure, the topology is reinitialized. It should be either
            ///   destroyed with [`hwloc_topology_destroy()`] or configured and
            ///   loaded again.
            /// - This function may be called only once per topology.
            /// - The binding of the current thread or process may temporarily
            ///   change during this call but it will be restored before it
            ///   returns.
            #[must_use]
            pub fn hwloc_topology_load(topology: hwloc_topology_t) -> c_int;

            /// Terminate and free a topology context
            ///
            /// # Parameters
            ///
            /// `topology` is the topology to be freed
            pub fn hwloc_topology_destroy(topology: hwloc_topology_t);

            /// Duplicate a topology
            ///
            /// The entire topology structure as well as its objects are
            /// duplicated into a new one.
            ///
            /// This is useful for keeping a backup while modifying a topology.
            ///
            /// # Returns
            ///
            /// 0 on success, -1 on error
            ///
            /// # Note
            ///
            /// Object userdata is not duplicated since hwloc does not know what
            /// it points to. The objects of both old and new topologies will
            /// point to the same userdata.
            #[must_use]
            pub fn hwloc_topology_dup(
                newtop: *mut hwloc_topology_t,
                oldtop: hwloc_const_topology_t,
            ) -> c_int;

            /// Check that this topology is compatible with the current hwloc
            /// library
            ///
            /// This is useful when using the same topology structure (in
            /// memory) in different libraries that may use different hwloc
            /// installations (for instance if one library embeds a specific
            /// version of hwloc, while another library uses a default
            /// system-wide hwloc installation).
            ///
            /// If all libraries/programs use the same hwloc installation, this
            /// function always returns 0.
            ///
            /// # Returns
            ///
            /// - `0` on success
            /// - `-1` with errno set to `EINVAL` if incompatible
            //
            // TODO: Propagate note about interprocess sharing from upstream docs
            //       once interprocess sharing is implemented.
            #[must_use]
            pub fn hwloc_topology_abi_check(topology: hwloc_const_topology_t) -> c_int;

            /// Run internal checks on a topology structure
            ///
            /// The program aborts if an inconsistency is detected in the given
            /// topology.
            ///
            /// # Parameters
            ///
            /// `topology` is the topology to be checked
            ///
            /// # Note
            ///
            /// - This routine is only useful to developers.
            /// - The input topology should have been previously loaded with
            ///   [`hwloc_topology_load()`].
            pub fn hwloc_topology_check(topology: hwloc_const_topology_t);

            // === Object levels, depths and types: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__levels.html

            #[must_use]
            pub fn hwloc_topology_get_depth(
                topology: hwloc_const_topology_t,
            ) -> hwloc_get_type_depth_e;
            #[must_use]
            pub fn hwloc_get_type_depth(
                topology: hwloc_const_topology_t,
                object_type: hwloc_obj_type_t,
            ) -> hwloc_get_type_depth_e;
            #[must_use]
            pub fn hwloc_get_memory_parents_depth(
                topology: hwloc_const_topology_t,
            ) -> hwloc_get_type_depth_e;
            #[must_use]
            pub fn hwloc_get_depth_type(
                topology: hwloc_const_topology_t,
                depth: hwloc_get_type_depth_e,
            ) -> hwloc_obj_type_t;
            #[must_use]
            pub fn hwloc_get_nbobjs_by_depth(
                topology: hwloc_const_topology_t,
                depth: hwloc_get_type_depth_e,
            ) -> c_uint;
            #[must_use]
            pub fn hwloc_get_obj_by_depth(
                topology: hwloc_const_topology_t,
                depth: hwloc_get_type_depth_e,
                idx: c_uint,
            ) -> hwloc_obj_t;

            // === Converting between object types, attributes and strings: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__object__strings.html

            #[must_use]
            pub fn hwloc_obj_type_snprintf(
                into: *mut c_char,
                size: usize,
                object: *const hwloc_obj,
                verbose: c_int,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_obj_attr_snprintf(
                into: *mut c_char,
                size: usize,
                object: *const hwloc_obj,
                separator: *const c_char,
                verbose: c_int,
            ) -> c_int;
            // NOTE: Not exposing type printf/scanf for now

            // === Consulting and adding Key-Value info attributes: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__info__attr.html

            #[must_use]
            pub fn hwloc_obj_add_info(
                obj: hwloc_obj_t,
                name: *const c_char,
                value: *const c_char,
            ) -> c_int;

            // === CPU binding: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__cpubinding.html

            #[must_use]
            pub fn hwloc_set_cpubind(
                topology: hwloc_const_topology_t,
                set: hwloc_const_cpuset_t,
                flags: hwloc_cpubind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_get_cpubind(
                topology: hwloc_const_topology_t,
                set: hwloc_cpuset_t,
                flags: hwloc_cpubind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_set_proc_cpubind(
                topology: hwloc_const_topology_t,
                pid: hwloc_pid_t,
                set: hwloc_const_cpuset_t,
                flags: hwloc_cpubind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_get_proc_cpubind(
                topology: hwloc_const_topology_t,
                pid: hwloc_pid_t,
                set: hwloc_cpuset_t,
                flags: hwloc_cpubind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_set_thread_cpubind(
                topology: hwloc_const_topology_t,
                thread: hwloc_thread_t,
                set: hwloc_const_cpuset_t,
                flags: hwloc_cpubind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_get_thread_cpubind(
                topology: hwloc_const_topology_t,
                pid: hwloc_thread_t,
                set: hwloc_cpuset_t,
                flags: hwloc_cpubind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_get_last_cpu_location(
                topology: hwloc_const_topology_t,
                set: hwloc_cpuset_t,
                flags: hwloc_cpubind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_get_proc_last_cpu_location(
                topology: hwloc_const_topology_t,
                pid: hwloc_pid_t,
                set: hwloc_cpuset_t,
                flags: hwloc_cpubind_flags_t,
            ) -> c_int;

            // === Memory binding: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__membinding.html

            #[must_use]
            pub fn hwloc_set_membind(
                topology: hwloc_const_topology_t,
                set: hwloc_const_bitmap_t,
                policy: RawMemoryBindingPolicy,
                flags: hwloc_membind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_get_membind(
                topology: hwloc_const_topology_t,
                set: hwloc_bitmap_t,
                policy: *mut RawMemoryBindingPolicy,
                flags: hwloc_membind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_set_proc_membind(
                topology: hwloc_const_topology_t,
                pid: hwloc_pid_t,
                set: hwloc_const_bitmap_t,
                policy: RawMemoryBindingPolicy,
                flags: hwloc_membind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_get_proc_membind(
                topology: hwloc_const_topology_t,
                pid: hwloc_pid_t,
                set: hwloc_bitmap_t,
                policy: *mut RawMemoryBindingPolicy,
                flags: hwloc_membind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_set_area_membind(
                topology: hwloc_const_topology_t,
                addr: *const c_void,
                len: usize,
                set: hwloc_const_bitmap_t,
                policy: RawMemoryBindingPolicy,
                flags: hwloc_membind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_get_area_membind(
                topology: hwloc_const_topology_t,
                addr: *const c_void,
                len: usize,
                set: hwloc_bitmap_t,
                policy: *mut RawMemoryBindingPolicy,
                flags: hwloc_membind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_get_area_memlocation(
                topology: hwloc_const_topology_t,
                addr: *const c_void,
                len: usize,
                set: hwloc_bitmap_t,
                flags: hwloc_membind_flags_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_alloc(topology: hwloc_const_topology_t, len: usize) -> *mut c_void;
            #[must_use]
            pub fn hwloc_alloc_membind(
                topology: hwloc_const_topology_t,
                len: usize,
                set: hwloc_const_bitmap_t,
                policy: RawMemoryBindingPolicy,
                flags: hwloc_membind_flags_t,
            ) -> *mut c_void;
            #[must_use]
            pub fn hwloc_free(
                topology: hwloc_const_topology_t,
                addr: *mut c_void,
                len: usize,
            ) -> c_int;

            // === Changing the source of topology discovery: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__setsource.html

            #[must_use]
            pub fn hwloc_topology_set_pid(topology: hwloc_topology_t, pid: hwloc_pid_t) -> c_int;
            #[must_use]
            pub fn hwloc_topology_set_synthetic(
                topology: hwloc_topology_t,
                description: *const c_char,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_topology_set_xml(
                topology: hwloc_topology_t,
                xmlpath: *const c_char,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_topology_set_xmlbuffer(
                topology: hwloc_topology_t,
                buffer: *const c_char,
                size: c_int,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_1_0")]
            #[must_use]
            pub fn hwloc_topology_set_components(
                topology: hwloc_topology_t,
                flags: hwloc_topology_components_flag_e,
                name: *const c_char,
            ) -> c_int;

            // === Topology detection configuration and query: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__configuration.html

            #[must_use]
            pub fn hwloc_topology_set_flags(
                topology: hwloc_topology_t,
                flags: hwloc_topology_flags_e,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_topology_get_flags(
                topology: hwloc_const_topology_t,
            ) -> hwloc_topology_flags_e;
            #[must_use]
            pub fn hwloc_topology_is_thissystem(topology: hwloc_const_topology_t) -> c_int;
            #[must_use]
            pub fn hwloc_topology_get_support(
                topology: hwloc_const_topology_t,
            ) -> *const FeatureSupport;
            #[must_use]
            pub fn hwloc_topology_set_type_filter(
                topology: hwloc_topology_t,
                ty: hwloc_obj_type_t,
                filter: RawTypeFilter,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_topology_get_type_filter(
                topology: hwloc_const_topology_t,
                ty: hwloc_obj_type_t,
                filter: *mut RawTypeFilter,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_topology_set_all_types_filter(
                topology: hwloc_topology_t,
                filter: RawTypeFilter,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_topology_set_cache_types_filter(
                topology: hwloc_topology_t,
                filter: RawTypeFilter,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_topology_set_icache_types_filter(
                topology: hwloc_topology_t,
                filter: RawTypeFilter,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_topology_set_io_types_filter(
                topology: hwloc_topology_t,
                filter: RawTypeFilter,
            ) -> c_int;
            // NOTE: set_userdata and get_userdata are NOT exposed because they
            //       are hard to make work with copying, persistence and thread
            //       safety and are not so useful as to justify the effort.

            // === Modifying a loaded Topology: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__tinker.html

            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_topology_restrict(
                topology: hwloc_topology_t,
                set: hwloc_const_bitmap_t,
                flags: hwloc_restrict_flags_e,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_topology_allow(
                topology: hwloc_topology_t,
                cpuset: hwloc_const_cpuset_t,
                nodeset: hwloc_const_nodeset_t,
                flags: hwloc_allow_flags_e,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_topology_insert_misc_object(
                topology: hwloc_topology_t,
                parent: hwloc_obj_t,
                name: *const c_char,
            ) -> hwloc_obj_t;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_topology_alloc_group_object(topology: hwloc_topology_t) -> hwloc_obj_t;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_topology_insert_group_object(
                topology: hwloc_topology_t,
                group: hwloc_obj_t,
            ) -> hwloc_obj_t;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_obj_add_other_obj_sets(dst: hwloc_obj_t, src: *const hwloc_obj) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_topology_refresh(topology: hwloc_topology_t) -> c_int;

            // === Kinds of ObjectTypes: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__helper__types.html

            #[must_use]
            pub fn hwloc_obj_type_is_normal(ty: hwloc_obj_type_t) -> c_int;
            #[must_use]
            pub fn hwloc_obj_type_is_io(ty: hwloc_obj_type_t) -> c_int;
            #[must_use]
            pub fn hwloc_obj_type_is_memory(ty: hwloc_obj_type_t) -> c_int;
            #[must_use]
            pub fn hwloc_obj_type_is_cache(ty: hwloc_obj_type_t) -> c_int;
            #[must_use]
            pub fn hwloc_obj_type_is_dcache(ty: hwloc_obj_type_t) -> c_int;
            #[must_use]
            pub fn hwloc_obj_type_is_icache(ty: hwloc_obj_type_t) -> c_int;

            // === Finding objects, miscellaneous helpers: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__helper__find__misc.html

            #[cfg(feature = "hwloc-2_2_0")]
            #[must_use]
            pub fn hwloc_bitmap_singlify_per_core(
                topology: hwloc_const_topology_t,
                cpuset: hwloc_cpuset_t,
                which: c_uint,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_5_0")]
            #[must_use]
            pub fn hwloc_get_obj_with_same_locality(
                topology: hwloc_const_topology_t,
                src: *const hwloc_obj,
                ty: hwloc_obj_type_t,
                subtype: *const c_char,
                nameprefix: *const c_char,
                flags: c_ulong,
            ) -> *const hwloc_obj;

            // === CPU and node sets of entire topologies: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__helper__topology__sets.html

            #[must_use]
            pub fn hwloc_topology_get_complete_cpuset(
                topology: hwloc_const_topology_t,
            ) -> hwloc_const_cpuset_t;
            #[must_use]
            pub fn hwloc_topology_get_topology_cpuset(
                topology: hwloc_const_topology_t,
            ) -> hwloc_const_cpuset_t;
            #[must_use]
            pub fn hwloc_topology_get_allowed_cpuset(
                topology: hwloc_const_topology_t,
            ) -> hwloc_const_cpuset_t;
            #[must_use]
            pub fn hwloc_topology_get_complete_nodeset(
                topology: hwloc_const_topology_t,
            ) -> hwloc_const_nodeset_t;
            #[must_use]
            pub fn hwloc_topology_get_topology_nodeset(
                topology: hwloc_const_topology_t,
            ) -> hwloc_const_nodeset_t;
            #[must_use]
            pub fn hwloc_topology_get_allowed_nodeset(
                topology: hwloc_const_topology_t,
            ) -> hwloc_const_nodeset_t;

            // === Bitmap API: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__bitmap.html

            #[must_use]
            pub fn hwloc_bitmap_alloc() -> hwloc_bitmap_t;
            #[must_use]
            pub fn hwloc_bitmap_alloc_full() -> hwloc_bitmap_t;
            pub fn hwloc_bitmap_free(bitmap: hwloc_bitmap_t);
            #[must_use]
            pub fn hwloc_bitmap_dup(src: hwloc_const_bitmap_t) -> hwloc_bitmap_t;
            #[must_use]
            pub fn hwloc_bitmap_copy(dst: hwloc_bitmap_t, src: hwloc_const_bitmap_t) -> c_int;

            #[must_use]
            pub fn hwloc_bitmap_list_snprintf(
                buf: *mut c_char,
                len: usize,
                bitmap: hwloc_const_bitmap_t,
            ) -> c_int;
            // NOTE: Not exposing other printfs and scanfs for now

            pub fn hwloc_bitmap_zero(bitmap: hwloc_bitmap_t);
            pub fn hwloc_bitmap_fill(bitmap: hwloc_bitmap_t);
            #[must_use]
            pub fn hwloc_bitmap_only(bitmap: hwloc_bitmap_t, id: c_uint) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_allbut(bitmap: hwloc_bitmap_t, id: c_uint) -> c_int;
            // NOTE: Not exposing ulong-based APIs for now, so no from_ulong, from_ith_ulong, from_ulongs
            //       If I decide to add them, gate from_ulongs with #[cfg(feature = "hwloc-2_1_0")]
            #[must_use]
            pub fn hwloc_bitmap_set(bitmap: hwloc_bitmap_t, id: c_uint) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_set_range(
                bitmap: hwloc_bitmap_t,
                begin: c_uint,
                end: c_int,
            ) -> c_int;
            // NOTE: Not exposing ulong-based APIs for now, so no set_ith_ulong
            #[must_use]
            pub fn hwloc_bitmap_clr(bitmap: hwloc_bitmap_t, id: c_uint) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_clr_range(
                bitmap: hwloc_bitmap_t,
                begin: c_uint,
                end: c_int,
            ) -> c_int;
            pub fn hwloc_bitmap_singlify(bitmap: hwloc_bitmap_t) -> c_int;
            // NOTE: Not exposing ulong-based APIs for now, so no to_ulong, to_ith_ulong, to_ulongs and nr_ulongs
            //       If I decide to add them, gate nr_ulongs and to_ulongs with #[cfg(feature = "hwloc-2_1_0")]

            #[must_use]
            pub fn hwloc_bitmap_isset(bitmap: hwloc_const_bitmap_t, id: c_uint) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_iszero(bitmap: hwloc_const_bitmap_t) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_isfull(bitmap: hwloc_const_bitmap_t) -> c_int;

            #[must_use]
            pub fn hwloc_bitmap_first(bitmap: hwloc_const_bitmap_t) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_next(bitmap: hwloc_const_bitmap_t, prev: c_int) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_last(bitmap: hwloc_const_bitmap_t) -> c_int;

            #[must_use]
            pub fn hwloc_bitmap_weight(bitmap: hwloc_const_bitmap_t) -> c_int;

            #[must_use]
            pub fn hwloc_bitmap_first_unset(bitmap: hwloc_const_bitmap_t) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_next_unset(bitmap: hwloc_const_bitmap_t, prev: c_int) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_last_unset(bitmap: hwloc_const_bitmap_t) -> c_int;

            #[must_use]
            pub fn hwloc_bitmap_or(
                result: hwloc_bitmap_t,
                bitmap1: hwloc_const_bitmap_t,
                bitmap2: hwloc_const_bitmap_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_and(
                result: hwloc_bitmap_t,
                bitmap1: hwloc_const_bitmap_t,
                bitmap2: hwloc_const_bitmap_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_andnot(
                result: hwloc_bitmap_t,
                bitmap1: hwloc_const_bitmap_t,
                bitmap2: hwloc_const_bitmap_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_xor(
                result: hwloc_bitmap_t,
                bitmap1: hwloc_const_bitmap_t,
                bitmap2: hwloc_const_bitmap_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_not(result: hwloc_bitmap_t, bitmap: hwloc_const_bitmap_t) -> c_int;

            #[must_use]
            pub fn hwloc_bitmap_intersects(
                left: hwloc_const_bitmap_t,
                right: hwloc_const_bitmap_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_isincluded(
                left: hwloc_const_bitmap_t,
                right: hwloc_const_bitmap_t,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_bitmap_isequal(
                left: hwloc_const_bitmap_t,
                right: hwloc_const_bitmap_t,
            ) -> c_int;
            // NOTE: Not providing compare_first since it trivially follows from
            //       first_set and seems obscure.
            #[must_use]
            pub fn hwloc_bitmap_compare(
                left: hwloc_const_bitmap_t,
                right: hwloc_const_bitmap_t,
            ) -> c_int;

            // === Exporting Topologies to XML: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__xmlexport.html

            #[must_use]
            pub fn hwloc_topology_export_xml(
                topology: hwloc_const_topology_t,
                xmlpath: *const c_char,
                flags: hwloc_topology_export_xml_flags_e,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_topology_export_xmlbuffer(
                topology: hwloc_const_topology_t,
                xmlbuffer: *mut *mut c_char,
                buflen: *mut c_int,
                flags: hwloc_topology_export_xml_flags_e,
            ) -> c_int;
            pub fn hwloc_free_xmlbuffer(topology: hwloc_const_topology_t, xmlbuffer: *mut c_char);
            // NOTE: Not exposing userdata at the moment, so no need to bind
            //       associated API functions yet.

            // === Exporting Topologies to Synthetic: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__syntheticexport.html

            #[must_use]
            pub fn hwloc_topology_export_synthetic(
                topology: hwloc_const_topology_t,
                buffer: *mut c_char,
                buflen: usize,
                flags: hwloc_topology_export_synthetic_flags_e,
            ) -> c_int;

            // === Retrieve distances between objects: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__distances__get.html

            #[must_use]
            pub fn hwloc_distances_get(
                topology: hwloc_const_topology_t,
                nr: *mut c_uint,
                distances: *mut *mut RawDistances,
                kind: hwloc_distances_kind_e,
                flags: c_ulong,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_distances_get_by_depth(
                topology: hwloc_const_topology_t,
                depth: c_int,
                nr: *mut c_uint,
                distances: *mut *mut RawDistances,
                kind: hwloc_distances_kind_e,
                flags: c_ulong,
            ) -> c_int;
            #[must_use]
            pub fn hwloc_distances_get_by_type(
                topology: hwloc_const_topology_t,
                ty: hwloc_obj_type_t,
                nr: *mut c_uint,
                distances: *mut *mut RawDistances,
                kind: hwloc_distances_kind_e,
                flags: c_ulong,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_1_0")]
            #[must_use]
            pub fn hwloc_distances_get_by_name(
                topology: hwloc_const_topology_t,
                name: *const c_char,
                nr: *mut c_uint,
                distances: *mut *mut RawDistances,
                flags: c_ulong,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_1_0")]
            #[must_use]
            pub fn hwloc_distances_get_name(
                topology: hwloc_const_topology_t,
                distances: *const RawDistances,
            ) -> *const c_char;
            pub fn hwloc_distances_release(
                topology: hwloc_const_topology_t,
                distances: *const RawDistances,
            );
            #[cfg(feature = "hwloc-2_5_0")]
            #[must_use]
            pub fn hwloc_distances_transform(
                topology: hwloc_const_topology_t,
                distances: *mut RawDistances,
                transform: hwloc_distances_transform_e,
                transform_attr: *mut c_void,
                flags: c_ulong,
            ) -> c_int;

            // === Add distances between objects: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__distances__add.html

            #[cfg(feature = "hwloc-2_5_0")]
            #[must_use]
            pub fn hwloc_distances_add_create(
                topology: hwloc_topology_t,
                name: *const c_char,
                kind: hwloc_distances_kind_e,
                flags: c_ulong,
            ) -> DistancesAddHandle;
            #[cfg(feature = "hwloc-2_5_0")]
            #[must_use]
            pub fn hwloc_distances_add_values(
                topology: hwloc_topology_t,
                handle: DistancesAddHandle,
                nbobjs: c_uint,
                objs: *const *const hwloc_obj,
                values: *const u64,
                flags: c_ulong,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_5_0")]
            #[must_use]
            pub fn hwloc_distances_add_commit(
                topology: hwloc_topology_t,
                handle: DistancesAddHandle,
                flags: hwloc_distances_add_flag_e,
            ) -> c_int;

            // === Remove distances between objects: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__distances__remove.html

            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_distances_remove(topology: hwloc_topology_t) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_distances_remove_by_depth(
                topology: hwloc_topology_t,
                depth: hwloc_get_type_depth_e,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_distances_release_remove(
                topology: hwloc_topology_t,
                distances: *mut RawDistances,
            ) -> c_int;

            // === Comparing memory node attributes for finding where to allocate on: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__memattrs.html

            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_memattr_get_by_name(
                topology: hwloc_const_topology_t,
                name: *const c_char,
                id: *mut MemoryAttributeID,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_get_local_numanode_objs(
                topology: hwloc_const_topology_t,
                location: *const RawLocation,
                nr: *mut c_uint,
                nodes: *mut *const hwloc_obj,
                flags: hwloc_local_numanode_flag_e,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_memattr_get_value(
                topology: hwloc_const_topology_t,
                attribute: MemoryAttributeID,
                target_node: *const hwloc_obj,
                initiator: *const RawLocation,
                flags: c_ulong,
                value: *mut u64,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_memattr_get_best_target(
                topology: hwloc_const_topology_t,
                attribute: MemoryAttributeID,
                initiator: *const RawLocation,
                flags: c_ulong,
                best_target: *mut *const hwloc_obj,
                value: *mut u64,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_memattr_get_best_initiator(
                topology: hwloc_const_topology_t,
                attribute: MemoryAttributeID,
                target: *const hwloc_obj,
                flags: c_ulong,
                best_initiator: *mut RawLocation,
                value: *mut u64,
            ) -> c_int;

            // === Managing memory attributes: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__memattrs__manage.html

            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_memattr_get_name(
                topology: hwloc_const_topology_t,
                attribute: MemoryAttributeID,
                name: *mut *const c_char,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_memattr_get_flags(
                topology: hwloc_const_topology_t,
                attribute: MemoryAttributeID,
                flags: *mut hwloc_memattr_flag_e,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_memattr_register(
                topology: hwloc_const_topology_t,
                name: *const c_char,
                flags: hwloc_memattr_flag_e,
                id: *mut MemoryAttributeID,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_memattr_set_value(
                topology: hwloc_const_topology_t,
                attribute: MemoryAttributeID,
                target_node: *const hwloc_obj,
                initiator: *const RawLocation,
                flags: c_ulong,
                value: u64,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_memattr_get_targets(
                topology: hwloc_const_topology_t,
                attribute: MemoryAttributeID,
                initiator: *const RawLocation,
                flags: c_ulong,
                nr: *mut c_uint,
                targets: *mut *const hwloc_obj,
                values: *mut u64,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_3_0")]
            #[must_use]
            pub fn hwloc_memattr_get_initiators(
                topology: hwloc_const_topology_t,
                attribute: MemoryAttributeID,
                target_node: *const hwloc_obj,
                flags: c_ulong,
                nr: *mut c_uint,
                initiators: *mut RawLocation,
                values: *mut u64,
            ) -> c_int;

            // === Kinds of CPU cores: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__cpukinds.html

            #[cfg(feature = "hwloc-2_4_0")]
            #[must_use]
            pub fn hwloc_cpukinds_get_nr(topology: hwloc_const_topology_t, flags: c_ulong)
                -> c_int;
            #[cfg(feature = "hwloc-2_4_0")]
            #[must_use]
            pub fn hwloc_cpukinds_get_by_cpuset(
                topology: hwloc_const_topology_t,
                cpuset: hwloc_const_cpuset_t,
                flags: c_ulong,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_4_0")]
            #[must_use]
            pub fn hwloc_cpukinds_get_info(
                topology: hwloc_const_topology_t,
                kind_index: c_uint,
                cpuset: hwloc_cpuset_t,
                efficiency: *mut c_int,
                nr_infos: *mut c_uint,
                infos: *mut *mut TextualInfo,
                flags: c_ulong,
            ) -> c_int;
            #[cfg(feature = "hwloc-2_4_0")]
            #[must_use]
            pub fn hwloc_cpukinds_register(
                topology: hwloc_topology_t,
                cpuset: hwloc_const_cpuset_t,
                forced_efficiency: c_int,
                nr_infos: c_uint,
                infos: *const TextualInfo,
                flags: c_ulong,
            ) -> c_int;

            // === Linux-specific helpers: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__linux.html

            #[cfg(target_os = "linux")]
            #[must_use]
            pub fn hwloc_linux_set_tid_cpubind(
                topology: hwloc_const_topology_t,
                tid: pid_t,
                set: hwloc_const_cpuset_t,
            ) -> c_int;
            #[cfg(target_os = "linux")]
            #[must_use]
            pub fn hwloc_linux_get_tid_cpubind(
                topology: hwloc_const_topology_t,
                tid: pid_t,
                set: hwloc_cpuset_t,
            ) -> c_int;
            #[cfg(target_os = "linux")]
            #[must_use]
            pub fn hwloc_linux_get_tid_last_cpu_location(
                topology: hwloc_const_topology_t,
                tid: pid_t,
                set: hwloc_cpuset_t,
            ) -> c_int;
            #[cfg(target_os = "linux")]
            #[must_use]
            pub fn hwloc_linux_read_path_as_cpumask(
                path: *const c_char,
                set: hwloc_cpuset_t,
            ) -> c_int;

            // NOTE: libnuma interop is waiting for higher quality libnuma bindings

            // === Windows-specific helpers: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__windows.html

            #[cfg(all(feature = "hwloc-2_5_0", target_os = "windows"))]
            #[must_use]
            pub fn hwloc_windows_get_nr_processor_groups(
                topology: hwloc_const_topology_t,
                flags: c_ulong,
            ) -> c_int;
            #[cfg(all(feature = "hwloc-2_5_0", target_os = "windows"))]
            #[must_use]
            pub fn hwloc_windows_get_processor_group_cpuset(
                topology: hwloc_const_topology_t,
                pg_index: c_uint,
                cpuset: hwloc_cpuset_t,
                flags: c_ulong,
            ) -> c_int;

            // NOTE: glibc interop is waiting for higher quality cpuset support
            //       in the libc crate: right now, it is not possible to safely
            //       crate a `cpu_set_t`, but functions that manipulate them
            //       expect `&mut cpu_set_t`...

            // TODO: Cover more later: interop, differences, sharing, etc...
            //       Beware that primitives that modify the topology should be
            //       exposed in the TopologyEditor, not Topology, because per
            //       hwloc documentation hwloc_topology_refresh() must be called
            //       before multithreaded access is thread-safe again.
        }
    };
}

#[cfg(all(not(feature = "bundled"), target_os = "windows"))]
extern_c_block!("libhwloc");

#[cfg(all(feature = "bundled", target_os = "windows"))]
extern_c_block!("hwloc");

#[cfg(not(target_os = "windows"))]
extern_c_block!("hwloc");
