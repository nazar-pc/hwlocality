//! Memory attributes
//!
//! On some platforms, NUMA nodes do not represent homogeneous memory, but
//! instead reflect multiple tiers of memory with different performance
//! characteristics (HBM, DDR, NVRAM...).
//!
//! hwloc's memory attribute API, which this module is all about, lets you know
//! pick which memory you want to allocate from based on capacity, locality,
//! latency and bandwidth considerations.
//!
//! Most of this module's functionality is exposed via [methods of the Topology
//! struct](../../topology/struct.Topology.html#comparing-memory-node-attributes-for-finding-where-to-allocate-on).
//! The module itself only hosts type definitions that are related to this
//! functionality.

#[cfg(doc)]
use crate::topology::support::DiscoverySupport;
use crate::{
    bitmap::BitmapRef,
    cpu::cpuset::CpuSet,
    errors::{self, FlagsError, ForeignObjectError, HybridError, NulError, RawHwlocError},
    ffi::{
        int,
        string::LibcString,
        transparent::{AsInner, AsNewtype},
    },
    object::TopologyObject,
    topology::{editor::TopologyEditor, Topology},
};
use bitflags::bitflags;
use derive_more::{Display, From};
use errno::Errno;
use hwlocality_sys::{
    hwloc_const_topology_t, hwloc_local_numanode_flag_e, hwloc_location, hwloc_location_u,
    hwloc_memattr_flag_e, hwloc_memattr_id_t, hwloc_obj, HWLOC_LOCAL_NUMANODE_FLAG_ALL,
    HWLOC_LOCAL_NUMANODE_FLAG_LARGER_LOCALITY, HWLOC_LOCAL_NUMANODE_FLAG_SMALLER_LOCALITY,
    HWLOC_LOCATION_TYPE_CPUSET, HWLOC_LOCATION_TYPE_OBJECT, HWLOC_MEMATTR_FLAG_HIGHER_FIRST,
    HWLOC_MEMATTR_FLAG_LOWER_FIRST, HWLOC_MEMATTR_FLAG_NEED_INITIATOR, HWLOC_MEMATTR_ID_BANDWIDTH,
    HWLOC_MEMATTR_ID_CAPACITY, HWLOC_MEMATTR_ID_LATENCY, HWLOC_MEMATTR_ID_LOCALITY,
};
#[cfg(feature = "hwloc-2_8_0")]
use hwlocality_sys::{
    HWLOC_MEMATTR_ID_READ_BANDWIDTH, HWLOC_MEMATTR_ID_READ_LATENCY,
    HWLOC_MEMATTR_ID_WRITE_BANDWIDTH, HWLOC_MEMATTR_ID_WRITE_LATENCY,
};
use libc::{EBUSY, EINVAL, ENOENT};
#[allow(unused)]
#[cfg(test)]
use similar_asserts::assert_eq;
use std::{
    ffi::{c_int, c_uint, c_ulong, CStr},
    hash::Hash,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};
use thiserror::Error;

/// # Comparing memory node attributes for finding where to allocate on
///
/// Platforms with heterogeneous memory require ways to decide whether a buffer
/// should be allocated on "fast" memory (such as HBM), "normal" memory (DDR) or
/// even "slow" but large-capacity memory (non-volatile memory). These memory
/// nodes are called "Targets" while the CPU accessing them is called the
/// "Initiator". Access performance depends on their locality (NUMA platforms)
/// as well as the intrinsic performance of the targets (heterogeneous platforms).
///
/// The following attributes describe the performance of memory accesses from an
/// Initiator to a memory Target, for instance their latency or bandwidth.
/// Initiators performing these memory accesses are usually some PUs or Cores
/// (described as a CPU set). Hence a Core may choose where to allocate a memory
/// buffer by comparing the attributes of different target memory nodes nearby.
///
/// There are also some attributes that are system-wide. Their value does not
/// depend on a specific initiator performing an access. The memory node
/// capacity is an example of such attribute without initiator.
///
/// One way to use this API is to start with a cpuset describing the Cores where
/// a program is bound. The best target NUMA node for allocating memory in this
/// program on these Cores may be obtained by passing this cpuset as an
/// initiator to [`MemoryAttribute::best_target()`] with the relevant
/// memory attribute. For instance, if the code is latency limited, use the
/// Latency attribute.
///
/// A more flexible approach consists in getting the list of local NUMA nodes by
/// passing this cpuset to [`Topology::local_numa_nodes()`]. Attribute values
/// for these nodes, if any, may then be obtained with
/// [`MemoryAttribute::value()`] and manually compared with the desired criteria.
///
/// The API also supports specific objects as initiator, but it is currently not
/// used internally by hwloc. Users may for instance use it to provide custom
/// performance values for host memory accesses performed by GPUs.
/// The interface actually also accepts targets that are not NUMA nodes.
//
// --- Implementation details ---
//
// Upstream docs: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__memattrs.html
impl Topology {
    /// Identifier of the memory attribute with the given name
    ///
    /// Note that a number of predefined memory attributes have hard-coded
    /// identifiers and need not be queried by name at runtime, see the
    /// constructors of [`MemoryAttribute`] for more information.
    ///
    /// # Errors
    ///
    /// - [`NulError`] if `name` contains NUL chars.
    #[doc(alias = "hwloc_memattr_get_by_name")]
    pub fn memory_attribute_named(
        &self,
        name: &str,
    ) -> Result<Option<MemoryAttribute<'_>>, NulError> {
        let name = LibcString::new(name)?;
        let mut id = MaybeUninit::uninit();
        // SAFETY: - Topology is trusted to contain a valid ptr (type invariant)
        //         - name is trusted to be a valid C string (type invariant)
        //         - hwloc ops are trusted not to modify *const parameters
        //         - Per documentation, id is a pure out parameter that hwloc
        //           does not read
        let res = errors::call_hwloc_int_normal("hwloc_memattr_get_by_name", || unsafe {
            hwlocality_sys::hwloc_memattr_get_by_name(self.as_ptr(), name.borrow(), id.as_mut_ptr())
        });
        match res {
            // SAFETY: If hwloc indicates success, it should have initialized id
            //         to a valid memory attribute ID
            Ok(_positive) => Ok(Some(unsafe {
                MemoryAttribute::wrap(self, id.assume_init())
            })),
            Err(RawHwlocError {
                errno: Some(Errno(EINVAL)),
                ..
            }) => Ok(None),
            Err(raw_err) => unreachable!("Unexpected hwloc error: {raw_err}"),
        }
    }

    /// Find local NUMA nodes
    ///
    /// If `target` is given as a [`TopologyObject`], its CPU set is used to
    /// find NUMA nodes with the corresponding locality. If the object does not
    /// have a CPU set (e.g. I/O object), the CPU parent (where the I/O object
    /// is attached) is used.
    ///
    /// Some of these NUMA nodes may not have any memory attribute values and
    /// hence not be reported as actual targets in other functions.
    ///
    /// When an object CPU set is given as locality, for instance a Package, and
    /// when `flags` contains both [`LocalNUMANodeFlags::LARGER_LOCALITY`] and
    /// [`LocalNUMANodeFlags::SMALLER_LOCALITY`], the returned array corresponds
    /// to the nodeset of that object.
    ///
    /// # Errors
    ///
    /// - [`ForeignObjectError`] if `target` refers to a [`TopologyObject`] that
    ///   does not belong to this topology.
    #[allow(clippy::missing_errors_doc)]
    #[doc(alias = "hwloc_get_local_numanode_objs")]
    pub fn local_numa_nodes<'target>(
        &self,
        target: impl Into<TargetNumaNodes<'target>>,
    ) -> Result<Vec<&TopologyObject>, HybridError<ForeignObjectError>> {
        /// Polymorphized version of this function (avoids generics code bloat)
        fn polymorphized<'self_>(
            self_: &'self_ Topology,
            target: TargetNumaNodes<'_>,
        ) -> Result<Vec<&'self_ TopologyObject>, HybridError<ForeignObjectError>> {
            // Prepare to call hwloc
            // SAFETY: Will only be used before returning from this function
            let (location, flags) = unsafe { target.to_checked_raw(self_)? };
            let mut nr = 0;
            let call_ffi = |nr_mut, out_ptr| {
                // SAFETY: - Topology is trusted to contain a valid ptr (type invariant)
                //         - hwloc ops are trusted not to modify *const parameters
                //         - The TargetNumaNodes API is designed in such a way that
                //           an invalid (location, flags) tuple cannot happen.
                //         - nr_mut and out_ptr are call site dependent, see below.
                errors::call_hwloc_int_normal("hwloc_get_local_numanode_objs", || unsafe {
                    hwlocality_sys::hwloc_get_local_numanode_objs(
                        self_.as_ptr(),
                        &location,
                        nr_mut,
                        out_ptr,
                        flags.bits(),
                    )
                })
                .map_err(HybridError::Hwloc)
            };

            // Query node count
            // SAFETY: A null output is allowed, and 0 elements is the correct way
            //         to model it in the "nr" parameter. This will set nr to the
            //         actual buffer size we need to allocate.
            call_ffi(&mut nr, ptr::null_mut())?;
            let len = int::expect_usize(nr);

            // Allocate storage and fill node list
            let mut out = vec![ptr::null(); len];
            let old_nr = nr;
            // SAFETY: out is a valid buffer of size len, which is just nr as usize
            call_ffi(&mut nr, out.as_mut_ptr())?;
            assert_eq!(old_nr, nr, "Inconsistent node count from hwloc");

            // Translate node pointers into node references
            Ok(out
                .into_iter()
                .map(|ptr| {
                    assert!(!ptr.is_null(), "Invalid NUMA node pointer from hwloc");
                    // SAFETY: We trust that if hwloc emits a non-null pointer,
                    //         it is valid, bound to the topology's lifetime,
                    //         and points to a valid target.
                    unsafe { (&*ptr).as_newtype() }
                })
                .collect())
        }
        polymorphized(self, target.into())
    }
}

/// # Managing memory attributes
//
// --- Implementation details ---
//
// Upstream docs: https://hwloc.readthedocs.io/en/v2.9/group__hwlocality__memattrs__manage.html
impl<'topology> TopologyEditor<'topology> {
    /// Register a new memory attribute
    ///
    /// # Errors
    ///
    /// - [`BadFlags`] if `flags` does not contain exactly one of the
    ///   [`HIGHER_IS_BEST`] and [`LOWER_IS_BEST`] flags.
    /// - [`NameContainsNul`] if `name` contains NUL chars.
    /// - [`NameTaken`] if another attribute called `name` already exists.
    ///
    /// [`BadFlags`]: RegisterError::BadFlags
    /// [`HIGHER_IS_BEST`]: [`MemoryAttributeFlags::HIGHER_IS_BEST`]
    /// [`LOWER_IS_BEST`]: [`MemoryAttributeFlags::LOWER_IS_BEST`]
    /// [`NameContainsNul`]: RegisterError::NameContainsNul
    /// [`NameTaken`]: RegisterError::NameTaken
    #[doc(alias = "hwloc_memattr_register")]
    pub fn register_memory_attribute(
        &mut self,
        name: &str,
        flags: MemoryAttributeFlags,
    ) -> Result<MemoryAttributeBuilder<'_, 'topology>, RegisterError> {
        if !flags.is_valid() {
            return Err(flags.into());
        }
        let libc_name = LibcString::new(name)?;
        let mut id = hwloc_memattr_id_t::MAX;
        // SAFETY: - Topology is trusted to contain a valid ptr (type invariant)
        //         - libc_name is trusted to be a valid C string (type
        //           invariant)
        //         - hwloc ops are trusted not to modify *const parameters
        //         - hwloc ops are trusted to keep *mut parameters in a
        //           valid state unless stated otherwise
        //         - flags are validated to be correct
        //         - id is an out-parameter, so it can take any input value
        let res = errors::call_hwloc_int_normal("hwloc_memattr_register", || unsafe {
            hwlocality_sys::hwloc_memattr_register(
                self.topology_mut_ptr(),
                libc_name.borrow(),
                flags.bits(),
                &mut id,
            )
        });
        match res {
            Ok(_positive) => Ok(MemoryAttributeBuilder {
                editor: self,
                flags,
                id,
                name: name.into(),
            }),
            Err(RawHwlocError {
                errno: Some(Errno(EBUSY)),
                ..
            }) => Err(RegisterError::NameTaken(name.into())),
            Err(raw_err) => unreachable!("Unexpected hwloc error: {raw_err}"),
        }
    }
}

/// Error returned when trying to create an memory attribute
#[cfg_attr(windows, allow(variant_size_differences))]
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum RegisterError {
    /// Provided `name` contains NUL chars
    #[error("memory attribute name can't contain NUL chars")]
    NameContainsNul,

    /// Another attribute already uses this name
    #[error("there is already a memory attribute named \"{0}\"")]
    NameTaken(Box<str>),

    /// Specified flags are not correct
    ///
    /// You must specify exactly one of the [`HIGHER_IS_BEST`] and
    /// [`LOWER_IS_BEST`] flags.
    ///
    /// [`HIGHER_IS_BEST`]: [`MemoryAttributeFlags::HIGHER_IS_BEST`]
    /// [`LOWER_IS_BEST`]: [`MemoryAttributeFlags::LOWER_IS_BEST`]
    #[error(transparent)]
    BadFlags(#[from] FlagsError<MemoryAttributeFlags>),
}
//
impl From<NulError> for RegisterError {
    fn from(_: NulError) -> Self {
        Self::NameContainsNul
    }
}
//
impl From<MemoryAttributeFlags> for RegisterError {
    fn from(value: MemoryAttributeFlags) -> Self {
        Self::BadFlags(value.into())
    }
}

/// Mechanism to configure a memory attribute
//
// --- Implementation details ---
//
// # Safety
//
// `id` must be a valid new memory attribute ID from `hwloc_memattr_register()`
#[derive(Debug)]
pub struct MemoryAttributeBuilder<'editor, 'topology> {
    /// Underlying [`TopologyEditor`]
    editor: &'editor mut TopologyEditor<'topology>,

    /// Flags which this memory attribute was registered with
    flags: MemoryAttributeFlags,

    /// ID that `hwloc_memattr_register` allocated to this memory attribute
    id: hwloc_memattr_id_t,

    /// Name of this memory attribute
    name: Box<str>,
}
//
impl MemoryAttributeBuilder<'_, '_> {
    /// Set attribute values for (initiator, target node) pairs
    ///
    /// Given a read-only view of the underlying [`Topology`], the provided
    /// `find_values` callback should conceptually extract a list of
    /// `(initiator, target, value)` tuples if this memory attribute has
    /// initiators (flag [`MemoryAttributeFlags::NEED_INITIATOR`] was set at
    /// attribute registration time), and a list of `(target, value)` tuples
    /// if the attribute has no initiators.
    ///
    /// However, for efficiency reasons, the callback does not literally return
    /// a list of ternary tuples with an optional initiator member, as this
    /// would require one initiator check per attribute value. Instead, the
    /// callback returns a list of `(target, value)` pairs along with an
    /// optional list of initiators. If a list of initiators is provided, it
    /// must be as long as the provided list of `(target, value)` pairs.
    ///
    /// Initiators should be specified as
    /// [`CpuSet`](MemoryAttributeLocation::CpuSet) when referring to accesses
    /// performed by CPU cores. The [`Object`](MemoryAttributeLocation::Object)
    /// initiator type is currently  unused internally by hwloc, but users may
    /// for instance use it to provide custom information about host memory
    /// accesses performed by GPUs.
    ///
    /// # Errors
    ///
    /// - [`InconsistentData`] if the number of provided initiators and
    ///   attribute values does not match
    /// - [`ForeignInitiator`] if some of the provided initiators are
    ///   [`TopologyObject`]s that do not belong to this [`Topology`]
    /// - [`ForeignTarget`] if some of the provided targets are
    ///   [`TopologyObject`]s that do not belong to this [`Topology`]
    /// - [`NeedInitiator`] if initiators were not specified for an attribute
    ///   that requires them
    /// - [`UnwantedInitiator`] if initiators were specified for an attribute
    ///   that does not requires them
    ///
    /// [`InconsistentData`]: ValueInputError::InconsistentData
    /// [`ForeignInitiator`]: InitiatorInputError::ForeignInitiator
    /// [`ForeignTarget`]: ValueInputError::ForeignTarget
    /// [`NeedInitiator`]: InitiatorInputError::NeedInitiator
    /// [`UnwantedInitiator`]: InitiatorInputError::UnwantedInitiator
    #[doc(alias = "hwloc_memattr_set_value")]
    pub fn set_values(
        &mut self,
        find_values: impl FnOnce(
            &Topology,
        ) -> (
            Option<Vec<MemoryAttributeLocation<'_>>>,
            Vec<(&TopologyObject, u64)>,
        ),
    ) -> Result<(), HybridError<ValueInputError>> {
        /// Polymorphized subset of this function (avoids generics code bloat)
        ///
        /// # Safety
        ///
        /// - `initiators` must have just gone through the `to_checked_raw()`
        ///   validation process against this attribute's topology.
        /// - `target_ptrs_and_values` must only contain pointers to objects
        ///   from this memory attribute's topology.
        unsafe fn polymorphized(
            self_: &mut MemoryAttributeBuilder<'_, '_>,
            initiators: Option<Vec<hwloc_location>>,
            target_ptrs_and_values: Vec<(NonNull<hwloc_obj>, u64)>,
        ) -> Result<(), HybridError<ValueInputError>> {
            // Validate initiator and target correctness
            if self_.flags.contains(MemoryAttributeFlags::NEED_INITIATOR) && initiators.is_none() {
                return Err(
                    ValueInputError::BadInitiators(InitiatorInputError::NeedInitiator(
                        self_.name.clone(),
                    ))
                    .into(),
                );
            }
            if !self_.flags.contains(MemoryAttributeFlags::NEED_INITIATOR) && initiators.is_some() {
                return Err(ValueInputError::BadInitiators(
                    InitiatorInputError::UnwantedInitiator(self_.name.clone()),
                )
                .into());
            }
            if let Some(initiators) = &initiators {
                if initiators.len() != target_ptrs_and_values.len() {
                    return Err(ValueInputError::InconsistentData.into());
                }
            }

            // Massage initiators into an iterator of what hwloc wants
            let initiator_ptrs = initiators
                .iter()
                .flatten()
                .map(|initiator_ref| {
                    let initiator_ptr: *const hwloc_location = initiator_ref;
                    initiator_ptr
                })
                .chain(std::iter::repeat_with(ptr::null));

            // Set memory attribute values
            for (initiator_ptr, (target_ptr, value)) in initiator_ptrs.zip(target_ptrs_and_values) {
                // SAFETY: - Topology is trusted to contain a valid ptr (type invariant)
                //         - hwloc ops are trusted to keep *mut parameters in a
                //           valid state unless stated otherwise
                //         - id from hwloc_memattr_register is trusted to be valid
                //         - target_ptr was checked to belong to this topology
                //         - initiator_ptr was checked to belong to this topology
                //         - Flags must be 0 for now
                //         - Any attribute value is valid
                errors::call_hwloc_int_normal("hwloc_memattr_set_value", || unsafe {
                    hwlocality_sys::hwloc_memattr_set_value(
                        self_.editor.topology_mut_ptr(),
                        self_.id,
                        target_ptr.as_ptr(),
                        initiator_ptr,
                        0,
                        value,
                    )
                })
                .map_err(HybridError::Hwloc)?;
            }
            Ok(())
        }

        // Run user callback, post-process results to fit hwloc and borrow
        // checker expectations
        let topology = self.editor.topology();
        let (initiators, targets_and_values) = find_values(topology);
        let initiators = initiators
            .map(|vec| {
                vec.into_iter()
                    // SAFETY: Will only be used before returning from this function
                    .map(|initiator| unsafe {
                        initiator.to_checked_raw(topology).map_err(|e| {
                            ValueInputError::BadInitiators(InitiatorInputError::ForeignInitiator(e))
                        })
                    })
                    .collect::<Result<Vec<_>, ValueInputError>>()
            })
            .transpose()
            .map_err(HybridError::Rust)?;
        let target_ptrs_and_values = targets_and_values
            .into_iter()
            .map(|(target_ref, value)| {
                if !topology.contains(target_ref) {
                    return Err(ValueInputError::ForeignTarget(target_ref.into()));
                }
                let target_ptr = NonNull::from(target_ref).as_inner();
                Ok((target_ptr, value))
            })
            .collect::<Result<Vec<_>, ValueInputError>>()?;

        // Invoke polymorphized subser with the results
        // SAFETY: Initiators and target_ptrs_and_values have been checked to
        //         belong to this memory attribute's topology, and initiators
        //         have not been tampered with.
        unsafe { polymorphized(self, initiators, target_ptrs_and_values) }
    }
}

/// Error returned by [`MemoryAttributeBuilder::set_values`] when the
/// `find_values` callback returns incorrect initiators or targets
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum ValueInputError {
    /// The number of provided initiators does not match the number of attribute values
    #[error("number of memory attribute values doesn't match number of initiators")]
    InconsistentData,

    /// Specified initiators for these attribute values are not valid
    #[error(transparent)]
    BadInitiators(#[from] InitiatorInputError),

    /// Some provided targets are [`TopologyObject`]s that do not belong to
    /// the topology being modified
    #[error("memory attribute target {0}")]
    ForeignTarget(ForeignObjectError),
}

/// Generate [`MemoryAttribute`] constructors around a predefined memory
/// attribute ID from hwloc with minimal boilerplate
///
/// # Safety
///
/// IDs must be ID constants from hwloc
macro_rules! wrap_ids_unchecked {
    (
        $(
            $(#[$attr:meta])*
            $id:ident -> $constructor:ident
        ),*
    ) => {
        $(
            $(#[$attr])*
            pub const fn $constructor(topology: &'topology Topology) -> Self {
                // SAFETY: Per macro precondition
                unsafe { Self::wrap(topology, $id) }
            }
        )*
    };
}

/// Memory attribute
///
/// May be either one of the predefined attributes (see associated const fns)
/// or a new attribute created using
/// [`TopologyEditor::register_memory_attribute()`].
//
// --- Implementation details ---
//
// # Safety
//
// `id` must be a valid identifier to a memory attribute known of `topology`.
#[derive(Copy, Clone, Debug)]
#[doc(alias = "hwloc_memattr_id_e")]
#[doc(alias = "hwloc_memattr_id_t")]
pub struct MemoryAttribute<'topology> {
    /// Topology for which memory attribute is defined
    topology: &'topology Topology,

    /// Identifier of the memory attribute being manipulated
    id: hwloc_memattr_id_t,
}
//
/// # Predefined memory attributes
impl<'topology> MemoryAttribute<'topology> {
    wrap_ids_unchecked!(
        /// Node capacity in bytes (see [`TopologyObject::total_memory()`])
        ///
        /// This attribute involves no initiator.
        ///
        /// Requires [`DiscoverySupport::numa_memory()`].
        #[doc(alias = "HWLOC_MEMATTR_ID_CAPACITY")]
        HWLOC_MEMATTR_ID_CAPACITY -> capacity,

        /// Number of PUs in that locality (i.e. cpuset weight)
        ///
        /// Smaller locality is better. This attribute involves no initiator.
        ///
        /// Requires [`DiscoverySupport::pu_count()`].
        #[doc(alias = "HWLOC_MEMATTR_ID_LOCALITY")]
        HWLOC_MEMATTR_ID_LOCALITY -> locality,

        /// Average bandwidth in MiB/s, as seen from the given initiator location
        ///
        /// This is the average bandwidth for read and write accesses. If the
        /// platform provides individual read and write bandwidths but no
        /// explicit average value, hwloc computes and returns the average.
        #[doc(alias = "HWLOC_MEMATTR_ID_BANDWIDTH")]
        HWLOC_MEMATTR_ID_BANDWIDTH -> bandwidth,

        /// Read bandwidth in MiB/s, as seen from the given initiator location
        #[cfg(feature = "hwloc-2_8_0")]
        #[doc(alias = "HWLOC_MEMATTR_ID_READ_BANDWIDTH")]
        HWLOC_MEMATTR_ID_READ_BANDWIDTH -> read_bandwidth,

        /// Write bandwidth in MiB/s, as seen from the given initiator location
        #[cfg(feature = "hwloc-2_8_0")]
        #[doc(alias = "HWLOC_MEMATTR_ID_WRITE_BANDWIDTH")]
        HWLOC_MEMATTR_ID_WRITE_BANDWIDTH -> write_bandwidth,

        /// Average latency in nanoseconds, as seen from the given initiator location
        ///
        /// This is the average latency for read and write accesses. If the
        /// platform value provides individual read and write latencies but no
        /// explicit average, hwloc computes and returns the average.
        #[doc(alias = "HWLOC_MEMATTR_ID_LATENCY")]
        HWLOC_MEMATTR_ID_LATENCY -> latency,

        /// Read latency in nanoseconds, as seen from the given initiator location
        #[cfg(feature = "hwloc-2_8_0")]
        #[doc(alias = "HWLOC_MEMATTR_ID_READ_LATENCY")]
        HWLOC_MEMATTR_ID_READ_LATENCY -> read_latency,

        /// Write latency in nanoseconds, as seen from the given initiator location
        #[cfg(feature = "hwloc-2_8_0")]
        #[doc(alias = "HWLOC_MEMATTR_ID_WRITE_LATENCY")]
        HWLOC_MEMATTR_ID_WRITE_LATENCY -> write_latency

        // TODO: If you add new attributes, add support to static_flags and
        //       a matching MemoryAttribute constructor below
    );
}
//
/// # Memory attribute API
impl<'topology> MemoryAttribute<'topology> {
    /// Extend an [`hwloc_memattr_id_t`] with topology access to enable method syntax
    ///
    /// # Safety
    ///
    /// `id` must be a valid memory attribute ID, corresponding either to one of
    /// hwloc's predefined attributes or an attribute that was user-allocated
    /// using `hwloc_memattr_register()`.
    pub(crate) const unsafe fn wrap(topology: &'topology Topology, id: hwloc_memattr_id_t) -> Self {
        Self { id, topology }
    }

    /// Name of this memory attribute
    #[doc(alias = "hwloc_memattr_get_name")]
    pub fn name(&self) -> &'topology CStr {
        let mut name = ptr::null();
        // SAFETY: - Topology is trusted to contain a valid ptr (type invariant)
        //         - hwloc ops are trusted not to modify *const parameters
        //         - id is assumed to be valid (type invariant)
        //         - name is an out parameter, its initial value doesn't matter
        let res = errors::call_hwloc_int_normal("hwloc_memattr_get_name", || unsafe {
            hwlocality_sys::hwloc_memattr_get_name(self.topology.as_ptr(), self.id, &mut name)
        });
        match res {
            Ok(_positive) => {}
            Err(RawHwlocError {
                errno: Some(Errno(EINVAL)),
                ..
            }) => unreachable!("MemoryAttribute should only hold valid attribute indices"),
            Err(raw_err) => unreachable!("Unexpected hwloc error: {raw_err}"),
        }
        assert!(
            !name.is_null(),
            "Memory attributes should have non-NULL names"
        );
        // SAFETY: If hwloc does emit a non-NULL name in a successful query run,
        //         we trust that name to be a valid `char*` C string pointer
        //         bound to the topology's lifetime
        unsafe { CStr::from_ptr(name) }
    }

    /// Flags of this memory attribute
    #[doc(alias = "hwloc_memattr_get_flags")]
    pub fn flags(&self) -> MemoryAttributeFlags {
        let flags = Self::static_flags(self.id).unwrap_or_else(|| self.dynamic_flags());
        assert!(flags.is_valid(), "hwloc emitted invalid flags");
        flags
    }

    /// Tell attribute flags if known at compile time
    fn static_flags(id: hwloc_memattr_id_t) -> Option<MemoryAttributeFlags> {
        let bandwidth_flags =
            Some(MemoryAttributeFlags::HIGHER_IS_BEST | MemoryAttributeFlags::NEED_INITIATOR);
        let latency_flags =
            Some(MemoryAttributeFlags::LOWER_IS_BEST | MemoryAttributeFlags::NEED_INITIATOR);
        match id {
            HWLOC_MEMATTR_ID_CAPACITY => Some(MemoryAttributeFlags::HIGHER_IS_BEST),
            HWLOC_MEMATTR_ID_LOCALITY => Some(MemoryAttributeFlags::LOWER_IS_BEST),
            HWLOC_MEMATTR_ID_BANDWIDTH => bandwidth_flags,
            #[cfg(feature = "hwloc-2_8_0")]
            HWLOC_MEMATTR_ID_READ_BANDWIDTH | HWLOC_MEMATTR_ID_WRITE_BANDWIDTH => bandwidth_flags,
            HWLOC_MEMATTR_ID_LATENCY => latency_flags,
            #[cfg(feature = "hwloc-2_8_0")]
            HWLOC_MEMATTR_ID_READ_LATENCY | HWLOC_MEMATTR_ID_WRITE_LATENCY => latency_flags,
            _ => None,
        }
    }

    /// Dynamically query this memory attribute's flags
    fn dynamic_flags(&self) -> MemoryAttributeFlags {
        let mut flags = 0;
        // SAFETY: - Topology is trusted to contain a valid ptr (type invariant)
        //         - hwloc ops are trusted not to modify *const parameters
        //         - id is assumed to be valid (type invariant)
        //         - flags is an out parameter, its initial value doesn't matter
        let res = errors::call_hwloc_int_normal("hwloc_memattr_get_flags", || unsafe {
            hwlocality_sys::hwloc_memattr_get_flags(self.topology.as_ptr(), self.id, &mut flags)
        });
        match res {
            Ok(_positive) => MemoryAttributeFlags::from_bits_truncate(flags),
            Err(RawHwlocError {
                errno: Some(Errno(EINVAL)),
                ..
            }) => unreachable!("MemoryAttribute should only hold valid attribute indices"),
            Err(raw_err) => unreachable!("Unexpected hwloc error: {raw_err}"),
        }
    }

    /// Value of this attribute for a specific initiator and target NUMA node
    ///
    /// `initiator` should be specified if and only if this attribute has the
    /// flag [`MemoryAttributeFlags::NEED_INITIATOR`].
    ///
    /// The initiator should be a [`CpuSet`] when refering to accesses performed
    /// by CPU cores. [`MemoryAttributeLocation::Object`] is currently unused
    /// internally by hwloc, but user-defined memory attributes may for instance
    /// use it to provide custom information about host memory accesses
    /// performed by GPUs.
    ///
    /// # Errors
    ///
    /// - [`ForeignInitiator`] if the `initiator` parameter was set to a
    ///   [`TopologyObject`] that does not belong to this topology
    /// - [`ForeignTarget`] if the `target_node` object does not belong to this
    ///   topology
    /// - [`NeedInitiator`] if no `initiator` was provided but this memory
    ///   attribute needs one
    /// - [`UnwantedInitiator`] if an `initiator` was provided but this memory
    ///   attribute doesn't need one
    ///
    /// [`ForeignInitiator`]: InitiatorInputError::ForeignInitiator
    /// [`ForeignTarget`]: ValueQueryError::ForeignTarget
    /// [`NeedInitiator`]: InitiatorInputError::NeedInitiator
    /// [`UnwantedInitiator`]: InitiatorInputError::UnwantedInitiator
    #[doc(alias = "hwloc_memattr_get_value")]
    pub fn value<'initiator>(
        &self,
        initiator: Option<impl Into<MemoryAttributeLocation<'initiator>>>,
        target_node: &TopologyObject,
    ) -> Result<u64, HybridError<ValueQueryError>> {
        /// Polymorphized version of this function (avoids generics code bloat)
        fn polymorphized(
            self_: &MemoryAttribute<'_>,
            initiator: Option<MemoryAttributeLocation<'_>>,
            target_node: &TopologyObject,
        ) -> Result<u64, HybridError<ValueQueryError>> {
            // Check and translate initiator argument
            // SAFETY: Will only be used before returning from this function
            let initiator = unsafe {
                self_
                    .checked_initiator(initiator, false)
                    .map_err(|err| HybridError::Rust(ValueQueryError::BadInitiator(err)))?
            };

            // Check target argument
            if !self_.topology.contains(target_node) {
                return Err(ValueQueryError::ForeignTarget(target_node.into()).into());
            }

            // Run the query
            let mut value = u64::MAX;
            // SAFETY: - Topology is trusted to contain a valid ptr (type invariant)
            //         - hwloc ops are trusted not to modify *const parameters
            //         - id is trusted to be valid (type invariant)
            //         - target_node has been checked to come from this topology
            //         - initiator has been checked to come from this topology and
            //           to be NULL if and only if the attribute has no initiator
            //         - flags must be 0
            //         - Value is an out parameter, its initial value isn't read
            errors::call_hwloc_int_normal("hwloc_memattr_get_value", || unsafe {
                hwlocality_sys::hwloc_memattr_get_value(
                    self_.topology.as_ptr(),
                    self_.id,
                    target_node.as_inner(),
                    &initiator,
                    0,
                    &mut value,
                )
            })
            .map(|_| value)
            .map_err(HybridError::Hwloc)
        }
        polymorphized(self, initiator.map(Into::into), target_node)
    }

    /// Best target node and associated attribute value, if any, for a given initiator
    ///
    /// The notes on initiator semantics in [`MemoryAttribute::value()`] also
    /// apply to this function.
    ///
    /// If multiple targets have the same attribute values, only one is returned
    /// (and there is no way to clarify how that one is chosen). Applications
    /// that want to detect targets with identical/similar values, or that want
    /// to look at values for multiple attributes, should rather get all values
    /// using [`MemoryAttribute::value()`] and manually select the target they
    /// consider the best.
    ///
    /// # Errors
    ///
    /// - [`ForeignInitiator`] if the `initiator` parameter was set to a
    ///   [`TopologyObject`] that does not belong to this topology
    /// - [`NeedInitiator`] if no `initiator` was provided but this memory
    ///   attribute needs one
    /// - [`UnwantedInitiator`] if an `initiator` was provided but this memory
    ///   attribute doesn't need one
    ///
    /// [`ForeignInitiator`]: InitiatorInputError::ForeignInitiator
    /// [`NeedInitiator`]: InitiatorInputError::NeedInitiator
    /// [`UnwantedInitiator`]: InitiatorInputError::UnwantedInitiator
    #[doc(alias = "hwloc_memattr_get_best_target")]
    pub fn best_target<'initiator>(
        &self,
        initiator: Option<impl Into<MemoryAttributeLocation<'initiator>>>,
    ) -> Result<Option<(&'topology TopologyObject, u64)>, InitiatorInputError> {
        /// Polymorphized version of this function (avoids generics code bloat)
        fn polymorphized<'topology>(
            self_: &MemoryAttribute<'topology>,
            initiator: Option<MemoryAttributeLocation<'_>>,
        ) -> Result<Option<(&'topology TopologyObject, u64)>, InitiatorInputError> {
            // Validate the query
            // SAFETY: Will only be used before returning from this function,
            let initiator = unsafe { self_.checked_initiator(initiator, false)? };

            // Run the query
            let mut best_target = ptr::null();
            // SAFETY: - hwloc_memattr_get_best_target is a "best X" query
            //         - Parameters are forwarded in the right order
            //         - initiator has been checked to come from this topology and
            //           to be NULL if and only if the attribute has no initiator
            //         - best_target is an out parameter, its initial value isn't read
            let opt = unsafe {
                self_.get_best(
                    "hwloc_memattr_get_best_target",
                    |topology, attribute, flags, value| {
                        hwlocality_sys::hwloc_memattr_get_best_target(
                            topology,
                            attribute,
                            &initiator,
                            flags,
                            &mut best_target,
                            value,
                        )
                    },
                )
            };

            // Convert target node into a safe high-level form
            // SAFETY: Target originates from a query against this topology
            Ok(opt.map(|value| (unsafe { self_.encapsulate_target_node(best_target) }, value)))
        }
        polymorphized(self, initiator.map(Into::into))
    }

    /// Best initiator and associated attribute value, if any, for a given target node
    ///
    /// If multiple initiators have the same attribute values, only one is
    /// returned (and there is no way to clarify how that one is chosen).
    /// Applications that want to detect initiators with identical/similar
    /// values, or that want to look at values for multiple attributes, should
    /// rather get all values using [`MemoryAttribute::value()`] and manually
    /// select the initiator they consider the best.
    ///
    /// # Errors
    ///
    /// - [`NoInitiators`] if this memory attribute doesn't have initiators
    /// - [`ForeignTarget`] if `target` does not belong to this topology
    ///
    /// [`ForeignTarget`]: InitiatorQueryError::ForeignTarget
    /// [`NoInitiators`]: InitiatorQueryError::NoInitiators
    #[doc(alias = "hwloc_memattr_get_best_initiator")]
    pub fn best_initiator(
        &self,
        target: &TopologyObject,
    ) -> Result<Option<(MemoryAttributeLocation<'topology>, u64)>, InitiatorQueryError> {
        // Validate the query
        self.check_initiator_query_target(target)?;

        // Run the query
        // SAFETY: This is an out parameter, initial value won't be read
        let mut best_initiator = NULL_LOCATION;
        // SAFETY: - hwloc_memattr_get_best_initiator is a "best X" query
        //         - Parameters are forwarded in the right order
        //         - target node has been checked to come from this topology
        //         - best_initiator is an out parameter, its initial value isn't read
        let opt = unsafe {
            self.get_best(
                "hwloc_memattr_get_best_initiator",
                |topology, attribute, flags, value| {
                    hwlocality_sys::hwloc_memattr_get_best_initiator(
                        topology,
                        attribute,
                        target.as_inner(),
                        flags,
                        &mut best_initiator,
                        value,
                    )
                },
            )
        };

        // Convert initiator into a safe high-level form
        // SAFETY: Initiator originates from a query against this topology
        opt.map(|value| Ok((unsafe { self.encapsulate_initiator(best_initiator) }, value)))
            .transpose()
    }

    /// Target NUMA nodes that have some values for a given attribute, along
    /// with the associated values.
    ///
    /// An `initiator` may only be specified if this attribute has the flag
    /// [`MemoryAttributeFlags::NEED_INITIATOR`]. In that case, it acts as a
    /// filter to only report targets that have a value for this initiator.
    ///
    /// The initiator should be a [`CpuSet`] when refering to accesses performed
    /// by CPU cores. [`MemoryAttributeLocation::Object`] is currently unused
    /// internally by hwloc, but user-defined memory attributes may for instance
    /// use it to provide custom information about host memory accesses
    /// performed by GPUs.
    ///
    /// This function is meant for tools and debugging (listing internal
    /// information) rather than for application queries. Applications should
    /// rather select useful NUMA nodes with [`Topology::local_numa_nodes()`]
    /// and then look at their attribute values.
    ///
    /// # Errors
    ///
    /// - [`ForeignInitiator`] if the `initiator` parameter was set to a
    ///   [`TopologyObject`] that does not belong to this topology
    /// - [`NeedInitiator`] if no `initiator` was provided but this memory
    ///   attribute needs one
    /// - [`UnwantedInitiator`] if an `initiator` was provided but this memory
    ///   attribute doesn't need one
    ///
    /// [`ForeignInitiator`]: InitiatorInputError::ForeignInitiator
    /// [`NeedInitiator`]: InitiatorInputError::NeedInitiator
    /// [`UnwantedInitiator`]: InitiatorInputError::UnwantedInitiator
    #[doc(alias = "hwloc_memattr_get_targets")]
    pub fn targets<'initiator>(
        &self,
        initiator: Option<impl Into<MemoryAttributeLocation<'initiator>>>,
    ) -> Result<(Vec<&'topology TopologyObject>, Vec<u64>), HybridError<InitiatorInputError>> {
        /// Polymorphized version of this function (avoids generics code bloat)
        fn polymorphized<'topology>(
            self_: &MemoryAttribute<'topology>,
            initiator: Option<MemoryAttributeLocation<'_>>,
        ) -> Result<(Vec<&'topology TopologyObject>, Vec<u64>), HybridError<InitiatorInputError>>
        {
            // Validate the query + translate initiator to hwloc format
            // SAFETY: - Will only be used before returning from this function,
            //         - get_targets is documented to accept a NULL initiator
            let initiator = unsafe { self_.checked_initiator(initiator, true)? };

            // Run the query
            // SAFETY: - hwloc_memattr_get_targets is indeed an array query
            //         - Parameters are forwarded in the right order
            //         - initiator has been checked to come from this topology and
            //           is allowed by the API contract to be NULL
            let (targets, values) = unsafe {
                self_
                    .array_query(
                        "hwloc_memattr_get_targets",
                        ptr::null(),
                        |topology, attribute, flags, nr, targets, values| {
                            hwlocality_sys::hwloc_memattr_get_targets(
                                topology, attribute, &initiator, flags, nr, targets, values,
                            )
                        },
                    )
                    .map_err(HybridError::Hwloc)?
            };

            // Convert target list into a safe high-level form
            let targets = targets
                .into_iter()
                // SAFETY: Targets originate from a query against this topology
                .map(|node_ptr| unsafe { self_.encapsulate_target_node(node_ptr) })
                .collect();
            Ok((targets, values))
        }
        polymorphized(self, initiator.map(Into::into))
    }

    /// Initiators that have values for a given attribute for a specific target
    /// NUMA node, along with the associated values
    ///
    /// If this memory attribute has no initiator, an empty list is returned.
    ///
    /// This function is meant for tools and debugging (listing internal
    /// information) rather than for application queries. Applications should
    /// rather select useful NUMA nodes with [`Topology::local_numa_nodes()`]
    /// and then look at their attribute values.
    ///
    /// # Errors
    ///
    /// - [`NoInitiators`] if this memory attribute doesn't have initiators
    /// - [`ForeignTarget`] if `target` does not belong to this topology
    ///
    /// [`ForeignTarget`]: InitiatorQueryError::ForeignTarget
    /// [`NoInitiators`]: InitiatorQueryError::NoInitiators
    #[doc(alias = "hwloc_memattr_get_initiators")]
    pub fn initiators(
        &self,
        target_node: &TopologyObject,
    ) -> Result<(Vec<MemoryAttributeLocation<'topology>>, Vec<u64>), HybridError<InitiatorQueryError>>
    {
        // Validate the query
        self.check_initiator_query_target(target_node)?;

        // Run the query
        // SAFETY: - hwloc_memattr_get_initiators is indeed an array query
        //         - Parameters are forwarded in the right order
        //         - target_node has been checked to come from this topology
        let (initiators, values) = unsafe {
            self.array_query(
                "hwloc_memattr_get_initiators",
                NULL_LOCATION,
                |topology, attribute, flags, nr, initiators, values| {
                    hwlocality_sys::hwloc_memattr_get_initiators(
                        topology,
                        attribute,
                        target_node.as_inner(),
                        flags,
                        nr,
                        initiators,
                        values,
                    )
                },
            )
            .map_err(HybridError::Hwloc)?
        };

        // Convert initiators into a safe high-level form
        let initiators = initiators
            .into_iter()
            // SAFETY: Initiators originate from a query against this topology
            .map(|initiator| unsafe { self.encapsulate_initiator(initiator) })
            .collect();
        Ok((initiators, values))
    }

    /// Perform a `get_targets` style memory attribute query
    ///
    /// # Safety
    ///
    /// `query` must be one of the `hwloc_memattr_<plural>` queries, with the
    /// parameter list simplified to the elements that are common to all of
    /// these queries:
    ///
    /// - Topology
    /// - Memory attribute id
    /// - Flags
    /// - In/out number of memory attribute values
    /// - Output endpoint buffer with capacity given above
    /// - Output value buffer with capacity given above
    unsafe fn array_query<Endpoint: Copy>(
        &self,
        api: &'static str,
        placeholder: Endpoint,
        mut query: impl FnMut(
            hwloc_const_topology_t,
            hwloc_memattr_id_t,
            c_ulong,
            *mut c_uint,
            *mut Endpoint,
            *mut u64,
        ) -> c_int,
    ) -> Result<(Vec<Endpoint>, Vec<u64>), RawHwlocError> {
        // Prepare to call hwloc
        let mut nr = 0;
        let mut call_ffi = |nr_mut, endpoint_out, values_out| {
            errors::call_hwloc_int_normal(api, || {
                // SAFETY: - Topology is trusted to contain a valid ptr (type invariant)
                //         - hwloc ops are trusted not to modify *const parameters
                //         - id is trusted to be valid (type invariant)
                //         - flags must be 0 for all of these queries
                //         - Correctness of nr_mut, enpoint_out and values_out
                //           is call site dependent, see below
                query(
                    self.topology.as_ptr(),
                    self.id,
                    0,
                    nr_mut,
                    endpoint_out,
                    values_out,
                )
            })
        };

        // Allocate arrays of endpoints and values
        // SAFETY: 0 elements + null buffer pointers is the correct way to
        //         request the buffer size to be allocated from hwloc
        call_ffi(&mut nr, ptr::null_mut(), ptr::null_mut())?;
        let len = int::expect_usize(nr);
        let mut endpoints = vec![placeholder; len];
        let mut values = vec![u64::MAX; len];

        // Let hwloc fill the arrays
        let old_nr = nr;
        // SAFETY: - endpoints and values are indeed arrays of nr = len elements
        //         - Input array contents don't matter as this is an out-parameter
        call_ffi(&mut nr, endpoints.as_mut_ptr(), values.as_mut_ptr())?;
        assert_eq!(old_nr, nr, "Inconsistent node count from hwloc");
        Ok((endpoints, values))
    }

    /// Perform a `get_best_initiator`-style memory attribute query, assuming
    /// the query arguments have been checked for correctness
    ///
    /// # Safety
    ///
    /// `query` must be one of the `hwloc_memattr_get_best_` queries, with the
    /// parameter list simplified to the elements that are common to all of
    /// these queries:
    ///
    /// - Topology
    /// - Memory attribute id
    /// - Flags
    /// - Best value output
    unsafe fn get_best(
        &self,
        api: &'static str,
        query: impl FnOnce(hwloc_const_topology_t, hwloc_memattr_id_t, c_ulong, *mut u64) -> c_int,
    ) -> Option<u64> {
        /// Polymorphized subset of this function (avoids generics code bloat)
        fn process_result(final_value: u64, result: Result<c_uint, RawHwlocError>) -> Option<u64> {
            match result {
                Ok(_positive) => Some(final_value),
                Err(RawHwlocError {
                    errno: Some(Errno(ENOENT)),
                    ..
                }) => None,
                // All cases other than "no such attribute" should be handled by the caller
                Err(RawHwlocError {
                    errno: Some(Errno(EINVAL)),
                    ..
                }) => unreachable!("Parameters should have been checked by the caller"),
                Err(raw_err) => unreachable!("Unexpected hwloc error: {raw_err}"),
            }
        }

        // Call hwloc and process its results
        let mut value = u64::MAX;
        let result = errors::call_hwloc_int_normal(api, || {
            // SAFETY: - Topology is trusted to contain a valid ptr (type invariant)
            //         - hwloc ops are trusted not to modify *const parameters
            //         - id is trusted to be valid (type invariant)
            //         - flags must be 0 for all of these queries
            //         - value is an out-parameter, input value doesn't matter
            query(self.topology.as_ptr(), self.id, 0, &mut value)
        });
        process_result(value, result)
    }

    /// Encapsulate a `*const TopologyObject` to a target NUMA node from hwloc
    ///
    /// # Safety
    ///
    /// `node_ptr` must originate from a query against this attribute
    unsafe fn encapsulate_target_node(
        &self,
        node_ptr: *const hwloc_obj,
    ) -> &'topology TopologyObject {
        assert!(!node_ptr.is_null(), "Got null target pointer from hwloc");
        // SAFETY: Lifetime per input precondition, query output assumed valid
        unsafe { (&*node_ptr).as_newtype() }
    }

    /// Encapsulate an initiator location from hwloc
    ///
    /// # Safety
    ///
    /// `initiator` must originate a query against this attribute
    unsafe fn encapsulate_initiator(
        &self,
        initiator: hwloc_location,
    ) -> MemoryAttributeLocation<'topology> {
        // SAFETY: Lifetime per input precondition, query output assumed valid
        unsafe { MemoryAttributeLocation::from_raw(self.topology, initiator) }
            .expect("Failed to decode location from hwloc")
    }

    /// Check the initiator argument to some query, then translate it into the
    /// lower-level format that hwloc expects
    ///
    /// If `is_optional` is true, it is okay not to provide an initiator even
    /// if the memory attribute has flag [`MemoryAttributeFlags::NEED_INITIATOR`].
    ///
    /// # Safety
    ///
    /// - Do not use the output after the `'initiator` lifetime has expired.
    /// - `is_optional` should only be set to `true` for recipients that are
    ///   documented to accept NULL initiators.
    #[allow(clippy::needless_lifetimes)]
    unsafe fn checked_initiator<'initiator>(
        &self,
        initiator: Option<MemoryAttributeLocation<'initiator>>,
        is_optional: bool,
    ) -> Result<hwloc_location, InitiatorInputError> {
        // Collect flags
        let flags = self.flags();

        // Handle missing initiator case
        if flags.contains(MemoryAttributeFlags::NEED_INITIATOR)
            && initiator.is_none()
            && !is_optional
        {
            return Err(InitiatorInputError::NeedInitiator(self.error_name()));
        }

        // Handle unexpected initiator case
        if !flags.contains(MemoryAttributeFlags::NEED_INITIATOR) && initiator.is_some() {
            return Err(InitiatorInputError::UnwantedInitiator(self.error_name()));
        }

        // Handle expected absence of initiator
        let Some(initiator) = initiator else {
            // SAFETY: Per input precondition of is_optional + check above that
            //         initiator can only be none if initiator is optional
            return Ok(NULL_LOCATION);
        };

        // Make sure initiator does belong to this topology
        // SAFETY: Per function precondition on output usage
        unsafe {
            initiator
                .to_checked_raw(self.topology)
                .map_err(InitiatorInputError::ForeignInitiator)
        }
    }

    /// Check the target argument to some initiator query
    fn check_initiator_query_target(
        &self,
        target: &TopologyObject,
    ) -> Result<(), InitiatorQueryError> {
        if !self.flags().contains(MemoryAttributeFlags::NEED_INITIATOR) {
            return Err(InitiatorQueryError::NoInitiators(self.error_name()));
        }
        if !self.topology.contains(target) {
            return Err(target.into());
        }
        Ok(())
    }

    /// Translate this attribute's name to a form suitable for error reporting
    fn error_name(&self) -> Box<str> {
        self.name().to_string_lossy().into()
    }
}

/// An invalid initiator was passed to a memory attribute function
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum InitiatorInputError {
    /// Provided initiator is a [`TopologyObject`] that does not belong to this
    /// topology
    #[error("memory attribute initiator {0}")]
    ForeignInitiator(#[from] ForeignObjectError),

    /// An initiator had to be provided, but was not provided
    #[error("memory attribute {0} needs an initiator but none was provided")]
    NeedInitiator(Box<str>),

    /// No initiator should have been provided, but one was provided
    #[error("memory attribute {0} has no initiator but an initiator was provided")]
    UnwantedInitiator(Box<str>),
}
//
impl<'topology> From<&'topology TopologyObject> for InitiatorInputError {
    fn from(object: &'topology TopologyObject) -> Self {
        Self::ForeignInitiator(object.into())
    }
}

/// A query for memory attribute initiators is invalid
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum InitiatorQueryError {
    /// The specified `target` object does not belong to this topology
    #[error("memory attribute target {0}")]
    ForeignTarget(#[from] ForeignObjectError),

    /// This memory attribute doesn't have initiators
    #[error("memory attribute {0} has no initiator but its initiator was queried")]
    NoInitiators(Box<str>),
}
//
impl<'topology> From<&'topology TopologyObject> for InitiatorQueryError {
    fn from(object: &'topology TopologyObject) -> Self {
        Self::ForeignTarget(object.into())
    }
}

/// A memory attribute value query is invalid
#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
pub enum ValueQueryError {
    /// Specified `initiator` is bad
    #[error(transparent)]
    BadInitiator(#[from] InitiatorInputError),

    /// Specified target object does not belong to this topology
    #[error("memory attribute target {0}")]
    ForeignTarget(ForeignObjectError),
}

/// Where to measure attributes from
#[doc(alias = "hwloc_location")]
#[doc(alias = "hwloc_location::location")]
#[doc(alias = "hwloc_location::type")]
#[doc(alias = "hwloc_location_u")]
#[doc(alias = "hwloc_location::hwloc_location_u")]
#[doc(alias = "hwloc_location_type_e")]
#[derive(Clone, Copy, Debug, Display)]
pub enum MemoryAttributeLocation<'target> {
    /// Directly provide CPU set to find NUMA nodes with corresponding locality
    ///
    /// This is the only initiator type supported by most memory attribute
    /// queries on hwloc-defined memory attributes, though `Object` remains an
    /// option for user-defined memory attributes.
    #[doc(alias = "HWLOC_LOCATION_TYPE_CPUSET")]
    CpuSet(BitmapRef<'target, CpuSet>),

    /// Use a topology object as an initiator
    ///
    /// Most memory attribute queries on hwloc-defined memory attributes do not
    /// support this initiator type, or translate it to a cpuset (going up the
    /// ancestor chain if necessary). But user-defined memory attributes may for
    /// instance use it to provide custom information about host memory accesses
    /// performed by GPUs.
    ///
    /// Only objects belonging to the topology to which memory attributes are
    /// attached should be used here.
    #[doc(alias = "HWLOC_LOCATION_TYPE_OBJECT")]
    Object(&'target TopologyObject),
}
//
impl<'target> MemoryAttributeLocation<'target> {
    /// Convert to the C representation for the purpose of running an hwloc
    /// query against `topology`.
    ///
    /// # Errors
    ///
    /// [`ForeignObjectError`] if this [`MemoryAttributeLocation`] is constructed
    /// from an `&TopologyObject` that does not belong to the target [`Topology`].
    ///
    /// # Safety
    ///
    /// Do not use the output after the source lifetime has expired
    pub(crate) unsafe fn to_checked_raw(
        self,
        topology: &Topology,
    ) -> Result<hwloc_location, ForeignObjectError> {
        match self {
            Self::CpuSet(cpuset) => Ok(hwloc_location {
                ty: HWLOC_LOCATION_TYPE_CPUSET,
                location: hwloc_location_u {
                    cpuset: cpuset.as_ptr(),
                },
            }),
            Self::Object(object) => {
                if topology.contains(object) {
                    Ok(hwloc_location {
                        ty: HWLOC_LOCATION_TYPE_OBJECT,
                        location: hwloc_location_u {
                            object: object.as_inner(),
                        },
                    })
                } else {
                    Err(object.into())
                }
            }
        }
    }

    /// Convert from the C representation
    ///
    /// # Safety
    ///
    /// This function should only be used to encapsulate [`hwloc_location`] structs
    /// from hwloc topology queries, and the `_topology` parameter should match
    /// the [`Topology`] from which the location was extracted.
    unsafe fn from_raw(
        _topology: &'target Topology,
        raw: hwloc_location,
    ) -> Result<Self, LocationTypeError> {
        // SAFETY: - Location type information from hwloc is assumed to be
        //           correct and tells us which union variant we should read.
        //         - Pointer is assumed to point to a valid CpuSet or
        //           TopologyObject that is owned by _topology, and thus has a
        //           lifetime of 'target or greater.
        unsafe {
            match raw.ty {
                HWLOC_LOCATION_TYPE_CPUSET => {
                    let ptr = NonNull::new(raw.location.cpuset.cast_mut())
                        .expect("Unexpected null CpuSet from hwloc");
                    let cpuset = CpuSet::borrow_from_nonnull(ptr);
                    Ok(MemoryAttributeLocation::CpuSet(cpuset))
                }
                HWLOC_LOCATION_TYPE_OBJECT => {
                    let ptr = NonNull::new(raw.location.object.cast_mut())
                        .expect("Unexpected null TopologyObject from hwloc");
                    Ok(MemoryAttributeLocation::Object(ptr.as_ref().as_newtype()))
                }
                unknown => Err(LocationTypeError(unknown)),
            }
        }
    }
}
//
impl<'target> From<BitmapRef<'target, CpuSet>> for MemoryAttributeLocation<'target> {
    fn from(cpuset: BitmapRef<'target, CpuSet>) -> Self {
        Self::CpuSet(cpuset)
    }
}
//
impl<'target> From<&'target CpuSet> for MemoryAttributeLocation<'target> {
    fn from(cpuset: &'target CpuSet) -> Self {
        BitmapRef::from(cpuset).into()
    }
}
//
impl<'target> From<&'target TopologyObject> for MemoryAttributeLocation<'target> {
    fn from(object: &'target TopologyObject) -> Self {
        Self::Object(object)
    }
}

/// Error returned when an unknown location type is observed
#[derive(Copy, Clone, Debug, Eq, Error, From, Hash, PartialEq)]
#[error("hwloc provided a memory attribute location of unknown type {0}")]
struct LocationTypeError(c_int);

/// Invalid `hwloc_location`, which hwloc is assumed not to observe
///
/// # Safety
///
/// Do not expose this location value to an hwloc function that actually
/// reads it, unless it is explicitly documented to accept NULL locations.
const NULL_LOCATION: hwloc_location = hwloc_location {
    ty: HWLOC_LOCATION_TYPE_CPUSET,
    location: hwloc_location_u {
        cpuset: ptr::null(),
    },
};

bitflags! {
    /// Flags for selecting more target NUMA nodes
    ///
    /// By default only NUMA nodes whose locality is exactly the given location
    /// are selected.
    #[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
    #[doc(alias = "hwloc_local_numanode_flag_e")]
    pub struct LocalNUMANodeFlags: hwloc_local_numanode_flag_e {
        /// Select NUMA nodes whose locality is larger than the given cpuset
        ///
        /// For instance, if a single PU (or its cpuset) is given in `initiator`,
        /// select all nodes close to the package that contains this PU.
        #[doc(alias = "HWLOC_LOCAL_NUMANODE_FLAG_LARGER_LOCALITY")]
        const LARGER_LOCALITY = HWLOC_LOCAL_NUMANODE_FLAG_LARGER_LOCALITY;

        /// Select NUMA nodes whose locality is smaller than the given cpuset
        ///
        /// For instance, if a package (or its cpuset) is given in `initiator`,
        /// also select nodes that are attached to only a half of that package.
        #[doc(alias = "HWLOC_LOCAL_NUMANODE_FLAG_SMALLER_LOCALITY")]
        const SMALLER_LOCALITY = HWLOC_LOCAL_NUMANODE_FLAG_SMALLER_LOCALITY;

        /// Select all NUMA nodes in the topology
        ///
        /// The initiator is ignored.
        ///
        /// This flag is automatically set when users specify
        /// [`TargetNumaNodes::All`] as the target NUMA node set.
        #[doc(hidden)]
        const ALL = HWLOC_LOCAL_NUMANODE_FLAG_ALL;
    }
}
//
crate::impl_arbitrary_for_bitflags!(LocalNUMANodeFlags, hwloc_local_numanode_flag_e);

/// Target NUMA nodes
#[derive(Copy, Clone, Debug)]
pub enum TargetNumaNodes<'target> {
    /// Nodes local to some object
    Local {
        /// Initiator which NUMA nodes should be local to
        ///
        /// By default, the search only returns NUMA nodes whose locality is
        /// exactly the given `location`. More nodes can be selected using
        /// `flags`.
        location: MemoryAttributeLocation<'target>,

        /// Flags for enlarging the NUMA node search
        flags: LocalNUMANodeFlags,
    },

    /// All NUMA nodes in the topology
    #[doc(alias = "HWLOC_LOCAL_NUMANODE_FLAG_ALL")]
    All,
}
//
impl TargetNumaNodes<'_> {
    /// Convert to the inputs expected by a `hwloc_get_local_numanode_objs`
    /// query against `topology`
    ///
    /// # Errors
    ///
    /// [`ForeignObjectError`] if the input location is a [`TopologyObject`] that
    /// does not belong to the target [`Topology`]
    ///
    /// # Safety
    ///
    /// Do not use the output raw location after the source lifetime has expired
    pub(crate) unsafe fn to_checked_raw(
        self,
        topology: &Topology,
    ) -> Result<(hwloc_location, LocalNUMANodeFlags), ForeignObjectError> {
        match self {
            Self::Local {
                location,
                mut flags,
            } => {
                flags.remove(LocalNUMANodeFlags::ALL);
                // SAFETY: Per function precondition
                Ok((unsafe { location.to_checked_raw(topology)? }, flags))
            }
            // SAFETY: In presence of the ALL flag, the initiator is ignored,
            //         so a null location is fine.
            Self::All => Ok((NULL_LOCATION, LocalNUMANodeFlags::ALL)),
        }
    }
}
//
impl<'target, T: Into<MemoryAttributeLocation<'target>>> From<T> for TargetNumaNodes<'target> {
    fn from(location: T) -> Self {
        Self::Local {
            location: location.into(),
            flags: LocalNUMANodeFlags::empty(),
        }
    }
}

bitflags! {
    /// Memory attribute flags
    ///
    /// Exactly one of the `HIGHER_IS_BEST` and `LOWER_IS_BEST` flags must be set.
    #[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
    #[doc(alias = "hwloc_memattr_flag_e")]
    pub struct MemoryAttributeFlags: hwloc_memattr_flag_e {
        /// The best nodes for this memory attribute are those with the higher
        /// values
        ///
        /// This flag is mutually exclusive with [`LOWER_IS_BEST`].
        ///
        /// See for instance [`MemoryAttribute::bandwidth()`].
        ///
        /// [`LOWER_IS_BEST`]: Self::LOWER_IS_BEST
        #[doc(alias = "HWLOC_MEMATTR_FLAG_HIGHER_FIRST")]
        const HIGHER_IS_BEST = HWLOC_MEMATTR_FLAG_HIGHER_FIRST;

        /// The best nodes for this memory attribute are those with the lower
        /// values
        ///
        /// This flag is mutually exclusive with [`HIGHER_IS_BEST`].
        ///
        /// See for instance [`MemoryAttribute::latency()`].
        ///
        /// [`HIGHER_IS_BEST`]: Self::HIGHER_IS_BEST
        #[doc(alias = "HWLOC_MEMATTR_FLAG_LOWER_FIRST")]
        const LOWER_IS_BEST = HWLOC_MEMATTR_FLAG_LOWER_FIRST;

        /// The value returned for this memory attribute depends on the given
        /// initiator
        ///
        /// See for instance [`bandwidth()`] and [`latency()`], but not
        /// [`capacity()`].
        ///
        /// [`bandwidth()`]: MemoryAttribute::bandwidth()
        /// [`latency()`]: MemoryAttribute::latency()
        /// [`capacity()`]: MemoryAttribute::capacity()
        #[doc(alias = "HWLOC_MEMATTR_FLAG_NEED_INITIATOR")]
        const NEED_INITIATOR = HWLOC_MEMATTR_FLAG_NEED_INITIATOR;
    }
}
//
impl MemoryAttributeFlags {
    /// Truth that these flags are in a valid state
    pub(crate) fn is_valid(self) -> bool {
        self.contains(Self::HIGHER_IS_BEST) ^ self.contains(Self::LOWER_IS_BEST)
    }
}
//
crate::impl_arbitrary_for_bitflags!(MemoryAttributeFlags, hwloc_memattr_flag_e);
