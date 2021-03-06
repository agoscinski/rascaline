use std::path::PathBuf;
use std::process::Command;


#[test]
fn check_c_api() {
    let mut build_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    build_dir.push("tests");
    build_dir.push("c-api");
    build_dir.push("build");
    std::fs::create_dir_all(&build_dir).expect("failed to create build dir");

    // assume that debug assertion means that we are building the code in
    // debug mode, even if that could be not true in some cases
    let build_type = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };

    let mut cmake_config = Command::new("cmake");
    cmake_config.current_dir(&build_dir);
    cmake_config.arg("..");
    cmake_config.arg(format!("-DCMAKE_BUILD_TYPE={}", build_type));
    let status = cmake_config.status().expect("failed to configure cmake");
    assert!(status.success());

    let mut cmake_build = Command::new("cmake");
    cmake_build.current_dir(&build_dir);
    cmake_build.arg("--build");
    cmake_build.arg(".");
    cmake_build.arg("--config");
    cmake_build.arg(build_type);
    let status = cmake_build.status().expect("failed to build C++ code");
    assert!(status.success());

    let mut ctest = Command::new("ctest");
    ctest.current_dir(&build_dir);
    ctest.arg("--output-on-failure");
    ctest.arg("--C");
    ctest.arg(build_type);
    let status = ctest.status().expect("failed to run tests");
    assert!(status.success());
}
