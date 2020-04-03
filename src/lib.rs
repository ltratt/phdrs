extern crate libc;

use libc::{c_int, size_t};
use std::{
    ffi::{CStr, CString},
    fmt::{self, Debug},
    iter::Iterator,
    os::raw::c_void,
};

// We are using bindgen to access the `dl_iterate_phdr` API and its types. The `libc` crate exposes
// everything we need, but only on *some* platforms. If the `libc` definitions ever become portable
// then it would make sense to use those instead of bindgen. There's an issue for this here:
// https://github.com/rust-lang/libc/issues/1066
//
// Until then, we avoid namespace pollution by containing all the auto-generated stuff inside a
// (private) sub-module, exposing only what we intend to be public.
mod p_ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    #![allow(improper_ctypes)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    // Re-define C types that are usually erased by C macros.
    #[cfg(target_pointer_width = "64")]
    pub type Elf_Phdr = Elf64_Phdr;
    #[cfg(target_pointer_width = "32")]
    pub type Elf_Phdr = Elf32_Phdr;

    #[cfg(target_pointer_width = "64")]
    pub type Elf_Half = Elf64_Half;
    #[cfg(target_pointer_width = "32")]
    pub type Elf_Half = Elf32_Half;

    #[cfg(target_pointer_width = "64")]
    pub type Elf_Addr = Elf64_Addr;
    #[cfg(target_pointer_width = "32")]
    pub type Elf_Addr = Elf32_Addr;

    #[cfg(target_pointer_width = "64")]
    pub type Elf_Off = Elf64_Off;
    #[cfg(target_pointer_width = "32")]
    pub type Elf_Off = Elf32_Off;

    #[cfg(target_pointer_width = "64")]
    pub type Elf_Word = Elf64_Word;
    #[cfg(target_pointer_width = "32")]
    pub type Elf_Word = Elf32_Word;

    #[cfg(target_pointer_width = "64")]
    pub type Elf_Xword = u64;
    #[cfg(target_pointer_width = "32")]
    pub type Elf_Xword = u32;
}

pub use p_ffi::{
    Elf_Addr, Elf_Half, Elf_Off, Elf_Phdr, Elf_Word, Elf_Xword, PF_MASKPROC, PF_R, PF_W, PF_X,
    PT_DYNAMIC, PT_GNU_EH_FRAME, PT_GNU_RELRO, PT_HIOS, PT_HIPROC, PT_INTERP, PT_LOAD, PT_LOOS,
    PT_LOPROC, PT_NOTE, PT_NULL, PT_PHDR, PT_SHLIB, PT_TLS,
};

/// Contains information about an "object" in the virtual address space.
/// This corresponds with a `dl_phdr_info` in C. Note that the contents of the C struct differ
/// between platforms. We expose only the common fields for now.
pub struct Object {
    /// The base address of the object.
    addr: p_ffi::Elf_Addr,
    /// The name of the object.
    name: CString,
    /// Pointer to program headers C array.
    phdrs: *const p_ffi::Elf_Phdr,
    /// The number of program headers.
    num_phdrs: p_ffi::Elf_Half,
}

impl Object {
    /// Returns an iterator over the program headers of an object. Each item in the iterator
    /// corresponds with one ELF segment.
    pub fn iter_phdrs(&self) -> ProgramHeaderIterator {
        ProgramHeaderIterator {
            ptr: self.phdrs,
            num: self.num_phdrs,
        }
    }

    /// Returns the base address of the object in the virtual address space.
    pub fn addr(&self) -> Elf_Addr {
        self.addr
    }

    /// Returns the name of the object.
    pub fn name(&self) -> &CString {
        &self.name
    }

    /// Returns the number of program headers.
    pub fn num_phdrs(&self) -> p_ffi::Elf_Half {
        self.num_phdrs
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Object {{ addr: 0x{:x}, name: {:?}, phdrs: {:?}, num_phdrs: {} }}",
            self.addr, self.name, self.phdrs, self.num_phdrs
        )
    }
}

pub struct ProgramHeader(*const p_ffi::Elf_Phdr);

impl ProgramHeader {
    /// Returns the segment type (as one of the `PT_*` constants).
    pub fn type_(&self) -> Elf_Word {
        (unsafe { *self.0 }).p_type
    }

    /// Returns the segment flags bitfield.
    /// See the `PT_*` constants for the meaning of the bitfield.
    pub fn flags(&self) -> Elf_Word {
        (unsafe { *self.0 }).p_flags
    }

    /// Returns the segment offset.
    pub fn offset(&self) -> Elf_Off {
        (unsafe { *self.0 }).p_offset
    }

    /// Returns the segment virtual address.
    pub fn vaddr(&self) -> Elf_Addr {
        (unsafe { *self.0 }).p_vaddr
    }

    /// Returns the segment physical address.
    /// On modern systems, this is usually reported as the same as the virtual address.
    pub fn paddr(&self) -> Elf_Addr {
        (unsafe { *self.0 }).p_paddr
    }

    /// Returns the size of the segment when on disk.
    pub fn filesz(&self) -> Elf_Xword {
        (unsafe { *self.0 }).p_filesz
    }

    /// Returns the size of the segment when in memory.
    pub fn memsz(&self) -> Elf_Xword {
        (unsafe { *self.0 }).p_memsz
    }

    /// Returns the alignment of the segment.
    pub fn align(&self) -> Elf_Xword {
        (unsafe { *self.0 }).p_align
    }
}

impl Debug for ProgramHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut to_write = String::from("ProgramHeader(");

        let type_ = self.type_();
        let type_str = match type_ {
            PT_NULL => "PT_NULL",
            PT_LOAD => "PT_LOAD",
            PT_DYNAMIC => "PT_DYNAMIC",
            PT_INTERP => "PT_INTERP",
            PT_NOTE => "PT_NOTE",
            PT_SHLIB => "PT_SHLIB",
            PT_PHDR => "PT_PHDR",
            PT_TLS => "PT_TLS",
            PT_LOOS => "PT_LOOS",
            PT_HIOS => "PT_HIOS",
            PT_LOPROC => "PT_LOPROC",
            PT_HIPROC => "PT_HIPROC",
            PT_GNU_EH_FRAME => "PT_GNU_EH_FRAME",
            PT_GNU_RELRO => "PT_GNU_RELRO",
            // Some operating systems define their own non-standard segment types.
            _ => "other",
        };
        to_write.push_str(&format!("typ={} ({}), ", type_, type_str));

        let flags = self.flags();
        let mut flag_strs = Vec::new();
        if flags & PF_X != 0 {
            flag_strs.push("PF_X");
        }
        if flags & PF_W != 0 {
            flag_strs.push("PF_W");
        }
        if flags & PF_R != 0 {
            flag_strs.push("PF_R");
        }
        if flags & PF_MASKPROC != 0 {
            flag_strs.push("PF_MASKPROC");
        }
        to_write.push_str(&format!("flags=<{}>, ", flag_strs.join("|")));

        to_write.push_str(&format!("offset=<0x{:x}>, ", self.offset()));
        to_write.push_str(&format!("vaddr=<0x{:x}>, ", self.vaddr()));
        to_write.push_str(&format!("paddr=<0x{:x}>, ", self.paddr()));
        to_write.push_str(&format!("align=<0x{:x}>)", self.align()));

        write!(f, "{}", to_write)
    }
}

/// An iterator over the program headers of an `Object`.
///
/// Each program header describes an ELF segment loaded in the virtual adress space.
pub struct ProgramHeaderIterator {
    ptr: *const p_ffi::Elf_Phdr, // Pointer to the next raw `Elf_Phdr`.
    num: p_ffi::Elf_Half,        // How many left.
}

impl Iterator for ProgramHeaderIterator {
    type Item = ProgramHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.num == 0 {
            None
        } else {
            let ret = Some(ProgramHeader(self.ptr));
            self.ptr = unsafe { self.ptr.offset(1) };
            self.num -= 1;
            ret
        }
    }
}

/// Returns a `Vec` of objects loaded into the current address space.
pub fn objects() -> Vec<Object> {
    let mut ret = Vec::new();

    // Pushes an `Object` into the result vector on the behalf of C.
    extern "C" fn push_object(objs: &mut Vec<Object>, obj: &p_ffi::dl_phdr_info) {
        let name = unsafe { CStr::from_ptr(obj.dlpi_name) }.to_owned();
        // We have to copy the `dl_phdr_info` struct out, as the same memory buffer is used for
        // each entry during the iteration process. Otherwise we could have used a vector of
        // pointers.
        objs.push(Object {
            addr: obj.dlpi_addr,
            name,
            phdrs: obj.dlpi_phdr,
            num_phdrs: obj.dlpi_phnum,
        });
    }

    // Callback for `dl_iterate_phdr(3)`.
    unsafe extern "C" fn collect_objs(
        info: *mut p_ffi::dl_phdr_info,
        _sz: size_t,
        data: *mut c_void,
    ) -> c_int {
        push_object(&mut *(data as *mut Vec<Object>), &*info); // Get Rust to push the object.
        0
    };

    let ret_void_p = &mut ret as *mut Vec<Object> as *mut c_void;
    unsafe { p_ffi::dl_iterate_phdr(Some(collect_objs), ret_void_p) };

    ret
}

#[cfg(test)]
mod tests {
    use super::objects;
    use std::{env, path::PathBuf};

    const LINUX_VDSO: &str = "linux-vdso.so.1";

    // Check that iteration works.
    // Since the address space is often randomised, there's not a great deal we can actually test.
    #[test]
    fn test_iterate() {
        let objs = objects();
        assert!(objs.len() >= 1); // Should be at-least one object (the binary itself).
        for o in objs {
            assert_ne!(o.addr(), 0);

            let obj_name = o.name().clone().into_string().unwrap();
            let path = if cfg!(target_os = "linux") && obj_name == "" {
                // On Linux, the main binary has an empty name.
                env::current_exe().unwrap()
            } else {
                PathBuf::from(obj_name)
            };

            // Check the object exists on disk (unless it's a VDSO).
            if !(cfg!(target_os = "linux") && path.to_str().unwrap() == LINUX_VDSO) {
                assert!(path.exists());
            }

            assert_ne!(o.num_phdrs(), 0);

            for p in o.iter_phdrs() {
                assert_ne!(p.type_(), 0);
                assert_ne!(p.flags(), 0);
                // Anything is valid for these, so we just check it compiles.
                let _ = p.offset();
                let _ = p.vaddr();
                let _ = p.paddr();
                let _ = p.filesz();
                let _ = p.memsz();
                let _ = p.align();
            }
        }
    }
}
