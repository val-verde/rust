use crate::spec::crt_objects::{self, CrtObjectsFallback};
use crate::spec::{LinkArgs, LinkerFlavor, LldFlavor, TargetOptions};

pub fn opts() -> TargetOptions {
    let mut pre_link_args = LinkArgs::new();
    pre_link_args.insert(
        LinkerFlavor::Gcc,
        vec![
            // Tell GCC to avoid linker plugins, because we are not bundling
            // them with Windows installer, and Rust does its own LTO anyways.
            "-fno-use-linker-plugin".to_string(),
            // Always enable DEP (NX bit) when it is available
            "-Wl,--nxcompat".to_string(),
            // Enable ASLR
            "-Wl,--dynamicbase".to_string(),
            // ASLR will rebase it anyway so leaving that option enabled only leads to confusion
            "-Wl,--disable-auto-image-base".to_string(),
        ],
    );

    let mut late_link_args = LinkArgs::new();
    let mut late_link_args_dynamic = LinkArgs::new();
    let mut late_link_args_static = LinkArgs::new();
    // Order of `late_link_args*` was found through trial and error to work with various
    // mingw-w64 versions (not tested on the CI). It's expected to change from time to time.
    let mingw_libs = vec![
        "-lmsvcrt".to_string(),
        "-lmingwex".to_string(),
        "-lmingw32".to_string(),
        // mingw's msvcrt is a weird hybrid import library and static library.
        // And it seems that the linker fails to use import symbols from msvcrt
        // that are required from functions in msvcrt in certain cases. For example
        // `_fmode` that is used by an implementation of `__p__fmode` in x86_64.
        // The library is purposely listed twice to fix that.
        //
        // See https://github.com/rust-lang/rust/pull/47483 for some more details.
        "-lmsvcrt".to_string(),
        "-luser32".to_string(),
        "-lkernel32".to_string(),
    ];
    late_link_args.insert(LinkerFlavor::Gcc, mingw_libs.clone());
    late_link_args.insert(LinkerFlavor::Lld(LldFlavor::Ld), mingw_libs);
    let dynamic_unwind_libs = vec![
        "-lkernel32".to_string(),
        "-lunwind".to_string(),
    ];
    late_link_args_dynamic.insert(LinkerFlavor::Gcc, dynamic_unwind_libs.clone());
    late_link_args_dynamic.insert(LinkerFlavor::Lld(LldFlavor::Ld), dynamic_unwind_libs);
    let static_unwind_libs = vec![
        "-l:libunwind.a".to_string(),
        "-l:libpthread.a".to_string(),
    ];
    late_link_args_static.insert(LinkerFlavor::Gcc, static_unwind_libs.clone());
    late_link_args_static.insert(LinkerFlavor::Lld(LldFlavor::Ld), static_unwind_libs);

    TargetOptions {
        os: "windows".to_string(),
        env: "gnu".to_string(),
        vendor: "pc".to_string(),
        // FIXME(#13846) this should be enabled for windows
        function_sections: false,
        linker: Some("gcc".to_string()),
        dynamic_linking: true,
        executables: true,
        dll_prefix: String::new(),
        dll_suffix: ".dll".to_string(),
        exe_suffix: ".exe".to_string(),
        staticlib_prefix: "lib".to_string(),
        staticlib_suffix: ".a".to_string(),
        os_family: Some("windows".to_string()),
        is_like_windows: true,
        allows_weak_linkage: false,
        pre_link_args,
        pre_link_objects: crt_objects::pre_mingw(),
        post_link_objects: crt_objects::post_mingw(),
        pre_link_objects_fallback: crt_objects::pre_mingw_fallback(),
        post_link_objects_fallback: crt_objects::post_mingw_fallback(),
        crt_objects_fallback: Some(CrtObjectsFallback::Mingw),
        late_link_args,
        late_link_args_dynamic,
        late_link_args_static,
        abi_return_struct_as_int: true,
        emit_debug_gdb_scripts: false,
        requires_uwtable: true,
        eh_frame_header: false,

        ..Default::default()
    }
}
