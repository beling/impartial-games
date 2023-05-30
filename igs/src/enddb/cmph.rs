use bitm::BitAccess;
use csf::bitvec::{bitvec_with_bits_len_zeroed, BitVec};
use csf::bits_to_store;

type CmphUInt32 = ::std::os::raw::c_uint;
type CmphAlgo = ::std::os::raw::c_uint;
/*const CMPH_ALGO_CMPH_BMZ: CmphAlgo = 0;
const CMPH_ALGO_CMPH_BMZ8: CmphAlgo = 1;
const CMPH_ALGO_CMPH_CHM: CmphAlgo = 2;
const CMPH_ALGO_CMPH_BRZ: CmphAlgo = 3;
const CMPH_ALGO_CMPH_FCH: CmphAlgo = 4;
const CMPH_ALGO_CMPH_BDZ: CmphAlgo = 5;
const CMPH_ALGO_CMPH_BDZ_PH: CmphAlgo = 6;
const CMPH_ALGO_CMPH_CHD_PH: CmphAlgo = 7;*/
const CMPH_ALGO_CMPH_CHD: CmphAlgo = 8;
//const CMPH_ALGO_CMPH_COUNT: CmphAlgo = 9;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct __config_t {
    _unused: [u8; 0],
}
type CmphConfigT = __config_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct __cmph_t {
    _unused: [u8; 0],
}
type CmphT = __cmph_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct cmph_io_adapter_t {
    data: *mut ::std::os::raw::c_void,
    nkeys: CmphUInt32,
    read: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::std::os::raw::c_void,
            arg2: *mut *mut ::std::os::raw::c_char,
            arg3: *mut CmphUInt32,
        ) -> ::std::os::raw::c_int,
    >,
    dispose: ::std::option::Option<
        unsafe extern "C" fn(
            arg1: *mut ::std::os::raw::c_void,
            arg2: *mut ::std::os::raw::c_char,
            arg3: CmphUInt32,
        ),
    >,
    rewind: ::std::option::Option<unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void)>,
}
#[test]
fn bindgen_test_layout_cmph_io_adapter_t() {
    assert_eq!(
        ::std::mem::size_of::<cmph_io_adapter_t>(),
        40usize,
        concat!("Size of: ", stringify!(cmph_io_adapter_t))
    );
    assert_eq!(
        ::std::mem::align_of::<cmph_io_adapter_t>(),
        8usize,
        concat!("Alignment of ", stringify!(cmph_io_adapter_t))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<cmph_io_adapter_t>())).data as *const _ as usize },
        0usize,
        concat!(
        "Offset of field: ",
        stringify!(cmph_io_adapter_t),
        "::",
        stringify!(data)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<cmph_io_adapter_t>())).nkeys as *const _ as usize },
        8usize,
        concat!(
        "Offset of field: ",
        stringify!(cmph_io_adapter_t),
        "::",
        stringify!(nkeys)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<cmph_io_adapter_t>())).read as *const _ as usize },
        16usize,
        concat!(
        "Offset of field: ",
        stringify!(cmph_io_adapter_t),
        "::",
        stringify!(read)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<cmph_io_adapter_t>())).dispose as *const _ as usize },
        24usize,
        concat!(
        "Offset of field: ",
        stringify!(cmph_io_adapter_t),
        "::",
        stringify!(dispose)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<cmph_io_adapter_t>())).rewind as *const _ as usize },
        32usize,
        concat!(
        "Offset of field: ",
        stringify!(cmph_io_adapter_t),
        "::",
        stringify!(rewind)
        )
    );
}

#[link(name = "cmph")]
extern "C" {
    fn cmph_io_struct_vector_adapter(
        vector: *const ::std::os::raw::c_void,
        struct_size: CmphUInt32,
        key_offset: CmphUInt32,
        key_len: CmphUInt32,
        nkeys: CmphUInt32,
    ) -> *mut cmph_io_adapter_t;
    fn cmph_io_struct_vector_adapter_destroy(key_source: *mut cmph_io_adapter_t);

    fn cmph_config_new(key_source: *mut cmph_io_adapter_t) -> *mut CmphConfigT;
    fn cmph_config_destroy(mph: *mut CmphConfigT);

    fn cmph_config_set_algo(mph: *mut CmphConfigT, algo: CmphAlgo);
    fn cmph_config_set_graphsize(mph: *mut CmphConfigT, c: f64);
    fn cmph_config_set_b(mph: *mut CmphConfigT, b: CmphUInt32);

    fn cmph_new(mph: *mut CmphConfigT) -> *mut CmphT;
    fn cmph_destroy(mphf: *mut CmphT);
    fn cmph_size(mphf: *mut CmphT) -> CmphUInt32;

    fn cmph_packed_size(mphf: *mut CmphT) -> CmphUInt32;
    fn cmph_pack(mphf: *mut CmphT, packed_mphf: *mut ::std::os::raw::c_void);
    fn cmph_search_packed(
        packed_mphf: */*mut*/const ::std::os::raw::c_void,
        key: *const ::std::os::raw::c_char,
        keylen: CmphUInt32,
    ) -> CmphUInt32;
}

pub struct ClusterCMPH {
    packed_hash: Box<[u8]>,
    nimbers: Box<[u64]>,
    bits_per_value: u8
}

impl ClusterCMPH {
    #[inline] fn get_index(packed_hash: &[u8], p: u32) -> usize {
        unsafe{cmph_search_packed(
            packed_hash.as_ptr() as *const ::std::os::raw::c_void,
            p.to_le_bytes().as_ptr() as *const i8,
            4) as usize}
    }

    #[inline] pub fn get(&self, p: u32) -> u64 {
        self.nimbers.get_fragment(Self::get_index(self.packed_hash.as_ref(), p), self.bits_per_value)
    }

    /// lambda is an average number of keys per bucket and it can be tuned to obtain different trade-offs between generation time and representation size
    pub fn from_kv_bpv_lambda(keys: &[u32], values: &[u8], bits_per_value: u8, lambda: u8) -> Self {
        unsafe {
            let source = cmph_io_struct_vector_adapter(
                keys.as_ptr() as *const ::std::os::raw::c_void,         // structs
                4, // struct_size
                0,           // key_offset
                4, // key_len
                keys.len() as u32); // nkeys

            let config = cmph_config_new(source);
            //cmph_config_set_algo(config, CMPH_CHD_PH); // CMPH_CHD or CMPH_BDZ
            cmph_config_set_algo(config, CMPH_ALGO_CMPH_CHD); // CMPH_CHD or CMPH_BDZ
            cmph_config_set_graphsize(config, 1.01);
            cmph_config_set_b(config, lambda as CmphUInt32);
            let hash = cmph_new(config);
            cmph_config_destroy(config);
            cmph_io_struct_vector_adapter_destroy(source);//was: cmph_io_vector_adapter_destroy(source);
            //to_find_perfect_hash.release();

            //let mut packed_hash = vec![MaybeUninit::<u8>::uninit(); cmph_packed_size(hash) as usize].into_boxed_slice();
            let mut packed_hash = vec![0u8; cmph_packed_size(hash) as usize].into_boxed_slice();
            let nimbers_bits_len = cmph_size(hash) as usize * bits_per_value as usize;
            cmph_pack(hash, packed_hash.as_mut_ptr() as *mut ::std::os::raw::c_void);
            cmph_destroy(hash);

            let mut nimbers = bitvec_with_bits_len_zeroed(nimbers_bits_len);
            for (k, v) in keys.iter().zip(values.iter()) {
                nimbers.init_fragment(Self::get_index(&packed_hash, *k), *v as u64, bits_per_value);
            }

            Self { packed_hash, nimbers, bits_per_value }

            /*CRAM_LOG("CHD"
            # ifdef
            CRAM_CMPH_KEYS_PER_BUCKET
                << "b=" << CRAM_CMPH_KEYS_PER_BUCKET
            # endif
                << ": "
                << packed_hash_size << " (hash) + " << nimbers_size << " (nimbers) = " << total_size
                << " bytes  " << frac(total_size * 8, uncompressed.size()) << " bits/el");

            return total_size;*/

        }
    }

    #[inline]
    pub fn from_kv_lambda(keys: &[u32], values: &[u8], lambda: u8) -> Self {
        let bits_per_value = bits_to_store!(Into::<u64>::into(values.iter().max().unwrap().clone()));
        Self::from_kv_bpv_lambda(keys, values, bits_per_value.max(1), lambda)
    }

    pub fn size_bytes(&self) -> usize {
        self.packed_hash.len() * std::mem::size_of::<u8>() +
            self.nimbers.len() * std::mem::size_of::<u64>() +
            std::mem::size_of_val(self)
    }
}