//! External object support.

use crate::object::LeanObject;

const LEAN_EXTERNAL_TAG: u8 = 254;

pub type LeanExternalFinalize = unsafe extern "C" fn(*mut std::ffi::c_void);
pub type LeanExternalForeach = unsafe extern "C" fn(*mut std::ffi::c_void, *mut LeanObject);

#[repr(C)]
pub struct LeanExternalClass {
    pub finalize: Option<LeanExternalFinalize>,
    pub foreach: Option<LeanExternalForeach>,
}

#[repr(C)]
pub struct LeanExternal {
    pub header: LeanObject,
    pub class: *mut LeanExternalClass,
    pub data: *mut std::ffi::c_void,
}

pub unsafe fn lean_alloc_external(
    cls: *mut LeanExternalClass,
    data: *mut std::ffi::c_void,
) -> *mut LeanObject {
    let size = std::mem::size_of::<LeanExternal>();
    let obj = crate::object::lean_alloc_object(size);
    let ext = obj as *mut LeanExternal;
    (*ext).header.rc = 1;
    (*ext).header.tag = LEAN_EXTERNAL_TAG;
    (*ext).header.other = 0;
    (*ext).header.cs_sz = 0;
    (*ext).class = cls;
    (*ext).data = data;
    obj
}

pub unsafe fn lean_get_external_class(o: *mut LeanObject) -> *mut LeanExternalClass {
    (*(o as *mut LeanExternal)).class
}

pub unsafe fn lean_get_external_data(o: *mut LeanObject) -> *mut std::ffi::c_void {
    (*(o as *mut LeanExternal)).data
}

pub unsafe fn lean_set_external_data(
    o: *mut LeanObject,
    data: *mut std::ffi::c_void,
) -> *mut LeanObject {
    if crate::lean_is_exclusive(o) {
        (*(o as *mut LeanExternal)).data = data;
        o
    } else {
        let cls = lean_get_external_class(o);
        crate::lean_dec(o);
        lean_alloc_external(cls, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_external_basic() {
        unsafe {
            let mut cls = LeanExternalClass {
                finalize: None,
                foreach: None,
            };
            let data = 0x1234usize as *mut std::ffi::c_void;
            let obj = lean_alloc_external(&mut cls, data);
            assert_eq!((*obj).tag, 254);
            assert_eq!((*obj).rc, 1);
            assert_eq!(lean_get_external_data(obj), data);
            assert_eq!(lean_get_external_class(obj) as *const _, &cls as *const _);
            crate::lean_dec(obj);
        }
    }

    #[test]
    fn set_external_data_exclusive() {
        unsafe {
            let mut cls = LeanExternalClass {
                finalize: None,
                foreach: None,
            };
            let data1 = 0x1111usize as *mut std::ffi::c_void;
            let data2 = 0x2222usize as *mut std::ffi::c_void;
            let obj = lean_alloc_external(&mut cls, data1);
            // Object is exclusive (rc=1), so set_external_data should mutate in place
            let obj2 = lean_set_external_data(obj, data2);
            assert_eq!(obj2, obj);
            assert_eq!(lean_get_external_data(obj2), data2);
            crate::lean_dec(obj2);
        }
    }
}
