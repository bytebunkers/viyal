use vm::object::NativeBinding;
use vm::vm::VM;

pub mod math;
pub mod io;
pub mod fs;
pub mod os;
pub mod time;
pub mod http;
pub mod json;
pub mod path;
pub mod process;
pub mod str;

pub fn register_stdlib(vm: &mut VM) {
    // Legacy global functions
    vm.register_native(NativeBinding::new("Math_abs", math::math_abs));
    vm.register_native(NativeBinding::new("Math_sqrt", math::math_sqrt));
    vm.register_native(NativeBinding::new("Math_pow", math::math_pow));
    vm.register_native(NativeBinding::new("print", io::print));
    vm.register_native(NativeBinding::new("readLine", io::read_line));
    vm.register_native(NativeBinding::new("assert", io::assert));
    vm.register_native(NativeBinding::new("assertEq", io::assert_eq));

    // math module
    vm.register_module("math", vec![
        ("abs",    1, math::math_abs),
        ("sqrt",   1, math::math_sqrt),
        ("pow",    2, math::math_pow),
        ("floor",  1, math::floor),
        ("ceil",   1, math::ceil),
        ("round",  1, math::round),
        ("min",    2, math::min),
        ("max",    2, math::max),
        ("clamp",  3, math::clamp),
        ("log",    1, math::log),
        ("log2",   1, math::log2),
        ("log10",  1, math::log10),
        ("sin",    1, math::sin),
        ("cos",    1, math::cos),
        ("tan",    1, math::tan),
        ("PI",     0, math::pi),
        ("E",      0, math::e),
        ("random", 0, math::random),
    ]);

    // fs module
    vm.register_module("fs", vec![
        ("readText",  1, fs::read_text),
        ("writeText", 2, fs::write_text),
        ("append",    2, fs::append),
        ("copy",      2, fs::copy),
        ("delete",    1, fs::delete),
        ("rename",    2, fs::rename),
        ("mkdir",     1, fs::mkdir),
        ("mkdirAll",  1, fs::mkdir_all),
        ("exists",    1, fs::exists),
        ("isFile",    1, fs::is_file),
        ("isDir",     1, fs::is_dir),
        ("readLines", 1, fs::read_lines),
        ("listDir",   1, fs::list_dir),
        ("stat",      1, fs::stat),
    ]);

    // os module
    vm.register_module("os", vec![
        ("getenv",   1, os::get_env),
        ("setenv",   2, os::set_env),
        ("envAll",   0, os::env_all),
        ("cwd",      0, os::cwd),
        ("chdir",    1, os::chdir),
        ("hostname", 0, os::hostname),
        ("platform", 0, os::platform),
        ("args",     0, os::get_args),
        ("sleep",    1, os::sleep),
        ("exit",     1, os::exit),
        ("execute",  1, os::execute),
    ]);

    // time module
    vm.register_module("time", vec![
        ("now",       0, time::now),
        ("nowMillis", 0, time::now_millis),
        ("nowNanos",  0, time::now_nanos),
        ("sleep",     1, time::sleep),
        ("format",    1, time::format),
    ]);

    // http module
    vm.register_module("http", vec![
        ("get",      1, http::get),
        ("post",     2, http::post),
        ("postJson", 2, http::post_json),
        ("put",      2, http::put),
        ("delete",   1, http::delete),
    ]);

    // json module
    vm.register_module("json", vec![
        ("parse",     1, json::parse),
        ("stringify", 1, json::stringify),
    ]);

    // path module
    vm.register_module("path", vec![
        ("join",       2, path::join),
        ("basename",   1, path::basename),
        ("dirname",    1, path::dirname),
        ("extension",  1, path::extension),
        ("stem",       1, path::stem),
        ("isAbsolute", 1, path::is_absolute),
    ]);

    // process module
    vm.register_module("process", vec![
        ("output", 1, process::output),
        ("pid",    0, process::pid),
        ("spawn",  1, process::spawn),
    ]);

    // str module
    vm.register_module("str", vec![
        ("len",        1, str::str_len),
        ("trim",       1, str::str_trim),
        ("trimLeft",   1, str::str_trim_left),
        ("trimRight",  1, str::str_trim_right),
        ("toUpper",    1, str::to_upper),
        ("toLower",    1, str::to_lower),
        ("split",      2, str::split),
        ("join",       2, str::join_str),
        ("contains",   2, str::contains),
        ("startsWith", 2, str::starts_with),
        ("endsWith",   2, str::ends_with),
        ("replace",    3, str::replace),
        ("repeat",     2, str::repeat),
        ("indexOf",    2, str::index_of),
        ("slice",      3, str::slice),
        ("parseInt",   1, str::parse_int),
        ("parseFloat", 1, str::parse_float),
        ("chars",      1, str::chars),
    ]);
}

