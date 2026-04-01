fn main() {
    println!("Compiling protos...");
    tonic_prost_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir("../services/service-auth/src/")
        .compile_protos(&["./main.proto"], &["./"])
        .expect("Failed to compile auth protos");

    tonic_prost_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir("../services/service-users/src/")
        .compile_protos(&["./main.proto"], &["./"])
        .expect("Failed to compile users protos");

    tonic_prost_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir("../services/service-notes/src/")
        .compile_protos(&["./main.proto"], &["./"])
        .expect("Failed to compile notes protos");

    tonic_prost_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir("../services/service-utils/src/")
        .compile_protos(&["./main.proto"], &["./"])
        .expect("Failed to compile utils protos");

    tonic_prost_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .out_dir("../services/service-scraper/src/")
        .compile_protos(&["./main.proto"], &["./"])
        .expect("Failed to compile scraper protos");
}
